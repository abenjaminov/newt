import { LazyStore } from "@tauri-apps/plugin-store";
import { writable } from "svelte/store";

export type Settings = {
  uiZoom: number; // 0.5 .. 2.0, applies to the entire window via CSS zoom
  uiFontFamily: string;
  uiFontSize: number;
  editorFontFamily: string;
  editorFontSize: number;
  editorTabSize: number;
  editorLineWrap: boolean;
  editorAutoSave: "off" | "afterDelay" | "onFocusChange";
  editorAutoSaveDelayMs: number;
  terminalFontFamily: string;
  terminalFontSize: number;
  worktreeForeignTabs: "mark" | "close" | "keepActive";
};

export const DEFAULTS: Settings = {
  uiZoom: 1,
  uiFontFamily:
    '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Oxygen, Ubuntu, sans-serif',
  uiFontSize: 13,
  editorFontFamily:
    '"JetBrains Mono", "Fira Code", ui-monospace, SFMono-Regular, Menlo, Consolas, monospace',
  editorFontSize: 13,
  editorTabSize: 2,
  editorLineWrap: true,
  editorAutoSave: "off",
  editorAutoSaveDelayMs: 250,
  terminalFontFamily:
    '"JetBrains Mono", "Fira Code", Menlo, Consolas, monospace',
  terminalFontSize: 13,
  worktreeForeignTabs: "mark",
};

export const FONT_SUGGESTIONS_MONO = [
  '"JetBrains Mono"',
  '"Fira Code"',
  '"Cascadia Code"',
  '"Source Code Pro"',
  '"IBM Plex Mono"',
  '"Hack"',
  '"Inconsolata"',
  '"Iosevka"',
  '"Menlo"',
  '"Consolas"',
  "ui-monospace",
];

export const FONT_SUGGESTIONS_UI = [
  '"Inter"',
  '"SF Pro Text"',
  '"Segoe UI"',
  '"Roboto"',
  '"Helvetica Neue"',
  "system-ui",
  "-apple-system",
];

const store = new LazyStore("settings.json");
const KEY = "settings";

export const settings = writable<Settings>(DEFAULTS);

function applyCssVars(s: Settings) {
  const r = document.documentElement.style;
  r.setProperty("--font-mono", s.editorFontFamily);
  r.setProperty("--font-ui", s.uiFontFamily);
  r.setProperty("--editor-font-size", `${s.editorFontSize}px`);
  r.setProperty("--ui-font-size", `${s.uiFontSize}px`);
  r.setProperty("--terminal-font-family", s.terminalFontFamily);
  r.setProperty("--terminal-font-size", `${s.terminalFontSize}px`);
  // CSS `zoom` scales the whole webview proportionally. Chromium-based webviews
  // (WebView2 on Windows, WKWebView on macOS) all support it.
  document.body.style.zoom = String(s.uiZoom);
}

export async function loadSettings(): Promise<void> {
  try {
    const persisted = (await store.get<Partial<Settings>>(KEY)) ?? {};
    const merged: Settings = { ...DEFAULTS, ...persisted };
    settings.set(merged);
    applyCssVars(merged);
  } catch {
    applyCssVars(DEFAULTS);
  }
}

export async function updateSettings(patch: Partial<Settings>): Promise<void> {
  let next: Settings = DEFAULTS;
  settings.update((cur) => {
    next = { ...cur, ...patch };
    return next;
  });
  applyCssVars(next);
  try {
    await store.set(KEY, next);
    await store.save();
  } catch {
    // ignored — apply still wins for the session
  }
}

export async function resetSettings(): Promise<void> {
  settings.set(DEFAULTS);
  applyCssVars(DEFAULTS);
  try {
    await store.set(KEY, DEFAULTS);
    await store.save();
  } catch {
    // ignored
  }
}
