import "./dev/mockTauri";
import React from "react";
import ReactDOM from "react-dom/client";
import { platform } from "@tauri-apps/plugin-os";
import App from "./App";
import { normalizeAccent } from "./lib/constants/accents";
import { installCompatShims } from "./lib/compat";
import {
  applyTheme,
  getStoredTheme,
  syncThemeFromSettings,
} from "./lib/utils/theme";

installCompatShims();

// Set platform before render so CSS can scope per-platform (e.g. scrollbar styles)
document.documentElement.dataset.platform = platform();

// Apply the last-known theme synchronously before render to avoid a flash of
// the wrong palette, then reconcile with the persisted setting once it loads.
applyTheme(getStoredTheme());
try {
  document.documentElement.dataset.accent = normalizeAccent(
    localStorage.getItem("murmur-accent") ??
      localStorage.getItem("handy-accent"),
  );
} catch {
  document.documentElement.dataset.accent = "default";
}
syncThemeFromSettings();

// Initialize i18n
import "./i18n";

// Initialize model store (loads models and sets up event listeners)
import { useModelStore } from "./stores/modelStore";
import { useNoticeStore } from "./stores/noticeStore";
import { useTranscriptionJobsStore } from "./stores/transcriptionJobsStore";
useModelStore.getState().initialize();
useTranscriptionJobsStore.getState().initialize();
useNoticeStore.getState().initialize();

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
