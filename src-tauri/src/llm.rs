use std::time::Duration;

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::{AppHandle, Emitter};

use crate::config::AppConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

fn join_url(base: &str, path: &str) -> String {
    let base = base.trim_end_matches('/');
    if base.ends_with(path) {
        base.to_string()
    } else {
        format!("{base}/{path}")
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() > max {
        let cut: String = s.chars().take(max).collect();
        format!("{cut}…")
    } else {
        s.to_string()
    }
}

fn emit_chunk(app: &AppHandle, request_id: &str, delta: &str) {
    let _ = app.emit("llm-chunk", json!({ "requestId": request_id, "delta": delta }));
}

fn emit_done(app: &AppHandle, request_id: &str) {
    let _ = app.emit("llm-done", json!({ "requestId": request_id }));
}

fn emit_error(app: &AppHandle, request_id: &str, message: &str) {
    let _ = app.emit("llm-error", json!({ "requestId": request_id, "message": message }));
}

async fn stream_lines<S>(mut stream: S, mut on_line: impl FnMut(&str)) -> Result<(), String>
where
    S: futures_util::Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Unpin,
{
    let mut buffer = String::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("读取流失败: {e}"))?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));
        while let Some(pos) = buffer.find('\n') {
            let line = buffer[..pos].trim().to_string();
            buffer = buffer[pos + 1..].to_string();
            if !line.is_empty() {
                on_line(&line);
            }
        }
    }
    Ok(())
}

async fn openai_stream(
    client: &reqwest::Client,
    app: &AppHandle,
    request_id: &str,
    config: &AppConfig,
    messages: &[ChatMessage],
) -> Result<(), String> {
    let url = join_url(&config.base_url, "chat/completions");
    let mut body = json!({
        "model": config.model,
        "messages": messages,
        "temperature": config.temperature,
        "stream": true,
    });
    if let Some(level) = &config.reasoning_effort {
        if level != "default" {
            body["reasoning_effort"] = json!(level);
        }
    }

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", config.api_key))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("请求失败: {e}"))?;

    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("API 错误 ({status}): {}", truncate(&text, 500)));
    }

    stream_lines(resp.bytes_stream(), |line| {
        let Some(data) = line.strip_prefix("data:") else {
            return;
        };
        let data = data.trim();
        if data == "[DONE]" {
            return;
        }
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(data) {
            if let Some(delta) = v["choices"][0]["delta"]["content"].as_str() {
                emit_chunk(app, request_id, delta);
            }
        }
    })
    .await
}

async fn anthropic_stream(
    client: &reqwest::Client,
    app: &AppHandle,
    request_id: &str,
    config: &AppConfig,
    messages: &[ChatMessage],
) -> Result<(), String> {
    let url = join_url(&config.base_url, "v1/messages");
    let system: String = messages
        .iter()
        .filter(|m| m.role == "system")
        .map(|m| m.content.clone())
        .collect::<Vec<_>>()
        .join("\n");
    let body_messages: Vec<serde_json::Value> = messages
        .iter()
        .filter(|m| m.role != "system")
        .map(|m| json!({ "role": m.role, "content": m.content }))
        .collect();

    let mut body = json!({
        "model": config.model,
        "messages": body_messages,
        "system": system,
        "max_tokens": 4096,
        "temperature": config.temperature,
        "stream": true,
    });
    if let Some(level) = &config.reasoning_effort {
        let budget = match level.as_str() {
            "low" => 1024,
            "medium" => 4096,
            "high" => 8192,
            _ => 0,
        };
        if budget > 0 {
            body["thinking"] = json!({ "type": "enabled", "budget_tokens": budget });
            body["max_tokens"] = json!(budget + 2048);
        }
    }

    let resp = client
        .post(&url)
        .header("x-api-key", &config.api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("请求失败: {e}"))?;

    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("API 错误 ({status}): {}", truncate(&text, 500)));
    }

    stream_lines(resp.bytes_stream(), |line| {
        let Some(data) = line.strip_prefix("data:") else {
            return;
        };
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(data.trim()) {
            if v["type"] == "content_block_delta" && v["delta"]["type"] == "text_delta" {
                if let Some(text) = v["delta"]["text"].as_str() {
                    emit_chunk(app, request_id, text);
                }
            }
        }
    })
    .await
}

#[tauri::command]
pub async fn chat_stream(
    app: AppHandle,
    request_id: String,
    config: AppConfig,
    messages: Vec<ChatMessage>,
) -> Result<(), String> {
    if config.api_key.trim().is_empty() {
        let e = "尚未配置 API Key，请先到「设置」中填写。".to_string();
        emit_error(&app, &request_id, &e);
        return Ok(());
    }

    let client = crate::proxy::configure_client(reqwest::Client::builder())
        .map_err(|e| format!("配置网络代理失败: {e}"))?
        .timeout(Duration::from_secs(600))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {e}"))?;

    let result = if config.provider == "anthropic" {
        anthropic_stream(&client, &app, &request_id, &config, &messages).await
    } else {
        openai_stream(&client, &app, &request_id, &config, &messages).await
    };

    match result {
        Ok(()) => emit_done(&app, &request_id),
        Err(e) => emit_error(&app, &request_id, &e),
    }
    Ok(())
}
