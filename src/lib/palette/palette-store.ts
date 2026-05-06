import { invoke } from "@tauri-apps/api/core";
import { writable } from "svelte/store";

export type FileEntry = {
  path: string;
  rel: string;
  name: string;
};

export const paletteOpen = writable(false);
export const fileIndex = writable<FileEntry[]>([]);

export async function refreshFileIndex(root: string | null) {
  if (!root) {
    fileIndex.set([]);
    return;
  }
  try {
    const list = await invoke<FileEntry[]>("list_files", { root });
    fileIndex.set(list);
  } catch {
    fileIndex.set([]);
  }
}

export type Command = {
  id: string;
  title: string;
  group: "Action" | "View" | "File" | "Settings" | "Worktree" | "Terminal" | "Git";
  hint?: string; // keyboard shortcut hint
  run: () => unknown | Promise<unknown>;
};

export const commandRegistry = writable<Command[]>([]);
