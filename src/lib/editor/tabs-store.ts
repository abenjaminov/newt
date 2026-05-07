import { writable, derived } from "svelte/store";
import type { FileKind } from "./languages";

export type TabKind = FileKind | "diff" | "settings";

export const SETTINGS_TAB_PATH = "settings://app";

export type Tab = {
  path: string;             // unique tab key. For diffs, prefixed with "diff://"
  name: string;
  kind: TabKind;
  content: string;          // text content (svg/text only)
  saved: string;
  dirty: boolean;
  foreignWorktree: string | null;
  imageDataUrl: string | null;
  imageSize: number | null; // bytes
  viewMode: "text" | "image";
  diffMeta: { repo: string; absPath: string; relPath: string; isUntracked: boolean } | null;
};

type TabsState = {
  tabs: Tab[];
  activePath: string | null;
};

const RECENTLY_CLOSED_MAX = 20;
const recentlyClosed: string[] = [];

function pushRecentlyClosed(path: string) {
  if (path.startsWith("diff://")) return; // diff tabs aren't reopenable
  if (path.startsWith("settings://")) return;
  const idx = recentlyClosed.indexOf(path);
  if (idx !== -1) recentlyClosed.splice(idx, 1);
  recentlyClosed.push(path);
  if (recentlyClosed.length > RECENTLY_CLOSED_MAX) recentlyClosed.shift();
}

export function popRecentlyClosed(): string | null {
  return recentlyClosed.pop() ?? null;
}

function basename(p: string): string {
  return p.split(/[\\/]/).filter(Boolean).pop() ?? p;
}

export type OpenSpec = {
  path: string;
  kind: FileKind;
  content: string;
  imageDataUrl?: string | null;
  imageSize?: number | null;
};

export type OpenDiffSpec = {
  repo: string;
  absPath: string;
  relPath: string;
  isUntracked: boolean;
};

function diffTabPath(spec: OpenDiffSpec): string {
  return `diff://${spec.repo}|${spec.relPath}`;
}

function createTabs() {
  const { subscribe, update, set } = writable<TabsState>({
    tabs: [],
    activePath: null,
  });

  return {
    subscribe,
    open(spec: OpenSpec) {
      update((s) => {
        const existing = s.tabs.find((t) => t.path === spec.path);
        if (existing) {
          return { ...s, activePath: spec.path };
        }
        const defaultViewMode = spec.kind === "text" ? "text" : "image";
        const tab: Tab = {
          path: spec.path,
          name: basename(spec.path),
          kind: spec.kind,
          content: spec.content,
          saved: spec.content,
          dirty: false,
          foreignWorktree: null,
          imageDataUrl: spec.imageDataUrl ?? null,
          imageSize: spec.imageSize ?? null,
          viewMode: defaultViewMode,
          diffMeta: null,
        };
        return { tabs: [...s.tabs, tab], activePath: spec.path };
      });
    },
    openSettings() {
      const tabPath = SETTINGS_TAB_PATH;
      update((s) => {
        const existing = s.tabs.find((t) => t.path === tabPath);
        if (existing) {
          return { ...s, activePath: tabPath };
        }
        const tab: Tab = {
          path: tabPath,
          name: "Settings",
          kind: "settings",
          content: "",
          saved: "",
          dirty: false,
          foreignWorktree: null,
          imageDataUrl: null,
          imageSize: null,
          viewMode: "text",
          diffMeta: null,
        };
        return { tabs: [...s.tabs, tab], activePath: tabPath };
      });
    },
    openDiff(spec: OpenDiffSpec) {
      const tabPath = diffTabPath(spec);
      update((s) => {
        const existing = s.tabs.find((t) => t.path === tabPath);
        if (existing) {
          return { ...s, activePath: tabPath };
        }
        const tab: Tab = {
          path: tabPath,
          name: `${basename(spec.relPath)} (diff)`,
          kind: "diff",
          content: "",
          saved: "",
          dirty: false,
          foreignWorktree: null,
          imageDataUrl: null,
          imageSize: null,
          viewMode: "text",
          diffMeta: {
            repo: spec.repo,
            absPath: spec.absPath,
            relPath: spec.relPath,
            isUntracked: spec.isUntracked,
          },
        };
        return { tabs: [...s.tabs, tab], activePath: tabPath };
      });
    },
    activate(path: string) {
      update((s) => ({ ...s, activePath: path }));
    },
    close(path: string) {
      pushRecentlyClosed(path);
      update((s) => {
        const tabs = s.tabs.filter((t) => t.path !== path);
        let activePath = s.activePath;
        if (activePath === path) {
          activePath = tabs.length > 0 ? tabs[tabs.length - 1].path : null;
        }
        return { tabs, activePath };
      });
    },
    edit(path: string, next: string) {
      update((s) => ({
        ...s,
        tabs: s.tabs.map((t) =>
          t.path === path
            ? { ...t, content: next, dirty: next !== t.saved }
            : t,
        ),
      }));
    },
    markSaved(path: string) {
      update((s) => ({
        ...s,
        tabs: s.tabs.map((t) =>
          t.path === path ? { ...t, saved: t.content, dirty: false } : t,
        ),
      }));
    },
    closeAll() {
      set({ tabs: [], activePath: null });
    },
    replace(
      oldPath: string,
      next: {
        path: string;
        kind: FileKind;
        content: string;
        imageDataUrl?: string | null;
        imageSize?: number | null;
        foreignWorktree: string | null;
      },
    ) {
      update((s) => ({
        ...s,
        activePath: s.activePath === oldPath ? next.path : s.activePath,
        tabs: s.tabs.map((t) =>
          t.path === oldPath
            ? {
                path: next.path,
                name: basename(next.path),
                kind: next.kind,
                content: next.content,
                saved: next.content,
                dirty: false,
                foreignWorktree: next.foreignWorktree,
                imageDataUrl: next.imageDataUrl ?? null,
                imageSize: next.imageSize ?? null,
                viewMode: next.kind === "text" ? "text" : "image",
                diffMeta: null,
              }
            : t,
        ),
      }));
    },
    markForeign(path: string, branch: string | null) {
      update((s) => ({
        ...s,
        tabs: s.tabs.map((t) =>
          t.path === path ? { ...t, foreignWorktree: branch } : t,
        ),
      }));
    },
    setViewMode(path: string, mode: "text" | "image") {
      update((s) => ({
        ...s,
        tabs: s.tabs.map((t) =>
          t.path === path && t.kind === "svg" ? { ...t, viewMode: mode } : t,
        ),
      }));
    },
    snapshot(): { tabs: Tab[]; activePath: string | null } {
      let snap: { tabs: Tab[]; activePath: string | null } = { tabs: [], activePath: null };
      const unsub = subscribe((s) => (snap = { tabs: [...s.tabs], activePath: s.activePath }));
      unsub();
      return snap;
    },
    restore(snap: { tabs: Tab[]; activePath: string | null }) {
      set({ tabs: snap.tabs, activePath: snap.activePath });
    },
  };
}

export const tabs = createTabs();
export const activeTab = derived(tabs, ($t) =>
  $t.activePath ? $t.tabs.find((t) => t.path === $t.activePath) ?? null : null,
);
