<script lang="ts">
  import { onDestroy } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { workspace } from "../workspace/workspace-store";
  import { invoke } from "@tauri-apps/api/core";
  import {
    gitStatus,
    gitError,
    gitLoading,
    gitLoaded,
    refreshGit,
    stage,
    unstage,
    discard,
    commit,
    groupEntries,
    statusColor,
    statusLabel,
    type GitStatusEntry,
  } from "./git-store";

  import { tabs } from "../editor/tabs-store";

  function joinPath(repo: string, rel: string): string {
    const sep = repo.includes("\\") && !repo.includes("/") ? "\\" : "/";
    return `${repo.replace(/[\\/]+$/, "")}${sep}${rel.replace(/\//g, sep)}`;
  }

  let message = $state("");
  let busy = $state(false);
  let selectedPath = $state<string | null>(null);

  const groups = $derived(groupEntries($gitStatus.entries));
  const repo = $derived($workspace?.rootPath ?? null);

  let lastRepo: string | null = null;
  $effect(() => {
    if (repo !== lastRepo) {
      lastRepo = repo;
      void refreshGit(repo);
      selectedPath = null;
    }
  });

  // Refresh on filesystem changes (debounced via watcher).
  let unlisten: UnlistenFn | undefined;
  (async () => {
    unlisten = await listen("fs:changed", () => {
      if (repo) void refreshGit(repo);
    });
  })();
  onDestroy(() => unlisten?.());

  function pick(entry: GitStatusEntry, kind: "staged" | "modified" | "untracked") {
    if (!repo) return;
    selectedPath = `${kind}:${entry.path}`;
    tabs.openDiff({
      repo,
      relPath: entry.path,
      absPath: joinPath(repo, entry.path),
      isUntracked: kind === "untracked",
    });
  }

  async function withBusy(fn: () => Promise<unknown>) {
    busy = true;
    try {
      await fn();
    } catch (e) {
      gitError.set(String(e));
    } finally {
      busy = false;
    }
  }

  function stageOne(e: GitStatusEntry, ev: MouseEvent) {
    ev.stopPropagation();
    if (!repo) return;
    void withBusy(() => stage(repo, [e.path]));
  }
  function unstageOne(e: GitStatusEntry, ev: MouseEvent) {
    ev.stopPropagation();
    if (!repo) return;
    void withBusy(() => unstage(repo, [e.path]));
  }
  function discardOne(e: GitStatusEntry, ev: MouseEvent) {
    ev.stopPropagation();
    if (!repo) return;
    if (!window.confirm(`Discard changes to ${e.path}? This cannot be undone.`)) return;
    void withBusy(() => discard(repo, [e.path]));
  }

  function stageAll() {
    if (!repo) return;
    const paths = [
      ...groups.modified.map((e) => e.path),
      ...groups.untracked.map((e) => e.path),
    ];
    if (paths.length === 0) return;
    void withBusy(() => stage(repo, paths));
  }
  function unstageAll() {
    if (!repo) return;
    const paths = groups.staged.map((e) => e.path);
    if (paths.length === 0) return;
    void withBusy(() => unstage(repo, paths));
  }

  async function doCommit() {
    if (!repo) return;
    if (!message.trim()) return;
    await withBusy(async () => {
      await commit(repo, message);
      message = "";
    });
  }

  const lockError = $derived(
    $gitError !== null &&
      ($gitError.includes("index.lock") ||
        $gitError.includes("Unable to create") ||
        $gitError.includes("another git process")),
  );

  async function clearIndexLock() {
    if (!repo) return;
    try {
      const removed = await invoke<boolean>("git_clear_index_lock", { repo });
      if (removed) {
        gitError.set(null);
        await refreshGit(repo);
      } else {
        gitError.set("No stale lock file found.");
      }
    } catch (e) {
      gitError.set(String(e));
    }
  }
</script>

