<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { revealItemInDir } from "@tauri-apps/plugin-opener";
  import { get } from "svelte/store";
  import { tabs } from "../editor/tabs-store";
  import ContextMenu, { type MenuItem } from "../common/ContextMenu.svelte";
  import NamePrompt from "../common/NamePrompt.svelte";
  import { fileIcon, folderIcon } from "./file-icons";
  import { dimmedPaths, gitStatus } from "../git/git-store";
  import { workspace } from "../workspace/workspace-store";
  import { treeStore, type DirEntry, type TreeNode } from "./tree-store.svelte";
  import Self from "./TreeItem.svelte";

  type Props = {
    node: TreeNode;
    depth: number;
    onOpen: (node: TreeNode) => void;
  };

  let { node, depth, onOpen }: Props = $props();

  let menu = $state<{ x: number; y: number } | null>(null);
  let renameValue = $state("");
  let renameInput: HTMLInputElement | undefined = $state();
  let dragOver = $state(false);
  let namePrompt = $state<{ isDir: boolean } | null>(null);
  let createError = $state<string | null>(null);

  function nameValidator(name: string): string | null {
    if (!name) return null;
    if (name.includes("/") || name.includes("\\"))
      return "Name can't contain / or \\";
    if (name === "." || name === "..") return "Reserved name";
    return null;
  }

  const isRenaming = $derived(treeStore.renaming === node.path);
  const isSelected = $derived(treeStore.selected === node.path);
  const icon = $derived(node.is_dir ? folderIcon(node.name) : fileIcon(node.name));

  // Dim untracked / ignored files when a git repo is open.
  // Uses a pre-computed Set from git-store, so this is O(depth) per node
  // (walking ancestor segments) rather than O(entries).
  const isDimmed = $derived.by(() => {
    if (!$gitStatus.is_repo) return false;
    const set = $dimmedPaths;
    if (set.size === 0) return false;
    const ws = $workspace;
    if (!ws) return false;
    const root = ws.rootPath.replace(/\\/g, "/").replace(/\/$/, "");
    const nodeAbs = node.path.replace(/\\/g, "/").replace(/\/$/, "");
    if (nodeAbs === root) return false;
    if (!nodeAbs.startsWith(root + "/")) return false;
    let rel = nodeAbs.slice(root.length + 1);
    // Walk up the relative path segments, checking each ancestor.
    while (rel.length > 0) {
      if (set.has(rel)) return true;
      const slash = rel.lastIndexOf("/");
      if (slash <= 0) break;
      rel = rel.slice(0, slash);
    }
    return false;
  });

  $effect(() => {
    if (isRenaming) {
      renameValue = node.name;
      // Focus + select stem on next tick
      queueMicrotask(() => {
        if (!renameInput) return;
        renameInput.focus();
        const stem = node.name.lastIndexOf(".");
        renameInput.setSelectionRange(0, stem > 0 ? stem : node.name.length);
      });
    }
  });

  async function loadChildren() {
    if (!node.is_dir || node.loaded) return;
    try {
      const entries = await invoke<DirEntry[]>("read_dir", { path: node.path });
      node.children = entries.map((e) => ({ ...e, expanded: false, loaded: false }));
      node.loaded = true;
    } catch {
      node.children = [];
      node.loaded = true;
    }
  }

  async function toggle() {
    treeStore.selected = node.path;
    if (!node.is_dir) {
      onOpen(node);
      return;
    }
    if (!node.loaded) await loadChildren();
    node.expanded = !node.expanded;
  }

  function onContextMenu(e: MouseEvent) {
    e.preventDefault();
    e.stopPropagation();
    treeStore.selected = node.path;
    menu = { x: e.clientX, y: e.clientY };
  }

  function onKeyDown(e: KeyboardEvent) {
    if (e.key === "F2") {
      e.preventDefault();
      treeStore.renaming = node.path;
    } else if (e.key === "Delete") {
      e.preventDefault();
      void confirmDelete();
    }
  }

  async function confirmDelete() {
    if (!window.confirm(`Delete ${node.is_dir ? "folder" : "file"} "${node.name}"?\n\n${node.path}`))
      return;
    try {
      await invoke("delete_path", { path: node.path });
      tabs.close(node.path);
      treeStore.removeAt(node.path);
      if (treeStore.selected === node.path) treeStore.selected = null;
    } catch (e) {
      window.alert(`Delete failed: ${e}`);
    }
  }

  async function commitRename(e?: Event) {
    e?.preventDefault();
    const next = renameValue.trim();
    if (!next || next === node.name) {
      treeStore.renaming = null;
      return;
    }
    if (next.includes("/") || next.includes("\\")) {
      window.alert("Names can't contain / or \\");
      return;
    }
    const sep = node.path.includes("\\") && !node.path.includes("/") ? "\\" : "/";
    const parent = node.path.slice(0, node.path.length - node.name.length).replace(/[\\/]+$/, "");
    const newPath = `${parent}${sep}${next}`;
    try {
      await invoke("rename_path", { from: node.path, to: newPath });
      treeStore.renameAt(node.path, next);
      treeStore.renaming = null;
      treeStore.selected = newPath;
    } catch (err) {
      window.alert(`Rename failed: ${err}`);
    }
  }

  function cancelRename() {
    treeStore.renaming = null;
  }

  function newChild(isDir: boolean) {
    if (!node.is_dir) return;
    namePrompt = { isDir };
  }

  async function commitNewChild(name: string) {
    if (!namePrompt) return;
    const isDir = namePrompt.isDir;
    const sep = node.path.includes("\\") && !node.path.includes("/") ? "\\" : "/";
    const newPath = `${node.path.replace(/[\\/]+$/, "")}${sep}${name}`;
    try {
      if (isDir) await invoke("create_dir", { path: newPath });
      else await invoke("create_file", { path: newPath });
      if (!node.loaded) await loadChildren();
      node.expanded = true;
      treeStore.insertEntry(node.path, { name, path: newPath, is_dir: isDir });
      if (!isDir) {
        const content = await invoke<string>("read_file", { path: newPath });
        tabs.open({ path: newPath, kind: "text", content });
      }
      namePrompt = null;
    } catch (e) {
      createError = String(e);
    }
  }

  function buildItems(): MenuItem[] {
    const items: MenuItem[] = [];
    if (!node.is_dir) {
      items.push({
        label: "Open",
        action: () => onOpen(node),
      });
      items.push({ separator: true });
    }
    if (node.is_dir) {
      items.push(
        { label: "New File…", action: () => newChild(false) },
        { label: "New Folder…", action: () => newChild(true) },
        { separator: true },
      );
    }
    items.push(
      {
        label: "Rename",
        action: () => (treeStore.renaming = node.path),
        hint: "F2",
      },
      {
        label: "Delete",
        action: () => confirmDelete(),
        danger: true,
        hint: "Del",
      },
      { separator: true },
      {
        label: "Reveal in File Explorer",
        action: () => revealItemInDir(node.path).catch(() => {}),
      },
      {
        label: "Copy Path",
        action: () => navigator.clipboard.writeText(node.path).catch(() => {}),
      },
    );
    return items;
  }

  // --- Drag & drop ---

  function onDragStart(e: DragEvent) {
    if (!e.dataTransfer) return;
    e.dataTransfer.setData("application/x-newt-path", node.path);
    e.dataTransfer.effectAllowed = "move";
  }

  function onDragOver(e: DragEvent) {
    if (!node.is_dir) return;
    // Some browsers / webviews don't expose custom MIME types during dragover
    // (only on drop). Accept the dragover unconditionally on directories;
    // we'll validate the payload at drop time.
    e.preventDefault();
    if (e.dataTransfer) e.dataTransfer.dropEffect = "move";
    dragOver = true;
  }

  function onDragLeave() {
    dragOver = false;
  }

  async function onDrop(e: DragEvent) {
    dragOver = false;
    if (!node.is_dir) return;
    const src = e.dataTransfer?.getData("application/x-newt-path");
    if (!src || src === node.path) return;
    e.preventDefault();
    e.stopPropagation();
    if (node.path.startsWith(src + "/") || node.path.startsWith(src + "\\")) {
      window.alert("Can't move a folder into itself.");
      return;
    }
    try {
      await invoke<string>("move_into", { src, dstDir: node.path });
      // Close any tabs whose paths were inside the moved tree (path-rewrite is
      // overkill for v1; user can reopen from the new location).
      const snap = get(tabs).tabs;
      for (const t of snap) {
        if (
          t.path === src ||
          t.path.startsWith(src + "/") ||
          t.path.startsWith(src + "\\")
        ) {
          tabs.close(t.path);
        }
      }
      if (!node.loaded) await loadChildren();
      treeStore.moveTo(src, node.path);
    } catch (err) {
      window.alert(`Move failed: ${err}`);
    }
  }
