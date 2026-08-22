<script setup lang="ts">
import { nextTick, ref, watch } from "vue";
import { askFollowUp, discard, translate, useChat } from "../composables/useChat";
import MarkdownView from "../components/MarkdownView.vue";

const { state } = useChat();

const followUpOpen = ref(false);
const followUpText = ref("");
const resultRef = ref<HTMLElement | null>(null);

const lastTurn = () => state.turns[state.turns.length - 1];

async function onTranslate() {
  await translate();
  followUpOpen.value = false;
}

async function onFollowUp() {
  const q = followUpText.value.trim();
  if (!q) return;
  followUpText.value = "";
  await askFollowUp(q);
}

function onDiscard() {
  discard();
  followUpOpen.value = false;
  followUpText.value = "";
}

watch(
  () => lastTurn()?.answer.length ?? 0,
  async () => {
    await nextTick();
    resultRef.value?.scrollTo({ top: resultRef.value.scrollHeight, behavior: "smooth" });
  },
);
</script>

<template>
  <div class="home">
    <div class="container">
      <header class="hero fade-up">
        <h1>偶译析</h1>
        <p>输入任意语言的文本，获得中文翻译与深度解析</p>
      </header>

      <section class="input-card fade-up" :class="{ compact: state.turns.length > 0 }">
        <textarea
          v-model="state.inputText"
          class="translate-input"
          :rows="state.turns.length ? 3 : 6"
          :placeholder="
            state.turns.length
              ? '输入新的文本开始新一轮翻译…'
              : '粘贴任意语言的文本，如英文、日文、代码注释…'
          "
          spellcheck="false"
          @keydown.ctrl.enter="onTranslate"
        ></textarea>
        <div class="input-meta">
          <span class="hint">Ctrl + Enter 快捷翻译</span>
          <div class="input-actions">
            <button
              v-if="state.turns.length"
              class="btn-ghost btn-danger"
              :disabled="state.streaming"
              title="清空当前结果与输入"
              @click="onDiscard"
            >
              <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M3 6h18" />
                <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6" />
                <path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
              </svg>
              丢弃
            </button>
            <button class="btn-primary" :disabled="state.streaming || !state.inputText.trim()" @click="onTranslate">
              <span v-if="state.streaming" class="spin"></span>
              {{ state.streaming ? "生成中…" : "翻译并解析" }}
            </button>
          </div>
        </div>
      </section>

      <div v-if="state.error" class="error-banner fade-up">
        <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <circle cx="12" cy="12" r="10" />
          <line x1="12" y1="8" x2="12" y2="12" />
          <line x1="12" y1="16" x2="12.01" y2="16" />
        </svg>
        <span>{{ state.error }}</span>
      </div>

      <div v-if="!state.turns.length" class="empty-state fade-up">
        <div class="empty-icon">
          <svg viewBox="0 0 24 24" width="30" height="30" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
            <path d="M12 2a10 10 0 1 0 10 10" />
            <path d="M12 6v6l4 2" />
          </svg>
        </div>
        <p class="empty-title">轻量翻译 · 不留历史</p>
        <p class="empty-desc">
          每次请求只发送本次文本，不会堆积上下文。<br />
          翻译结果下方可「丢弃」清空，或「继续提问」单轮追问。
        </p>
      </div>

      <div ref="resultRef" v-else class="result-list">
        <article
          v-for="(turn, i) in state.turns"
          :key="i"
          class="result-card fade-up"
          :class="{ streaming: state.streaming && i === state.turns.length - 1 }"
        >
          <div v-if="turn.isFollowUp" class="followup-tag">追问</div>
          <div v-if="turn.isFollowUp" class="followup-question">{{ turn.question }}</div>

          <div class="result-body">
            <MarkdownView :source="turn.answer" />
            <span v-if="state.streaming && i === state.turns.length - 1 && !turn.answer" class="thinking-hint">
              <span class="spin"></span> 正在思考…
            </span>
          </div>

          <footer v-if="!state.streaming && i === state.turns.length - 1" class="action-bar fade-up">
            <button class="btn-ghost btn-danger" @click="onDiscard">
              <svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M3 6h18" />
                <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6" />
                <path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
              </svg>
              丢弃
            </button>
            <button class="btn-ghost" @click="followUpOpen = !followUpOpen">
              <svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
              </svg>
              继续提问
            </button>
          </footer>

          <div v-if="followUpOpen && !state.streaming && i === state.turns.length - 1" class="followup-panel fade-up">
            <textarea
              v-model="followUpText"
              class="textarea"
              rows="2"
              placeholder="针对这段文本继续提问（仅携带最近一轮结果，不累积历史）…"
              spellcheck="false"
              @keydown.ctrl.enter="onFollowUp"
            ></textarea>
            <div class="followup-actions">
              <span class="hint">Ctrl + Enter 发送</span>
              <button class="btn-primary" :disabled="!followUpText.trim()" @click="onFollowUp">发送追问</button>
            </div>
          </div>
        </article>
      </div>
    </div>
  </div>
