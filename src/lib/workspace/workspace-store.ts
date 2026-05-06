import { writable } from "svelte/store";

export type Workspace = {
  rootPath: string;
  rootName: string;
};

export const workspace = writable<Workspace | null>(null);

export type SidePanel = "files" | "search" | "git" | "processes";
export const activePanel = writable<SidePanel>("files");