</script>

{#if !isRenaming}
  <button
    class="row"
    class:dir={node.is_dir}
    class:selected={isSelected}
    class:drag-over={dragOver}
    style:padding-left="{depth * 12 + 6}px"
    onclick={toggle}
    oncontextmenu={onContextMenu}
    onkeydown={onKeyDown}
    onfocus={() => (treeStore.selected = node.path)}
    title={node.path}
    draggable="true"
    ondragstart={onDragStart}
    ondragover={onDragOver}
    ondragleave={onDragLeave}
    ondrop={onDrop}
  >
    <span class="chev">
      {#if node.is_dir}
        {node.expanded ? "▾" : "▸"}
      {:else}
        <span class="chev-spacer"></span>
      {/if}
    </span>
    <span class="ftype" style:color={icon.color} class:dir={node.is_dir}>
      {#if node.is_dir}
        📁
      {:else}
        {icon.glyph}
      {/if}
    </span>
    <span class="name" class:dimmed={isDimmed}>{node.name}</span>
  </button>
{:else}
  <form
    class="rename-row"
    style:padding-left="{depth * 12 + 6}px"
    onsubmit={commitRename}
  >
    <span class="chev"><span class="chev-spacer"></span></span>
    <span class="ftype" style:color={icon.color} class:dir={node.is_dir}>
      {node.is_dir ? "📁" : icon.glyph}
    </span>
    <input
      bind:this={renameInput}
      bind:value={renameValue}
      onkeydown={(e) => {
        if (e.key === "Escape") {
          e.preventDefault();
          cancelRename();
        }
      }}
      onblur={() => commitRename()}
      type="text"
      spellcheck="false"
    />
  </form>
{/if}

{#if node.is_dir && node.expanded && node.children}
  {#each node.children as child (child.path)}
    <Self node={child} depth={depth + 1} {onOpen} />
  {/each}
{/if}

{#if menu}
  <ContextMenu
    x={menu.x}
    y={menu.y}
    items={buildItems()}
    onClose={() => (menu = null)}
  />
{/if}

{#if namePrompt}
  <NamePrompt
    title={namePrompt.isDir ? "New folder" : "New file"}
    confirmLabel="Create"
    placeholder="name"
    validate={nameValidator}
    onConfirm={commitNewChild}
    onCancel={() => {
      namePrompt = null;
      createError = null;
    }}
  />
{/if}
{#if createError}
  <div class="ti-toast" onclick={() => (createError = null)} role="presentation">
    {createError}
  </div>
{/if}

<style>
  .row {
    display: flex;
    align-items: center;
    gap: 4px;
    width: 100%;
    height: 22px;
    padding-right: 8px;
    text-align: left;
    color: var(--fg-dim);
    font-size: 12px;
    border-radius: 0;
    border: 1px solid transparent;
  }
  .row:hover {
    background: var(--bg-hover);
    color: var(--fg);
  }
  .row.selected {
    background: rgba(232, 145, 90, 0.12);
    color: var(--fg);
  }
  .row.drag-over {
    background: rgba(232, 145, 90, 0.18);
    border-color: var(--accent);
  }
  .row:focus-visible {
    outline: 1px solid var(--accent);
    outline-offset: -1px;
  }
  .chev {
    width: 12px;
    color: var(--fg-faint);
    font-size: 10px;
    text-align: center;
    flex-shrink: 0;
  }
  .chev-spacer {
    display: inline-block;
    width: 12px;
  }
  .ftype {
    font-family: var(--font-mono);
    font-size: 9px;
    font-weight: 700;
    flex-shrink: 0;
    min-width: 22px;
    text-align: center;
    letter-spacing: -0.4px;
    line-height: 1;
  }
  .ftype.dir {
    font-size: 12px;
    font-weight: normal;
    letter-spacing: 0;
  }
  .name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .name.dimmed {
    color: var(--fg-faint);
    opacity: 0.55;
  }
  .rename-row {
    display: flex;
    align-items: center;
    gap: 4px;
    height: 22px;
    padding-right: 8px;
  }
  .rename-row input {
    flex: 1;
    height: 20px;
    padding: 0 4px;
    font-size: 12px;
    border: 1px solid var(--accent);
    border-radius: 3px;
    background: var(--bg-3);
    color: var(--fg);
    outline: none;
  }
  .ti-toast {
    position: fixed;
    bottom: 24px;
    left: 50%;
    transform: translateX(-50%);
    background: var(--bg-3);
    color: var(--red);
    border: 1px solid var(--border);
    padding: 8px 14px;
    border-radius: 6px;
    font-size: 12px;
    z-index: 500;
    cursor: pointer;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
    user-select: none;
  }
</style>
