use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::Manager;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub provider: String,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub temperature: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,
    pub system_prompt: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            provider: "openai".into(),
            base_url: "https://api.deepseek.com".into(),
            api_key: String::new(),
            model: "deepseek-chat".into(),
            temperature: 0.7,
            reasoning_effort: None,
            system_prompt: DEFAULT_SYSTEM_PROMPT.into(),
        }
    }
}

pub const DEFAULT_SYSTEM_PROMPT: &str = "你是一个专业的语言翻译与解析助手。请严格按照以下格式处理用户输入：

## 中文翻译
给出忠实、自然、准确的中文翻译。

## 词汇解析
列出原文中的重点词汇与短语，逐个给出中文释义和简明说明。

## 语法与结构
分析原文的句子结构、时态、语态及关键语法点，用中文说明。

## 例句
给出 1-2 个使用相似表达的中文例句，并附对应原文语言。

要求：翻译务必准确完整；解析简明清晰，一律使用中文输出。";

fn config_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("无法定位配置目录: {e}"))?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("无法创建配置目录: {e}"))?;
    Ok(dir.join("config.json"))
}

#[tauri::command]
pub fn get_config(app: tauri::AppHandle) -> Result<AppConfig, String> {
    let path = config_path(&app)?;
    if path.exists() {
        let raw = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        serde_json::from_str(&raw).map_err(|e| format!("配置文件解析失败: {e}"))
    } else {
        Ok(AppConfig::default())
    }
}

#[tauri::command]
pub fn save_config(app: tauri::AppHandle, config: AppConfig) -> Result<(), String> {
    let path = config_path(&app)?;
    let raw = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    std::fs::write(&path, raw).map_err(|e| format!("保存配置失败: {e}"))
}