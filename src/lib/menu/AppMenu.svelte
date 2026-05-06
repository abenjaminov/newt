<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { tabs, activeTab } from "../editor/tabs-store";
  import { workspace, activePanel } from "../workspace/workspace-store";
  import { paletteOpen } from "../palette/palette-store";
  import { settings, updateSettings } from "../settings/settings-store";
  import { pickAndOpenFolder } from "../workspace/open-workspace";
  import ContextMenu, { type MenuItem } from "../common/ContextMenu.svelte";

  type Props = {
    onOpenSettings: () => void;
    onCreateWorktree: () => void;
    onCloseWorkspace: () => void;
    onSaveActive: () => void | Promise<void>;
    onToggleSidebar: () => void;
    onToggleTerminal: () => void;
  };

  let {
    onOpenSettings,
    onCreateWorktree,
    onCloseWorkspace,
    onSaveActive,
    onToggleSidebar,
    onToggleTerminal,
  }: Props = $props();

  type MenuId = "file" | "edit" | "view" | "help";
  let openMenu = $state<MenuId | null>(null);
  let menuPos = $state<{ x: number; y: number }>({ x: 0, y: 0 });

  function showMenu(id: MenuId, e: MouseEvent) {
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    menuPos = { x: rect.left, y: rect.bottom };
    openMenu = id;
  }

  function hoverSwitch(id: MenuId, e: MouseEvent) {
    if (openMenu === null) return;
    showMenu(id, e);
  }

  async function pickFolder() {
    await pickAndOpenFolder();
  }

  function fileItems(): MenuItem[] {
    const t = $activeTab;
    return [
      {
        label: "Open Folder…",
        action: () => pickFolder(),
        hint: "Ctrl+O",
      },
      {
        label: "Close Folder",
        action: onCloseWorkspace,
        disabled: !$workspace,
      },
      { separator: true },
      {
        label: "New Worktree…",
        action: onCreateWorktree,
        disabled: !$workspace,
      },
      { separator: true },
      {
        label: "Save",
        action: () => onSaveActive(),
        hint: "Ctrl+S",
        disabled: !t || !t.dirty,
      },
      {
        label: "Close Tab",
        action: () => t && tabs.close(t.path),
        hint: "Ctrl+W",
        disabled: !t,
      },
      { separator: true },
      {
        label: "Settings",
        action: onOpenSettings,
        hint: "Ctrl+,",
      },
      { separator: true },
      {
        label: "Quit",
        action: () => getCurrentWindow().close(),
      },
    ];
  }

  function editItems(): MenuItem[] {
    return [
      {
        label: "Undo",
        action: () => document.execCommand("undo"),
        hint: "Ctrl+Z",
      },
      {
        label: "Redo",
        action: () => document.execCommand("redo"),
        hint: "Ctrl+Shift+Z",
      },
      { separator: true },
      {
        label: "Cut",
        action: () => document.execCommand("cut"),
        hint: "Ctrl+X",
      },
      {
        label: "Copy",
        action: () => document.execCommand("copy"),
        hint: "Ctrl+C",
      },
      {
        label: "Paste",
        action: () => document.execCommand("paste"),
        hint: "Ctrl+V",
      },
      {
        label: "Select All",
        action: () => document.execCommand("selectAll"),
        hint: "Ctrl+A",
      },
    ];
  }

  function viewItems(): MenuItem[] {
    return [
      {
        label: "Files",
        action: () => activePanel.set("files"),
        hint: "Ctrl+1",
      },
      {
        label: "Git Changes",
        action: () => activePanel.set("git"),
        hint: "Ctrl+2",
      },
      {
        label: "Processes",
        action: () => activePanel.set("processes"),
        hint: "Ctrl+3",
      },
      { separator: true },
      {
        label: "Toggle Sidebar",
        action: onToggleSidebar,
        hint: "Ctrl+B",
      },
      {
        label: "Toggle Terminal",
        action: onToggleTerminal,
        hint: "Ctrl+`",
      },
      { separator: true },
      {
        label: "Zoom In",
        action: () =>
          updateSettings({
            uiZoom: Math.min(2, +($settings.uiZoom + 0.1).toFixed(2)),
          }),
        hint: "Ctrl+=",
      },
      {
        label: "Zoom Out",
        action: () =>
          updateSettings({
            uiZoom: Math.max(0.5, +($settings.uiZoom - 0.1).toFixed(2)),
          }),
        hint: "Ctrl+-",
      },
      {
        label: "Reset Zoom",
        action: () => updateSettings({ uiZoom: 1 }),
        hint: "Ctrl+0",
      },
    ];
  }

  function helpItems(): MenuItem[] {
    return [
      {
        label: "Command Palette…",
        action: () => paletteOpen.set(true),
        hint: "Ctrl+P",
      },
      {
        label: "Keyboard Shortcuts",
        action: onOpenSettings,
        hint: "Ctrl+,",
      },
    ];
  }

  function itemsFor(id: MenuId): MenuItem[] {
    switch (id) {
      case "file":
        return fileItems();
      case "edit":
        return editItems();
      case "view":
        return viewItems();
      case "help":
        return helpItems();
    }
  }
</script>

<div class="menubar">
  {#each [
    { id: "file" as const, label: "File" },
    { id: "edit" as const, label: "Edit" },
    { id: "view" as const, label: "View" },
    { id: "help" as const, label: "Help" },
  ] as m}
    <button
      class="menu-trigger"
      class:active={openMenu === m.id}
      onclick={(e) => showMenu(m.id, e)}
      onmouseenter={(e) => hoverSwitch(m.id, e)}
    >
      {m.label}
    </button>
  {/each}
</div>

{#if openMenu}
  <ContextMenu
    x={menuPos.x}
    y={menuPos.y}
    items={itemsFor(openMenu)}
    onClose={() => (openMenu = null)}
  />
{/if}

<style>
  .menubar {
    display: flex;
    align-items: center;
    height: 100%;
    -webkit-app-region: no-drag;
  }
  .menu-trigger {
    height: 100%;
    padding: 0 9px;
    color: var(--fg-dim);
    font-size: 12px;
    border-radius: 0;
    line-height: 1;
  }
  .menu-trigger:hover,
  .menu-trigger.active {
    background: var(--bg-hover);
    color: var(--fg);
  }
</style>
