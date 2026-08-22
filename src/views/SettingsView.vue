<script setup lang="ts">
import { onMounted, reactive, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { loadConfig, saveConfig } from "../composables/useChat";
import type { AppConfig, ChatMessage, StreamPayload } from "../types";

const emit = defineEmits<{ back: [] }>();

const DEFAULT_SYSTEM_PROMPT = `你是一个专业的语言翻译与解析助手。请严格按照以下格式处理用户输入：

## 中文翻译
给出忠实、自然、准确的中文翻译。

## 词汇解析
列出原文中的重点词汇与短语，逐个给出中文释义和简明说明。

## 语法与结构
分析原文的句子结构、时态、语态及关键语法点，用中文说明。

## 例句
给出 1-2 个使用相似表达的中文例句，并附对应原文语言。

要求：翻译务必准确完整；解析简明清晰，一律使用中文输出。`;

const form = reactive<AppConfig>({
  provider: "openai",
  baseUrl: "",
  apiKey: "",
  model: "",
  temperature: 0.7,
  reasoningEffort: "default",
  systemPrompt: DEFAULT_SYSTEM_PROMPT,
});

const showKey = ref(false);
const loading = ref(true);
const saved = ref(false);
const testStatus = ref<{ kind: "idle" | "testing" | "ok" | "err"; text: string }>({
  kind: "idle",
  text: "",
});
let saveTimer: number | undefined;

onMounted(async () => {
  try {
    const cfg = await loadConfig(true);
    Object.assign(form, cfg);
    form.reasoningEffort = cfg.reasoningEffort ?? "default";
  } catch (e) {
    testStatus.value = { kind: "err", text: `读取配置失败: ${e}` };
  }
  loading.value = false;
});

async function onSave() {
  try {
    await saveConfig({ ...form });
    saved.value = true;
    window.clearTimeout(saveTimer);
    saveTimer = window.setTimeout(() => (saved.value = false), 2000);
  } catch (e) {
    testStatus.value = { kind: "err", text: `保存失败: ${e}` };
  }
}

async function onTest() {
  testStatus.value = { kind: "testing", text: "正在测试连接…" };
  const rid = `test-${Date.now()}`;
  let received = 0;
  let settled = false;
  const unlisteners: (() => void)[] = [];
  const cleanup = () => unlisteners.forEach((u) => u());

  const done = (kind: "ok" | "err", text: string) => {
    if (settled) return;
    settled = true;
    cleanup();
    testStatus.value = { kind, text };
  };

  unlisteners.push(
    await listen<StreamPayload>("llm-chunk", (e) => {
      if (e.payload.requestId === rid && e.payload.delta) received += e.payload.delta.length;
    }),
  );
  unlisteners.push(
    await listen<StreamPayload>("llm-done", (e) => {
      if (e.payload.requestId === rid) done("ok", `连接成功，返回 ${received} 字符`);
    }),
  );
  unlisteners.push(
    await listen<StreamPayload>("llm-error", (e) => {
      if (e.payload.requestId === rid) done("err", e.payload.message ?? "请求失败");
    }),
  );

  const messages: ChatMessage[] = [{ role: "user", content: "你好，请回复：连接测试成功" }];
  try {
    await invoke("chat_stream", { requestId: rid, config: { ...form }, messages });
  } catch (e) {
    done("err", String(e));
  }
  window.setTimeout(() => !settled && done("err", "测试超时（60 秒）"), 60_000);
}
</script>

<template>
  <div class="settings">
    <div class="container narrow">
      <header class="settings-header fade-up">
        <button class="icon-btn back-btn" title="返回" @click="emit('back')">
          <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M19 12H5" />
            <path d="m12 19-7-7 7-7" />
          </svg>
        </button>
        <div>
          <h2>设置</h2>
          <p>配置大模型 API，支持 OpenAI 兼容协议与 Anthropic 协议</p>
        </div>
      </header>

      <section v-if="loading" class="panel fade-up">
        <div class="panel-loading"><span class="spin"></span> 正在加载配置…</div>
      </section>

      <template v-else>
        <section class="panel fade-up">
          <h3 class="panel-title">API 配置</h3>
          <div class="field">
            <label>协议</label>
            <select v-model="form.provider" class="select">
              <option value="openai">OpenAI 兼容协议（DeepSeek / 通义 / Kimi / GLM 等）</option>
              <option value="anthropic">Anthropic 协议（Claude）</option>
            </select>
          </div>
          <div class="field">
            <label>Base URL</label>
            <input v-model="form.baseUrl" class="input" type="text" placeholder="https://api.deepseek.com" spellcheck="false" />
          </div>
          <div class="field">
            <label>API Key</label>
            <div class="key-row">
              <input v-model="form.apiKey" class="input" :type="showKey ? 'text' : 'password'" placeholder="sk-…" spellcheck="false" autocomplete="off" />
              <button class="icon-btn key-toggle" :title="showKey ? '隐藏' : '显示'" @click="showKey = !showKey">
                <svg v-if="showKey" viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M9.88 9.88a3 3 0 1 0 4.24 4.24" />
                  <path d="M10.73 5.08A10.43 10.43 0 0 1 12 5c7 0 10 7 10 7a13.16 13.16 0 0 1-1.67 2.68" />
                  <path d="M6.61 6.61A13.526 13.526 0 0 0 2 12s3 7 10 7a9.74 9.74 0 0 0 5.39-1.61" />
                  <line x1="2" y1="2" x2="22" y2="22" />
                </svg>
                <svg v-else viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7Z" />
                  <circle cx="12" cy="12" r="3" />
                </svg>
              </button>
            </div>
          </div>
          <div class="field">
            <label>模型</label>
            <input v-model="form.model" class="input" type="text" placeholder="deepseek-chat" spellcheck="false" />
          </div>
          <div class="field">
            <label class="range-label">
              温度（随机性）
              <span class="range-value">{{ form.temperature.toFixed(1) }}</span>
            </label>
            <input v-model.number="form.temperature" class="range" type="range" min="0" max="1" step="0.1" />
          </div>
          <div class="field">
            <label>思考深度</label>
            <select v-model="form.reasoningEffort" class="select">
              <option value="default">默认（不设置，由模型决定）</option>
              <option value="low">低</option>
              <option value="medium">中</option>
              <option value="high">高</option>
            </select>
            <p class="field-hint">
              OpenAI 兼容协议发送 reasoning_effort；Anthropic 协议开启 extended thinking。<br />
              部分模型/服务不支持该参数时可能忽略或报错，请先「测试连接」。
            </p>
          </div>
        </section>

        <section class="panel fade-up">
          <h3 class="panel-title">
            系统提示词
            <button class="link-btn" @click="form.systemPrompt = DEFAULT_SYSTEM_PROMPT">恢复默认</button>
          </h3>
          <textarea v-model="form.systemPrompt" class="textarea prompt-area" rows="12" spellcheck="false"></textarea>
        </section>

        <section class="settings-actions fade-up">
          <div class="test-area">
            <button class="btn-ghost" :disabled="testStatus.kind === 'testing'" @click="onTest">
              <span v-if="testStatus.kind === 'testing'" class="spin"></span>
              测试连接
            </button>
            <span v-if="testStatus.kind === 'ok'" class="test-msg ok">{{ testStatus.text }}</span>
            <span v-else-if="testStatus.kind === 'err'" class="test-msg err">{{ testStatus.text }}</span>
          </div>
          <button class="btn-primary" @click="onSave">
            <span v-if="saved" class="saved-mark">✓</span>
            {{ saved ? "已保存" : "保存配置" }}
          </button>
        </section>
      </template>
    </div>
  </div>
</template>

<style scoped>
.settings {
  padding-top: 12px;
}
.container.narrow {
  max-width: 640px;
}
.settings-header {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 10px 0 20px;
}
.settings-header h2 {
  font-size: 20px;
  font-weight: 700;
}
.settings-header p {
  color: var(--text-dim);
  font-size: 12.5px;
  margin-top: 2px;
}
.back-btn {
  border: 1px solid var(--border);
  background: var(--panel);
  flex-shrink: 0;
}

.panel {
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 18px 20px;
  margin-bottom: 14px;
  box-shadow: var(--shadow-sm);
}
.panel-title {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 13px;
  font-weight: 700;
  color: var(--text-dim);
  letter-spacing: 2px;
  margin-bottom: 16px;
  text-transform: uppercase;
}
.panel-loading {
  display: flex;
  align-items: center;
  gap: 10px;
  color: var(--text-dim);
  padding: 10px 0;
}

.field {
  margin-bottom: 14px;
}
.field label {
  display: block;
  font-size: 12.5px;
  color: var(--text-dim);
  margin-bottom: 7px;
  font-weight: 500;
}
.key-row {
  display: flex;
  gap: 8px;
}
.key-row .input {
  flex: 1;
}
.key-toggle {
  flex-shrink: 0;
  border: 1px solid var(--border);
  background: var(--bg-soft);
}
.range-label {
  display: flex !important;
  align-items: center;
  justify-content: space-between;
}
.range-value {
  color: var(--accent);
  font-weight: 700;
}
.field-hint {
  margin-top: 6px;
  font-size: 12px;
  color: var(--text-faint);
  line-height: 1.7;
}
.range {
  width: 100%;
  appearance: none;
  height: 6px;
  border-radius: 4px;
  background: var(--panel-3);
  outline: none;
  cursor: pointer;
}
.range::-webkit-slider-thumb {
  appearance: none;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: var(--gradient);
  border: 2px solid #fff;
  cursor: pointer;
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.4);
}
.prompt-area {
  font-size: 13px;
  line-height: 1.7;
}
.link-btn {
  background: none;
  color: var(--accent-2);
  font-size: 12px;
  text-transform: none;
  letter-spacing: 0;
  padding: 0;
}
.link-btn:hover {
  text-decoration: underline;
}

.settings-actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-bottom: 12px;
}
.test-area {
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 0;
}
.test-msg {
  font-size: 12.5px;
  word-break: break-all;
}
.test-msg.ok {
  color: var(--ok);
}
.test-msg.err {
  color: var(--danger);
}
.saved-mark {
  color: #fff;
  font-weight: 800;
}
</style>