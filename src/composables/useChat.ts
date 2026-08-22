import { reactive } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { AppConfig, ChatMessage, SessionTurn, StreamPayload } from "../types";

const state = reactive({
  inputText: "",
  turns: [] as SessionTurn[],
  streaming: false,
  error: "",
  configReady: false,
});

let configCache: AppConfig | null = null;
let currentRequestId = "";
let requestSeq = 0;
let listenersPromise: Promise<void> | null = null;

async function initEventListeners() {
  if (listenersPromise) return listenersPromise;
  listenersPromise = (async () => {
    await listen<StreamPayload>("llm-chunk", (e) => {
      if (e.payload.requestId !== currentRequestId || !e.payload.delta) return;
      const last = state.turns[state.turns.length - 1];
      if (last) last.answer += e.payload.delta;
    });
    await listen<StreamPayload>("llm-done", (e) => {
      if (e.payload.requestId !== currentRequestId) return;
      state.streaming = false;
    });
    await listen<StreamPayload>("llm-error", (e) => {
      if (e.payload.requestId !== currentRequestId) return;
      state.streaming = false;
      state.error = e.payload.message ?? "请求失败";
    });
  })();
  return listenersPromise;
}

export async function loadConfig(force = false): Promise<AppConfig> {
  if (!configCache || force) {
    configCache = await invoke<AppConfig>("get_config");
    state.configReady = true;
  }
  return configCache;
}

export async function saveConfig(config: AppConfig) {
  await invoke("save_config", { config });
  configCache = config;
}

function startStream(config: AppConfig, messages: ChatMessage[]) {
  currentRequestId = `req-${++requestSeq}`;
  state.streaming = true;
  state.error = "";
  invoke("chat_stream", { requestId: currentRequestId, config, messages }).catch(
    (err) => {
      state.streaming = false;
      if (!state.error) state.error = String(err);
    },
  );
}

export async function translate() {
  const config = await loadConfig();
  const text = state.inputText.trim();
  if (!text || state.streaming) return;

  state.turns = [];
  state.error = "";
  state.turns.push({ question: text, answer: "", isFollowUp: false });
  startStream(config, [
    { role: "system", content: config.systemPrompt },
    { role: "user", content: text },
  ]);
}

export async function askFollowUp(question: string) {
  const config = await loadConfig();
  const q = question.trim();
  if (!q || state.streaming) return;

  const original = state.turns[0]?.question ?? "";
  const last = state.turns[state.turns.length - 1];
  const context = `【原始文本】\n${original}\n\n【最近一轮结果】\n${last?.answer ?? ""}`;

  state.turns.push({ question: q, answer: "", isFollowUp: true });
  startStream(config, [
    { role: "system", content: config.systemPrompt },
    { role: "user", content: context },
    { role: "user", content: `请针对以上内容，回答这个问题：${q}` },
  ]);
}

export function discard() {
  currentRequestId = "";
  state.turns = [];
  state.inputText = "";
  state.error = "";
  state.streaming = false;
}

export function useChat() {
  return { state, initEventListeners, loadConfig };
}