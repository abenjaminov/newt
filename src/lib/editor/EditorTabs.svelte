<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { revealItemInDir } from "@tauri-apps/plugin-opener";
  import { get } from "svelte/store";
  import { tabs } from "./tabs-store";
  import { workspace } from "../workspace/workspace-store";
  import ContextMenu, { type MenuItem } from "../common/ContextMenu.svelte";

  let menu = $state<{ x: number; y: number; path: string } | null>(null);

  function relativeToWorkspace(absPath: string): string {
    const ws = get(workspace);
    if (!ws) return absPath;
    const norm = (s: string) => s.replace(/\\/g, "/").replace(/\/$/, "");
    const root = norm(ws.rootPath);
    const full = norm(absPath);
    if (full === root) return "";
    if (!full.startsWith(root + "/")) return absPath;
    return full.slice(root.length + 1);
  }

  function onTabMouseDown(e: MouseEvent, path: string) {
    if (e.button === 1) {
      // Middle click closes
      e.preventDefault();
      tabs.close(path);
    }
  }

  function onTabContextMenu(e: MouseEvent, path: string) {
    e.preventDefault();
    menu = { x: e.clientX, y: e.clientY, path };
  }

  function buildItems(path: string): MenuItem[] {
    const all = $tabs.tabs;
    const others = all.filter((t) => t.path !== path);
    const dirty = all.filter((t) => t.dirty);
    return [
      {
        label: "Close",
        action: () => tabs.close(path),
        hint: "Mid-click",
      },
      {
        label: "Close Others",
        action: () => others.forEach((t) => tabs.close(t.path)),
        disabled: others.length === 0,
      },
      {
        label: "Close Saved",
        action: () =>
          all.filter((t) => !t.dirty).forEach((t) => tabs.close(t.path)),
        disabled: all.length === dirty.length,
      },
      {
        label: "Close All",
        action: () => tabs.closeAll(),
      },
      { separator: true },
      {
        label: "Copy Path",
        action: () => navigator.clipboard.writeText(path).catch(() => {}),
      },
      {
        label: "Copy Relative Path",
        action: () =>
          navigator.clipboard.writeText(relativeToWorkspace(path)).catch(() => {}),
      },
      {
        label: "Reveal in File Explorer",
        action: () => revealItemInDir(path).catch(() => {}),
      },
      { separator: true },
      {
        label: "Save",
        action: async () => {
          const t = $tabs.tabs.find((tt) => tt.path === path);
          if (!t || !t.dirty) return;
          try {
            await invoke("write_file", { path, content: t.content });
            tabs.markSaved(path);
          } catch {
            // ignored
          }
        },
        disabled: !$tabs.tabs.find((t) => t.path === path)?.dirty,
        hint: "Ctrl+S",
      },
    ];
  }
</script>

<div class="tabs">
  {#each $tabs.tabs as t (t.path)}
    <div
      class="tab"
      class:active={t.path === $tabs.activePath}
      class:foreign={t.foreignWorktree !== null}
      title={t.foreignWorktree
        ? `${t.path}\n(from worktree: ${t.foreignWorktree})`
        : t.path}
      onmousedown={(e) => onTabMouseDown(e, t.path)}
      oncontextmenu={(e) => onTabContextMenu(e, t.path)}
      role="presentation"
    >
      <button class="tab-main" onclick={() => tabs.activate(t.path)}>
        {#if t.dirty}<span class="dot"></span>{/if}
        <span class="name">{t.name}</span>
        {#if t.foreignWorktree}
          <span class="wt-chip" title={`From worktree: ${t.foreignWorktree}`}>⎇ {t.foreignWorktree}</span>
        {/if}
      </button>
      <button class="close" title="Close (middle-click)" onclick={() => tabs.close(t.path)}
        >×</button
      >
    </div>
  {/each}
</div>

{#if menu}
  <ContextMenu
    x={menu.x}
    y={menu.y}
    items={buildItems(menu.path)}
    onClose={() => (menu = null)}
  />
{/if}

<style>
  .tabs {
    height: 32px;
    display: flex;
    background: var(--bg-2);
    border-bottom: 1px solid var(--border);
    overflow-x: auto;
    overflow-y: hidden;
    scrollbar-width: thin;
  }
  .tab {
    display: flex;
    align-items: center;
    height: 100%;
    border-right: 1px solid var(--border);
    padding: 0 0 0 12px;
    color: var(--fg-dim);
    flex-shrink: 0;
    position: relative;
    user-select: none;
  }
  .tab.active {
    background: var(--bg);
    color: var(--fg);
  }
  .tab.foreign .name {
    font-style: italic;
    color: var(--fg-faint);
  }
  .tab.foreign.active .name {
    color: var(--fg-dim);
  }
  .wt-chip {
    font-family: var(--font-mono);
    font-size: 10px;
    padding: 1px 5px;
    border-radius: 3px;
    background: rgba(201, 122, 63, 0.15);
    color: var(--accent-2);
    margin-left: 4px;
  }
  .tab.active::after {
    content: "";
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 2px;
    background: var(--accent);
  }
  .tab-main {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 0 8px 0 0;
    height: 100%;
  }
  .name {
    font-size: 12px;
    white-space: nowrap;
  }
  .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--accent);
    display: inline-block;
  }
  .close {
    width: 18px;
    height: 18px;
    border-radius: 4px;
    color: var(--fg-faint);
    margin-right: 6px;
    font-size: 14px;
    line-height: 1;
    opacity: 0;
  }
  .tab:hover .close,
  .tab.active .close {
    opacity: 1;
  }
  .close:hover {
    background: var(--bg-hover);
    color: var(--fg);
  }
</style>
