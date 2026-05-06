import { LazyStore } from "@tauri-apps/plugin-store";
import { get, writable } from "svelte/store";

export type KeyMap = Record<string, string>; // commandId -> chord like "Ctrl+S"

export const DEFAULT_KEYMAP: KeyMap = {
  "palette.open": "Ctrl+P",
  "settings.open": "Ctrl+,",
  "file.save": "Ctrl+S",
  "file.close": "Ctrl+W",
  "view.toggleSidebar": "Ctrl+B",
  "view.toggleTerminal": "Ctrl+`",
  "view.files": "Ctrl+1",
  "view.git": "Ctrl+2",
  "view.processes": "Ctrl+3",
  "settings.zoomIn": "Ctrl+=",
  "settings.zoomOut": "Ctrl+-",
  "settings.zoomReset": "Ctrl+0",
  "settings.editorBigger": "Ctrl+]",
  "settings.editorSmaller": "Ctrl+[",
  "action.openFolder": "Ctrl+O",
};

const store = new LazyStore("keybindings.json");
const KEY = "keymap";

export const keymap = writable<KeyMap>({ ...DEFAULT_KEYMAP });

export async function loadKeymap(): Promise<void> {
  try {
    const persisted = await store.get<KeyMap>(KEY);
    if (persisted) keymap.set({ ...DEFAULT_KEYMAP, ...persisted });
  } catch {
    // use defaults
  }
}

export async function setBinding(
  commandId: string,
  chord: string | null,
): Promise<void> {
  keymap.update((m) => {
    const next = { ...m };
    if (chord === null) delete next[commandId];
    else next[commandId] = chord;
    return next;
  });
  try {
    await store.set(KEY, get(keymap));
    await store.save();
  } catch {
    // session-only
  }
}

export async function resetKeymap(): Promise<void> {
  keymap.set({ ...DEFAULT_KEYMAP });
  try {
    await store.set(KEY, { ...DEFAULT_KEYMAP });
    await store.save();
  } catch {
    // session-only
  }
}

const MODIFIER_KEYS = new Set(["Control", "Meta", "Shift", "Alt", "Hyper", "Super"]);

/** Convert a KeyboardEvent into a canonical chord string ("Ctrl+Shift+P"). */
export function eventToChord(e: KeyboardEvent): string | null {
  if (MODIFIER_KEYS.has(e.key)) return null;
  const parts: string[] = [];
  if (e.ctrlKey || e.metaKey) parts.push("Ctrl");
  if (e.altKey) parts.push("Alt");
  if (e.shiftKey) parts.push("Shift");
  let key = e.key;
  if (key === " ") key = "Space";
  else if (key === "Escape") key = "Esc";
  else if (key === "ArrowUp") key = "Up";
  else if (key === "ArrowDown") key = "Down";
  else if (key === "ArrowLeft") key = "Left";
  else if (key === "ArrowRight") key = "Right";
  else if (key.length === 1) key = key.toUpperCase();
  parts.push(key);
  return parts.join("+");
}

/** Find the command whose chord matches the given chord, if any. */
export function findCommandForChord(
  chord: string,
  map: KeyMap,
): string | null {
  for (const [cid, k] of Object.entries(map)) {
    if (k === chord) return cid;
  }
  return null;
}

/** Detect whether the keyboard event originated inside an editable element. */
export function isEditableTarget(target: EventTarget | null): boolean {
  const el = target as HTMLElement | null;
  if (!el) return false;
  if (el.isContentEditable) return true;
  const tag = el.tagName;
  return tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT";
}
