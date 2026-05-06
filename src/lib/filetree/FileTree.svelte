<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { workspace } from "../workspace/workspace-store";
  import { openFileAtPath } from "../editor/open-file";
  import { treeStore, type DirEntry, type TreeNode } from "./tree-store.svelte";
  import TreeItem from "./TreeItem.svelte";

  let loading = $state(false);
  let error = $state<string | null>(null);

  async function loadRoot() {
    const ws = $workspace;
    if (!ws) return;
    loading = true;
    error = null;
    try {
      const entries = await invoke<DirEntry[]>("read_dir", { path: ws.rootPath });
      treeStore.root = {
        name: ws.rootName,
        path: ws.rootPath,
        is_dir: true,
        expanded: true,
        loaded: true,
        children: entries.map((e) => ({ ...e, expanded: false, loaded: false })),
      };
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function openFile(node: TreeNode) {
    const r = await openFileAtPath(node.path);
    if (!r.ok) error = r.error;
  }

  onMount(loadRoot);

  workspace.subscribe(() => {
    if ($workspace) loadRoot();
    else treeStore.root = null;
  });
</script>

<div class="tree">
  {#if loading}
    <div class="muted">Loading…</div>
  {:else if error}
    <div class="error">{error}</div>
  {:else if treeStore.root}
    {#each treeStore.root.children ?? [] as child (child.path)}
      <TreeItem node={child} depth={0} onOpen={openFile} />
    {/each}
  {/if}
</div>

<style>
  .tree {
    padding: 4px 0;
    overflow: auto;
    height: 100%;
    user-select: none;
  }
  .muted {
    padding: 8px 12px;
    color: var(--fg-faint);
    font-size: 12px;
  }
  .error {
    padding: 8px 12px;
    color: var(--red);
    font-size: 12px;
    white-space: pre-wrap;
  }
</style>
