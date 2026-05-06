<script lang="ts">
  import { onMount } from "svelte";
  import {
    activeWorktreePath,
    worktrees,
    setActiveWorktree,
    type Worktree,
  } from "./worktree-store";

  type Props = {
    onSwitch: (target: Worktree) => void;
    onCreate: () => void;
    direction?: "down" | "up";
  };

  let { onSwitch, onCreate, direction = "down" }: Props = $props();

  let open = $state(false);
  let menuRoot: HTMLDivElement;

  const active = $derived(
    $worktrees.find((w) => w.path === $activeWorktreePath) ?? null,
  );

  function pick(wt: Worktree) {
    open = false;
    if (wt.path === $activeWorktreePath) return;
    setActiveWorktree(wt.path);
    onSwitch(wt);
  }

  function handleDocClick(e: MouseEvent) {
    if (!open) return;
    if (menuRoot && !menuRoot.contains(e.target as Node)) open = false;
  }

  onMount(() => {
    const handler = (e: KeyboardEvent) => {
      const cmd = e.metaKey || e.ctrlKey;
      if (cmd && e.shiftKey && e.key.toLowerCase() === "w") {
        e.preventDefault();
        open = !open;
      }
    };
    window.addEventListener("keydown", handler);
    document.addEventListener("click", handleDocClick);
    return () => {
      window.removeEventListener("keydown", handler);
      document.removeEventListener("click", handleDocClick);
    };
  });

  function basename(p: string): string {
    return p.split(/[\\/]/).filter(Boolean).pop() ?? p;
  }
</script>

<div class="switcher" bind:this={menuRoot}>
  <button class="trigger" onclick={() => (open = !open)} title="Switch worktree (Ctrl+Shift+W)">
    <span class="branch">⎇ {active?.branch ?? (active?.detached ? "detached" : "—")}</span>
    {#if active?.dirty}<span class="dot dirty" title="Uncommitted changes"></span>{/if}
    <span class="caret">▾</span>
  </button>

  {#if open}
    <div class="menu" class:up={direction === "up"}>
      <div class="menu-head">Worktrees</div>
      {#each $worktrees as wt (wt.path)}
        <button
          class="item"
          class:active={wt.path === $activeWorktreePath}
          onclick={() => pick(wt)}
        >
          <span class="item-branch">⎇ {wt.branch ?? (wt.detached ? "detached" : "?")}</span>
          {#if wt.dirty}<span class="dot dirty"></span>{/if}
          <span class="item-path">{basename(wt.path)}</span>
          {#if wt.ahead > 0 || wt.behind > 0}
            <span class="ab">
              {#if wt.ahead}↑{wt.ahead}{/if}
              {#if wt.behind}↓{wt.behind}{/if}
            </span>
          {/if}
        </button>
      {/each}
      <div class="divider"></div>
      <button
        class="item add"
        onclick={() => {
          open = false;
          onCreate();
        }}
      >
        + New worktree…
      </button>
    </div>
  {/if}
</div>

<style>
  .switcher {
    position: relative;
    -webkit-app-region: no-drag;
  }
  .trigger {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 10px;
    border-radius: 4px;
    color: var(--fg-dim);
  }
  .trigger:hover {
    background: var(--bg-hover);
    color: var(--fg);
  }
  .branch,
  .item-branch {
    font-family: var(--font-mono);
    font-size: 12px;
  }
  .caret {
    color: var(--fg-faint);
    font-size: 10px;
  }
  .menu {
    position: absolute;
    top: calc(100% + 4px);
    left: 0;
    background: var(--bg-3);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 4px;
    min-width: 280px;
    z-index: 100;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
  }
  .menu.up {
    top: auto;
    bottom: calc(100% + 4px);
  }
  .menu-head {
    padding: 4px 8px 6px;
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--fg-faint);
    font-weight: 600;
  }
  .item {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    text-align: left;
    padding: 6px 8px;
    border-radius: 4px;
    color: var(--fg-dim);
    font-size: 12px;
  }
  .item:hover {
    background: var(--bg-hover);
    color: var(--fg);
  }
  .item.active {
    background: var(--bg-hover);
    color: var(--fg);
  }
  .item.active::before {
    content: "•";
    color: var(--accent);
    margin-left: -2px;
  }
  .item-path {
    color: var(--fg-faint);
    font-family: var(--font-mono);
    font-size: 11px;
    margin-left: auto;
  }
  .ab {
    color: var(--fg-faint);
    font-size: 10px;
    margin-left: 6px;
  }
  .dot.dirty {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--yellow);
  }
  .divider {
    height: 1px;
    background: var(--border);
    margin: 4px 0;
  }
  .item.add {
    color: var(--accent);
  }
</style>
