import { createApp } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import App from "./App.vue";
import { APP_TITLE } from "./core/version";
import "./styles/main.css";

document.title = APP_TITLE;
if ("__TAURI_INTERNALS__" in window) {
  getCurrentWindow().setTitle(APP_TITLE);
}

createApp(App).mount("#app");
