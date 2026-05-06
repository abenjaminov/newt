<script lang="ts">
  import type { SidePanel } from "../workspace/workspace-store";

  type Props = {
    active: SidePanel;
    collapsed: boolean;
    onSelect: (panel: SidePanel) => void;
    onOpenSettings: () => void;
  };

  let { active, collapsed, onSelect, onOpenSettings }: Props = $props();

  const items: { id: SidePanel; label: string; icon: string }[] = [
    { id: "files", label: "Files", icon: "📁" },
    { id: "search", label: "Search", icon: "🔍" },
    { id: "git", label: "Git", icon: "⎇" },
    { id: "processes", label: "Processes", icon: "▤" },
  ];
</script>

<aside class="activity-bar">
  <div class="top">
    {#each items as item}
      <button
        class="item"
        class:active={!collapsed && active === item.id}
        title={collapsed && active === item.id ? `${item.label} (expand)` : item.label}
        onclick={() => onSelect(item.id)}
      >
        <span class="icon">{item.icon}</span>
      </button>
    {/each}
  </div>
  <div class="bottom">
    <button class="item" title="Settings (Ctrl+,)" onclick={onOpenSettings}>
      <span class="icon">⚙</span>
    </button>
  </div>
</aside>

<style>
  .activity-bar {
    width: var(--activity-bar-w);
    background: var(--bg-2);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    align-items: stretch;
    justify-content: space-between;
    padding: 8px 0;
  }
  .top,
  .bottom {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
  }
  .item {
    width: 36px;
    height: 36px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 6px;
    color: var(--fg-dim);
    border-left: 2px solid transparent;
    transition: background 80ms ease;
  }
  .item:hover {
    background: var(--bg-hover);
    color: var(--fg);
  }
  .item.active {
    color: var(--fg);
    border-left-color: var(--accent);
  }
  .icon {
    font-size: 16px;
    line-height: 1;
  }
</style>