</template>

<style scoped>
.home {
  padding-top: 10px;
}

.hero {
  text-align: center;
  padding: 26px 0 24px;
}
.hero h1 {
  font-size: 34px;
  font-weight: 800;
  letter-spacing: 8px;
  background: var(--gradient);
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
  margin-bottom: 6px;
}
.hero p {
  color: var(--text-dim);
  font-size: 13.5px;
}

.input-card {
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 14px;
  box-shadow: var(--shadow-sm);
  transition: padding 0.2s;
}
.translate-input {
  width: 100%;
  resize: vertical;
  min-height: 64px;
  background: var(--bg-soft);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  color: var(--text);
  padding: 12px 14px;
  font-size: 14.5px;
  font-family: inherit;
  line-height: 1.6;
  transition: border-color 0.15s, box-shadow 0.15s;
}
.translate-input:focus {
  outline: none;
  border-color: var(--accent);
  box-shadow: 0 0 0 3px rgba(111, 124, 255, 0.18);
}
.translate-input::placeholder {
  color: var(--text-faint);
}
.input-meta {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-top: 12px;
}
.input-actions {
  display: flex;
  align-items: center;
  gap: 10px;
}
.hint {
  font-size: 12px;
  color: var(--text-faint);
}

.error-banner {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-top: 14px;
  background: rgba(248, 113, 113, 0.1);
  border: 1px solid rgba(248, 113, 113, 0.3);
  color: var(--danger);
  border-radius: var(--radius-sm);
  padding: 10px 16px;
  font-size: 13px;
  word-break: break-all;
}

.empty-state {
  text-align: center;
  padding: 48px 0 30px;
  color: var(--text-faint);
}
.empty-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 64px;
  height: 64px;
  border-radius: 20px;
  background: var(--panel);
  border: 1px solid var(--border);
  color: var(--accent);
  margin-bottom: 16px;
}
.empty-title {
  color: var(--text-dim);
  font-size: 15px;
  font-weight: 600;
  margin-bottom: 8px;
}
.empty-desc {
  font-size: 13px;
  line-height: 1.8;
}

.result-list {
  margin-top: 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.result-card {
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  box-shadow: var(--shadow-sm);
  overflow: hidden;
}
.result-card.streaming {
  border-color: rgba(111, 124, 255, 0.45);
  box-shadow: 0 0 0 3px rgba(111, 124, 255, 0.1), var(--shadow-sm);
}
.result-body {
  padding: 18px 20px 14px;
}
.thinking-hint {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  color: var(--text-faint);
  font-size: 13px;
  padding: 6px 0;
}

.followup-tag {
  display: inline-block;
  margin: 16px 20px 0;
  padding: 2px 10px;
  font-size: 11px;
  font-weight: 600;
  color: var(--accent-2);
  background: rgba(56, 189, 248, 0.12);
  border-radius: 20px;
  border: 1px solid rgba(56, 189, 248, 0.25);
}
.followup-question {
  padding: 8px 20px 0;
  font-size: 13px;
  color: var(--text-dim);
  font-style: italic;
}

.action-bar {
  display: flex;
  gap: 10px;
  padding: 10px 20px 16px;
  border-top: 1px dashed var(--border);
  margin: 0 20px 0;
  padding-bottom: 16px;
  justify-content: flex-end;
}

.followup-panel {
  padding: 14px 20px 18px;
  border-top: 1px dashed var(--border);
  background: var(--bg-soft);
}
.followup-actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-top: 10px;
}
</style>