<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { createWorktree } from "./worktree-store";

  type Props = {
    repo: string;
    onClose: () => void;
    onCreated: (path: string) => void;
  };

  let { repo, onClose, onCreated }: Props = $props();

  let mode = $state<"existing" | "new">("new");
  let branchName = $state("");
  let baseBranch = $state("HEAD");
  let path = $state("");
  let busy = $state(false);
  let error = $state<string | null>(null);
  let localBranches = $state<string[]>([]);

  function defaultPath(name: string): string {
    if (!name) return "";
    const sep = repo.includes("\\") && !repo.includes("/") ? "\\" : "/";
    const parts = repo.split(/[\\/]/).filter(Boolean);
    parts.pop(); // drop the repo dir
    const repoBase = repo.split(/[\\/]/).filter(Boolean).pop() ?? "repo";
    const safe = name.replace(/[\\/]/g, "-");
    return [...parts, `${repoBase}-${safe}`].join(sep);
  }

  $effect(() => {
    if (!path && branchName) {
      path = defaultPath(branchName);
    }
  });

  onMount(async () => {
    try {
      const r = await invoke<{ current: string | null; local: string[]; remote: string[] }>(
        "git_branches",
        { repo },
      );
      localBranches = r.local;
    } catch {
      localBranches = [];
    }
  });

  async function submit() {
    if (!branchName.trim() || !path.trim()) return;
    busy = true;
    error = null;
    try {
      await createWorktree({
        repo,
        newPath: path,
        branch: mode === "new" ? branchName : branchName,
        createNew: mode === "new",
      });
      onCreated(path);
      onClose();
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }
</script>

<div class="backdrop" onclick={onClose} role="presentation"></div>
<div class="modal" role="dialog" aria-modal="true" aria-label="Create worktree">
  <h2>New worktree</h2>

  <div class="row">
    <label>
      <input type="radio" bind:group={mode} value="new" /> Create new branch
    </label>
    <label>
      <input type="radio" bind:group={mode} value="existing" /> Use existing branch
    </label>
  </div>

  {#if mode === "new"}
    <label class="field">
      <span>New branch name</span>
      <input
        type="text"
        bind:value={branchName}
        placeholder="feature/something"
      />
    </label>
    <label class="field">
      <span>Based on</span>
      <input type="text" bind:value={baseBranch} placeholder="HEAD" disabled />
    </label>
  {:else}
    <label class="field">
      <span>Branch</span>
      <select bind:value={branchName}>
        <option value="">Select…</option>
        {#each localBranches as b}
          <option value={b}>{b}</option>
        {/each}
      </select>
    </label>
  {/if}

  <label class="field">
    <span>Worktree path</span>
    <input type="text" bind:value={path} placeholder="C:\\path\\to\\worktree" />
  </label>

  {#if error}
    <div class="error">{error}</div>
  {/if}

  <div class="actions">
    <button onclick={onClose} disabled={busy}>Cancel</button>
    <button class="primary" onclick={submit} disabled={busy || !branchName.trim() || !path.trim()}>
      {busy ? "Creating…" : "Create worktree"}
    </button>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    z-index: 200;
  }
  .modal {
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    background: var(--bg-2);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 20px 24px;
    width: min(480px, 90vw);
    z-index: 201;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
  }
  h2 {
    margin: 0 0 16px;
    font-size: 16px;
    font-weight: 600;
    color: var(--fg);
  }
  .row {
    display: flex;
    gap: 14px;
    margin-bottom: 14px;
    color: var(--fg-dim);
    font-size: 12px;
  }
  .row label {
    display: flex;
    gap: 6px;
    align-items: center;
    cursor: pointer;
  }
  .field {
    display: block;
    margin-bottom: 12px;
  }
  .field span {
    display: block;
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--fg-faint);
    margin-bottom: 4px;
    font-weight: 600;
  }
  .field input,
  .field select {
    width: 100%;
    font-size: 13px;
  }
  .field input:disabled {
    opacity: 0.6;
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 18px;
  }
  .actions button {
    padding: 6px 14px;
    border-radius: 4px;
    color: var(--fg-dim);
    font-size: 12px;
  }
  .actions button:hover:not(:disabled) {
    background: var(--bg-hover);
    color: var(--fg);
  }
  .primary {
    background: var(--accent) !important;
    color: #0e0f12 !important;
  }
  .primary:hover:not(:disabled) {
    filter: brightness(1.08);
    background: var(--accent) !important;
  }
  .primary:disabled,
  .actions button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .error {
    color: var(--red);
    font-size: 12px;
    margin-top: 8px;
    padding: 8px 10px;
    background: rgba(247, 118, 142, 0.1);
    border-radius: 4px;
    white-space: pre-wrap;
  }
</style>
