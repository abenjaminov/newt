<script lang="ts">
  import { onMount } from "svelte";
  import Terminal from "./Terminal.svelte";
  import { listShells, type ShellInfo } from "./pty-client";
  import { terminals } from "./terminal-store";
  import { workspace } from "../workspace/workspace-store";

  type Props = {
    collapsed: boolean;
    onToggleCollapse: () => void;
  };

  let { collapsed, onToggleCollapse }: Props = $props();

  let shells = $state<ShellInfo[]>([]);
  let menuOpen = $state(false);
  let loading = $state(true);

  onMount(async () => {
    try {
      shells = await listShells();
    } catch (e) {
      console.error("listShells failed", e);
    } finally {
      loading = false;
    }
  });

  function preferredWslDistro(rootPath: string | undefined): string | null {
    if (!rootPath) return null;
    const m = rootPath.match(/^\\\\wsl(?:\$|\.localhost)\\([^\\]+)/i);
    return m ? m[1] : null;
  }

  function defaultShell(): ShellInfo | undefined {
    const distro = preferredWslDistro($workspace?.rootPath);
    if (distro) {
      const wsl = shells.find(
        (s) => s.kind === "wsl" && s.id.toLowerCase() === `wsl:${distro.toLowerCase()}`,
      );
      if (wsl) return wsl;
      const anyWsl = shells.find((s) => s.kind === "wsl");
      if (anyWsl) return anyWsl;
    }
    return shells.find((s) => s.default) ?? shells[0];
  }

  function spawn(shell: ShellInfo) {
    const ws = $workspace;
    terminals.add({
      id: 0, // placeholder; Terminal component owns the actual pty handle
      label: shell.label,
      shell,
    });
    void ws;
    menuOpen = false;
  }

  function spawnDefault() {
    const s = defaultShell();
    if (s) spawn(s);
  }

  function close(uid: number) {
    terminals.remove(uid);
  }

  function shellIcon(kind: ShellInfo["kind"]): string {
    switch (kind) {
      case "wsl":
        return "🐧";
      case "pwsh":
      case "powershell":
        return "❯_";
      case "cmd":
        return "▶";
      case "posix":
        return "$";
      default:
        return "$";
    }
  }
</script>

<div class="panel">
  <div class="tabs">
    {#each $terminals.sessions as t (t.uid)}
      <div class="tab" class:active={t.uid === $terminals.activeUid}>
        <button class="tab-main" onclick={() => terminals.activate(t.uid)} title={t.shell.label}>
          <span class="ic">{shellIcon(t.shell.kind)}</span>
          <span class="lbl">{t.label}</span>
          {#if t.exited}<span class="exited">·</span>{/if}
        </button>
        <button class="x" title="Close" onclick={() => close(t.uid)}>×</button>
      </div>
    {/each}

    <div class="add">
      <button class="add-btn" onclick={() => (menuOpen = !menuOpen)} title="New terminal" disabled={loading}>+</button>
      {#if menuOpen}
        <div class="menu">
          {#each shells as s (s.id)}
            <button class="menu-item" onclick={() => spawn(s)}>
              <span class="ic">{shellIcon(s.kind)}</span>
              {s.label}
              {#if s.default}<span class="default">default</span>{/if}
            </button>
          {/each}
        </div>
      {/if}
    </div>

    <div class="spacer"></div>

    <button
      class="collapse-btn"
      title={collapsed ? "Expand terminal" : "Collapse terminal"}
      onclick={onToggleCollapse}
    >
      {collapsed ? "▴" : "▾"}
    </button>
  </div>

  <div class="panes" class:hidden={collapsed}>
    {#if $terminals.sessions.length === 0}
      <div class="empty">
        <button class="big" onclick={spawnDefault} disabled={loading || !defaultShell()}>
          {loading ? "Detecting shells…" : `Open ${defaultShell()?.label ?? "terminal"}`}
        </button>
      </div>
    {:else}
      {#each $terminals.sessions as t (t.uid)}
        <div class="pane" class:active={t.uid === $terminals.activeUid}>
          <Terminal
            uid={t.uid}
            shell={t.shell}
            cwd={$workspace?.rootPath ?? null}
            visible={t.uid === $terminals.activeUid}
          />
        </div>
      {/each}
    {/if}
  </div>
</div>

<style>
  .panel {
    height: 100%;
    display: flex;
    flex-direction: column;
    background: var(--bg);
  }
  .tabs {
    display: flex;
    align-items: center;
    height: 28px;
    background: var(--bg-2);
    border-bottom: 1px solid var(--border);
    padding-left: 6px;
    flex-shrink: 0;
    position: relative;
  }
  .tab {
    display: flex;
    align-items: center;
    height: 100%;
    border-radius: 6px 6px 0 0;
    padding: 0 0 0 8px;
    margin-right: 2px;
    color: var(--fg-dim);
    font-size: 12px;
    flex-shrink: 0;
    user-select: none;
  }
  .tab:hover {
    background: var(--bg-hover);
  }
  .tab.active {
    background: var(--bg);
    color: var(--fg);
  }
  .tab-main {
    display: flex;
    align-items: center;
    gap: 6px;
    height: 100%;
    padding-right: 4px;
  }
  .ic {
    font-size: 11px;
    opacity: 0.85;
  }
  .lbl {
    white-space: nowrap;
  }
  .exited {
    color: var(--fg-faint);
  }
  .x {
    width: 18px;
    height: 18px;
    color: var(--fg-faint);
    border-radius: 4px;
    margin-right: 4px;
    line-height: 1;
    font-size: 14px;
  }
  .x:hover {
    background: var(--bg-hover);
    color: var(--fg);
  }
  .add {
    position: relative;
  }
  .add-btn {
    width: 22px;
    height: 22px;
    border-radius: 4px;
    color: var(--fg-faint);
    margin: 0 4px;
  }
  .add-btn:hover:not(:disabled) {
    background: var(--bg-hover);
    color: var(--fg);
  }
  .menu {
    position: absolute;
    top: 26px;
    left: 0;
    background: var(--bg-3);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 4px;
    min-width: 200px;
    z-index: 10;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
  }
  .menu-item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    text-align: left;
    padding: 6px 8px;
    border-radius: 4px;
    color: var(--fg-dim);
    font-size: 12px;
  }
  .menu-item:hover {
    background: var(--bg-hover);
    color: var(--fg);
  }
  .default {
    margin-left: auto;
    font-size: 10px;
    color: var(--fg-faint);
    background: var(--bg-2);
    padding: 1px 5px;
    border-radius: 3px;
  }
  .spacer {
    flex: 1;
  }
  .collapse-btn {
    width: 22px;
    height: 22px;
    border-radius: 4px;
    color: var(--fg-faint);
    margin-right: 6px;
    font-size: 10px;
    line-height: 1;
  }
  .collapse-btn:hover {
    background: var(--bg-hover);
    color: var(--fg);
  }
  .panes {
    flex: 1;
    position: relative;
    overflow: hidden;
    min-height: 0;
  }
  .panes.hidden {
    display: none;
  }
  .pane {
    position: absolute;
    inset: 0;
    visibility: hidden;
  }
  .pane.active {
    visibility: visible;
  }
  .empty {
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .big {
    color: var(--fg-dim);
    font-size: 12px;
    padding: 8px 16px;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--bg-2);
  }
  .big:hover:not(:disabled) {
    background: var(--bg-hover);
    color: var(--fg);
  }
  .big:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
</style>
