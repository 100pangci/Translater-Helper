<script setup lang="ts">
import { onMounted, ref } from "vue";
import { loadConfig, useChat } from "./composables/useChat";
import HomeView from "./views/HomeView.vue";
import SettingsView from "./views/SettingsView.vue";

const { initEventListeners } = useChat();

const view = ref<"home" | "settings">("home");
const apiKeyMissing = ref(false);

onMounted(async () => {
  await initEventListeners();
  try {
    const cfg = await loadConfig();
    apiKeyMissing.value = !cfg.apiKey.trim();
  } catch {
    apiKeyMissing.value = true;
  }
});

async function openSettings() {
  view.value = "settings";
}

async function onSettingsBack() {
  view.value = "home";
  try {
    const cfg = await loadConfig(true);
    apiKeyMissing.value = !cfg.apiKey.trim();
  } catch {
    apiKeyMissing.value = true;
  }
}
</script>

<template>
  <div class="app-shell">
    <header class="topbar">
      <div class="brand">
        <span class="brand-badge">偶</span>
        <div>
          <div>偶译析</div>
          <div class="brand-sub">TRANSLATOR</div>
        </div>
      </div>
      <button class="icon-btn" title="设置" @click="openSettings">
        <svg viewBox="0 0 24 24" width="19" height="19" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="3" />
          <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09a1.65 1.65 0 0 0-1-1.51 1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09a1.65 1.65 0 0 0 1.51-1 1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33h.01a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51h.01a1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82v.01a1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
        </svg>
      </button>
    </header>

    <div v-if="view === 'home' && apiKeyMissing" class="setup-tip">
      <svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="12" cy="12" r="10" />
        <line x1="12" y1="8" x2="12" y2="12" />
        <line x1="12" y1="16" x2="12.01" y2="16" />
      </svg>
      <span>尚未配置 API Key，请先到设置中完成配置</span>
      <button class="tip-btn" @click="openSettings">前往设置 →</button>
    </div>

    <main class="app-main">
      <HomeView v-if="view === 'home'" />
      <SettingsView v-else @back="onSettingsBack" />
    </main>
  </div>
</template>

<style scoped>
.setup-tip {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 28px;
  background: rgba(251, 191, 36, 0.1);
  border-bottom: 1px solid rgba(251, 191, 36, 0.25);
  color: var(--warn);
  font-size: 12.5px;
}
.tip-btn {
  background: none;
  color: var(--warn);
  font-size: 12.5px;
  font-weight: 600;
  padding: 0;
  margin-left: auto;
}
.tip-btn:hover {
  text-decoration: underline;
}
</style>