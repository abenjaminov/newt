import { invoke } from "@tauri-apps/api/core";
import { derived, writable } from "svelte/store";

export type GitStatusEntry = {
  path: string;
  orig_path: string | null;
  index_status: string;
  worktree_status: string;
};

export type GitStatus = {
  is_repo: boolean;
  branch: string | null;
  upstream: string | null;
  ahead: number;
  behind: number;
  entries: GitStatusEntry[];
};

const empty: GitStatus = {
  is_repo: false,
  branch: null,
  upstream: null,
  ahead: 0,
  behind: 0,
  entries: [],
};

export const gitStatus = writable<GitStatus>(empty);
export const gitError = writable<string | null>(null);
export const gitLoading = writable(false);
/** True once the first git_status call for the current workspace has completed
 * (success or error). Used by the panel to distinguish "haven't checked yet"
 * from "checked and it isn't a repo". */
export const gitLoaded = writable(false);

// Set of repo-relative paths that are ignored by .gitignore. Refreshed lazily
// on workspace/worktree change (not on every fs:changed event) because
// `git ls-files --others --ignored` can be expensive on large repos.
export const ignoredPaths = writable<Set<string>>(new Set());

/**
 * Pre-computed set of relative paths that should be visually dimmed in the
 * file tree — combines untracked entries from the live status with the
 * cached ignored paths. Tree items do an O(1) Set lookup instead of an
 * O(N) entry scan.
 */
export const dimmedPaths = derived(
  [gitStatus, ignoredPaths],
  ([$s, $ignored]) => {
    const set = new Set<string>($ignored);
    for (const e of $s.entries) {
      const isUntracked = e.index_status === "?" && e.worktree_status === "?";
      if (!isUntracked) continue;
      set.add(e.path.replace(/\\/g, "/").replace(/\/$/, ""));
    }
    return set;
  },
);

export async function refreshIgnored(repo: string | null): Promise<void> {
  if (!repo) {
    ignoredPaths.set(new Set());
    return;
  }
  try {
    const list = await invoke<string[]>("git_ignored_paths", { repo });
    ignoredPaths.set(new Set(list.map((p) => p.replace(/\\/g, "/").replace(/\/$/, ""))));
  } catch {
    // ignored — keep previous set
  }
}

let activeRepo: string | null = null;
let inflight = false;
let pendingRepo: string | null = null;

export async function refreshGit(repo: string | null) {
  // Reset "loaded" indicator when switching workspace/worktree.
  if (activeRepo !== repo) {
    gitLoaded.set(false);
  }
  activeRepo = repo;
  if (!repo) {
    gitStatus.set(empty);
    pendingRepo = null;
    return;
  }
  // Coalesce concurrent calls. If a refresh is already running, just remember
  // the most recent repo and let the in-flight loop pick it up.
  if (inflight) {
    pendingRepo = repo;
    return;
  }
  inflight = true;
  gitLoading.set(true);
  try {
    let target = repo;
    while (true) {
      try {
        const s = await invoke<GitStatus>("git_status", { repo: target });
        if (activeRepo === target) {
          gitStatus.set(s);
          gitError.set(null);
          gitLoaded.set(true);
        }
      } catch (e) {
        gitError.set(String(e));
        gitLoaded.set(true); // even on error, the attempt is done
      }
      if (pendingRepo === null) break;
      target = pendingRepo;
      pendingRepo = null;
    }
  } finally {
    inflight = false;
    gitLoading.set(false);
  }
}

async function refreshIfActive(repo: string) {
  if (activeRepo === repo) await refreshGit(repo);
}

export async function stage(repo: string, paths: string[]) {
  await invoke("git_stage", { repo, paths });
  await refreshIfActive(repo);
}

export async function unstage(repo: string, paths: string[]) {
  await invoke("git_unstage", { repo, paths });
  await refreshIfActive(repo);
}

export async function discard(repo: string, paths: string[]) {
  await invoke("git_discard", { repo, paths });
  await refreshIfActive(repo);
}

export async function commit(repo: string, message: string) {
  await invoke("git_commit", { repo, message });
  await refreshIfActive(repo);
}

export async function fileAtHead(repo: string, path: string): Promise<string> {
  return await invoke<string>("git_file_at_head", { repo, path });
}

// Convenience splits: which entries are "staged" vs "modified" vs "untracked"
export type GitGroups = {
  staged: GitStatusEntry[];
  modified: GitStatusEntry[];
  untracked: GitStatusEntry[];
};

export function groupEntries(entries: GitStatusEntry[]): GitGroups {
  const staged: GitStatusEntry[] = [];
  const modified: GitStatusEntry[] = [];
  const untracked: GitStatusEntry[] = [];
  for (const e of entries) {
    // Ignored entries ("!") are surfaced for dimming the tree only;
    // skip them in the panel groups.
    if (e.index_status === "!" || e.worktree_status === "!") continue;
    if (e.index_status === "?" && e.worktree_status === "?") {
      untracked.push(e);
      continue;
    }
    if (e.index_status !== "." && e.index_status !== " ") {
      staged.push(e);
    }
    if (e.worktree_status !== "." && e.worktree_status !== " ") {
      modified.push(e);
    }
  }
  return { staged, modified, untracked };
}

export function statusLabel(s: string): string {
  switch (s) {
    case "M":
      return "modified";
    case "A":
      return "added";
    case "D":
      return "deleted";
    case "R":
      return "renamed";
    case "C":
      return "copied";
    case "U":
      return "conflict";
    case "?":
      return "untracked";
    default:
      return s;
  }
}

export function statusColor(s: string): string {
  switch (s) {
    case "M":
      return "var(--yellow)";
    case "A":
      return "var(--green)";
    case "D":
      return "var(--red)";
    case "R":
    case "C":
      return "var(--accent-2)";
    case "U":
      return "var(--red)";
    case "?":
      return "var(--fg-faint)";
    default:
      return "var(--fg-dim)";
  }
}