<div class="git-panel">
  {#if !$gitLoaded}
    <div class="loading">
      <span class="big-spinner"></span>
      <span>Checking git status…</span>
    </div>
  {:else if !$gitStatus.is_repo}
    <div class="muted">Not a git repository.</div>
  {:else}
    <div class="branch-bar">
      <span class="branch">⎇ {$gitStatus.branch ?? "(detached)"}</span>
      {#if $gitStatus.ahead || $gitStatus.behind}
        <span class="ab">
          {#if $gitStatus.ahead}↑{$gitStatus.ahead}{/if}
          {#if $gitStatus.behind}↓{$gitStatus.behind}{/if}
        </span>
      {/if}
      {#if $gitLoading}
        <span class="spinner" title="Refreshing…"></span>
      {/if}
    </div>

    <div class="commit-box">
      <textarea
        rows="2"
        placeholder="Commit message…"
        bind:value={message}
        disabled={busy || groups.staged.length === 0}
      ></textarea>
      <div class="commit-actions">
        <button
          class="primary"
          disabled={busy || !message.trim() || groups.staged.length === 0}
          onclick={doCommit}
        >
          Commit ({groups.staged.length})
        </button>
      </div>
    </div>

    {#if $gitError}
      <div class="error">
        <div>{$gitError}</div>
        {#if lockError}
          <button class="lock-btn" onclick={clearIndexLock}>
            Clear stale .git/index.lock
          </button>
        {/if}
      </div>
    {/if}

    {#if groups.staged.length > 0}
      <div class="group">
        <div class="group-head">
          <span>Staged · {groups.staged.length}</span>
          <button class="link" onclick={unstageAll} disabled={busy}>Unstage all</button>
        </div>
        {#each groups.staged as e (e.path)}
          <div
            class="row"
            class:active={selectedPath === `staged:${e.path}`}
          >
            <button class="row-main" onclick={() => pick(e, "staged")} title={e.path}>
              <span class="status" style:color={statusColor(e.index_status)}>{e.index_status}</span>
              <span class="name">{e.path}</span>
            </button>
            <button class="action" title="Unstage" onclick={(ev) => unstageOne(e, ev)}>−</button>
          </div>
        {/each}
      </div>
    {/if}

    {#if groups.modified.length > 0}
      <div class="group">
        <div class="group-head">
          <span>Changes · {groups.modified.length}</span>
          <button class="link" onclick={stageAll} disabled={busy}>Stage all</button>
        </div>
        {#each groups.modified as e (e.path)}
          <div
            class="row"
            class:active={selectedPath === `modified:${e.path}`}
          >
            <button class="row-main" onclick={() => pick(e, "modified")} title={e.path}>
              <span class="status" style:color={statusColor(e.worktree_status)}
                >{e.worktree_status}</span
              >
              <span class="name">{e.path}</span>
            </button>
            <button class="action" title="Discard" onclick={(ev) => discardOne(e, ev)}>↺</button>
            <button class="action" title="Stage" onclick={(ev) => stageOne(e, ev)}>+</button>
          </div>
        {/each}
      </div>
    {/if}

    {#if groups.untracked.length > 0}
      <div class="group">
        <div class="group-head">
          <span>Untracked · {groups.untracked.length}</span>
        </div>
        {#each groups.untracked as e (e.path)}
          <div
            class="row"
            class:active={selectedPath === `untracked:${e.path}`}
          >
            <button class="row-main" onclick={() => pick(e, "untracked")} title={e.path}>
              <span class="status untracked">U</span>
              <span class="name">{e.path}</span>
            </button>
            <button class="action" title="Stage" onclick={(ev) => stageOne(e, ev)}>+</button>
          </div>
        {/each}
      </div>
    {/if}

    {#if groups.staged.length + groups.modified.length + groups.untracked.length === 0}
      <div class="muted">Working tree clean.</div>
    {/if}

    <!-- silence unused-warning for statusLabel until tooltips wire up -->
    <div class="hidden">{statusLabel("M")}</div>
  {/if}
</div>

<style>
  .git-panel {
    height: 100%;
    overflow: auto;
    padding: 4px 0;
    user-select: none;
    display: flex;
    flex-direction: column;
  }
  .branch-bar {
    padding: 6px 12px;
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--fg-dim);
    font-size: 12px;
    border-bottom: 1px solid var(--border);
  }
  .branch {
    font-family: var(--font-mono);
    color: var(--fg);
  }
  .ab {
    color: var(--fg-faint);
    font-size: 11px;
  }
  .spinner {
    width: 10px;
    height: 10px;
    border: 1.5px solid var(--border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
    margin-left: auto;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
  .commit-box {
    padding: 8px;
    border-bottom: 1px solid var(--border);
    background: var(--bg);
  }
  .commit-box textarea {
    width: 100%;
    resize: vertical;
    font-size: 12px;
    background: var(--bg-3);
    color: var(--fg);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 6px 8px;
  }
  .commit-box textarea:focus {
    border-color: var(--accent);
    outline: none;
  }
  .commit-actions {
    margin-top: 6px;
    display: flex;
    justify-content: flex-end;
  }
  .primary {
    background: var(--accent);
    color: #0e0f12;
    padding: 5px 10px;
    border-radius: 4px;
    font-weight: 500;
    font-size: 11px;
  }
  .primary:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .primary:not(:disabled):hover {
    filter: brightness(1.1);
  }
  .group {
    margin-top: 4px;
  }
  .group-head {
    padding: 4px 12px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    color: var(--fg-faint);
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    font-weight: 600;
  }
  .link {
    color: var(--fg-faint);
    font-size: 10px;
    text-transform: none;
    letter-spacing: 0;
  }
  .link:hover:not(:disabled) {
    color: var(--accent);
  }
  .row {
    display: flex;
    align-items: center;
    height: 22px;
    padding-right: 4px;
    color: var(--fg-dim);
    font-size: 12px;
  }
  .row:hover {
    background: var(--bg-hover);
  }
  .row.active {
    background: var(--bg-3);
    color: var(--fg);
  }
  .row-main {
    display: flex;
    align-items: center;
    gap: 6px;
    flex: 1;
    height: 100%;
    padding: 0 6px 0 12px;
    text-align: left;
    overflow: hidden;
  }
  .status {
    font-family: var(--font-mono);
    font-weight: 600;
    width: 14px;
    text-align: center;
  }
  .status.untracked {
    color: var(--fg-faint);
  }
  .name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .action {
    width: 18px;
    height: 18px;
    border-radius: 3px;
    color: var(--fg-faint);
    font-size: 12px;
    line-height: 1;
    opacity: 0;
  }
  .row:hover .action {
    opacity: 1;
  }
  .action:hover {
    background: var(--bg-2);
    color: var(--fg);
  }
  .muted {
    padding: 12px;
    color: var(--fg-faint);
    font-size: 12px;
  }
  .loading {
    padding: 24px 12px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    color: var(--fg-faint);
    font-size: 12px;
  }
  .big-spinner {
    width: 12px;
    height: 12px;
    border: 1.5px solid var(--border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
    flex-shrink: 0;
  }
  .lock-btn {
    margin-top: 8px;
    padding: 5px 10px;
    border-radius: 4px;
    background: var(--bg-3);
    border: 1px solid var(--border);
    color: var(--accent);
    font-size: 11px;
  }
  .lock-btn:hover {
    background: var(--bg-hover);
    color: var(--fg);
  }
  .error {
    padding: 8px 12px;
    color: var(--red);
    font-size: 12px;
    white-space: pre-wrap;
    background: rgba(247, 118, 142, 0.08);
    border-bottom: 1px solid var(--border);
  }
  .hidden {
    display: none;
  }
</style>
