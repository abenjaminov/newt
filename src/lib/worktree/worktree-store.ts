import { invoke } from "@tauri-apps/api/core";
import { writable, derived, get } from "svelte/store";

export type Worktree = {
  path: string;
  branch: string | null;
  head: string | null;
  bare: boolean;
  detached: boolean;
  locked: boolean;
  dirty: boolean;
  ahead: number;
  behind: number;
};

type State = {
  worktrees: Worktree[];
  activePath: string | null;
};

const state = writable<State>({ worktrees: [], activePath: null });

export const worktrees = derived(state, ($s) => $s.worktrees);
export const activeWorktreePath = derived(state, ($s) => $s.activePath);

export async function refreshWorktrees(repoPath: string | null) {
  if (!repoPath) {
    state.set({ worktrees: [], activePath: null });
    return;
  }
  try {
    const list = await invoke<Worktree[]>("list_worktrees", { path: repoPath });
    state.update((s) => ({
      worktrees: list,
      activePath: s.activePath ?? matchActive(list, repoPath),
    }));
  } catch {
    state.set({ worktrees: [], activePath: null });
  }
}

function matchActive(list: Worktree[], path: string): string | null {
  // The active path should be the worktree whose path the workspace was opened at.
  // Worktree paths from git may use forward or back slashes; normalize.
  const norm = (p: string) => p.replace(/\\/g, "/").replace(/\/$/, "").toLowerCase();
  const target = norm(path);
  return list.find((w) => norm(w.path) === target)?.path ?? list[0]?.path ?? null;
}

export function setActiveWorktree(path: string) {
  state.update((s) => ({ ...s, activePath: path }));
}

export function getActiveWorktree(): Worktree | null {
  const s = get(state);
  return s.worktrees.find((w) => w.path === s.activePath) ?? null;
}

export async function pathExists(path: string): Promise<boolean> {
  return await invoke<boolean>("path_exists", { path });
}

export async function createWorktree(opts: {
  repo: string;
  newPath: string;
  branch: string;
  createNew: boolean;
}): Promise<void> {
  await invoke("add_worktree", {
    repo: opts.repo,
    newPath: opts.newPath,
    branch: opts.branch,
    createNew: opts.createNew,
  });
  await refreshWorktrees(opts.repo);
}

export async function removeWorktree(repo: string, path: string, force: boolean) {
  await invoke("remove_worktree", { repo, path, force });
  await refreshWorktrees(repo);
}

// Path helpers — preserve OS-style separators.
export function joinPath(base: string, rel: string): string {
  if (!rel) return base;
  const sep = base.includes("\\") && !base.includes("/") ? "\\" : "/";
  const trimmed = base.replace(/[\\/]$/, "");
  const cleanRel = rel.replace(/^[\\/]+/, "").replace(/[\\/]+/g, sep);
  return `${trimmed}${sep}${cleanRel}`;
}

export function relativeTo(base: string, full: string): string | null {
  const norm = (p: string) => p.replace(/\\/g, "/");
  const nb = norm(base).replace(/\/$/, "");
  const nf = norm(full);
  if (!nf.startsWith(nb + "/") && nf !== nb) return null;
  return nf.slice(nb.length + 1);
}
