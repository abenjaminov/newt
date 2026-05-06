<script lang="ts">
  import { onDestroy, onMount, tick } from "svelte";

  export type MenuItem =
    | {
        label: string;
        action: () => unknown | Promise<unknown>;
        danger?: boolean;
        disabled?: boolean;
        hint?: string;
      }
    | { separator: true };

  type Props = {
    x: number;
    y: number;
    items: MenuItem[];
    onClose: () => void;
  };

  let { x, y, items, onClose }: Props = $props();

  let menu: HTMLDivElement | undefined = $state();
  let placedX = $state(0);
  let placedY = $state(0);

  function isItem(i: MenuItem): i is Exclude<MenuItem, { separator: true }> {
    return !("separator" in i);
  }

  async function place() {
    placedX = x;
    placedY = y;
    await tick();
    if (!menu) return;
    const rect = menu.getBoundingClientRect();
    const vw = window.innerWidth;
    const vh = window.innerHeight;
    let nx = x;
    let ny = y;
    if (nx + rect.width + 4 > vw) nx = vw - rect.width - 4;
    if (ny + rect.height + 4 > vh) ny = vh - rect.height - 4;
    placedX = Math.max(4, nx);
    placedY = Math.max(4, ny);
  }

  function handleDocPointerDown(e: PointerEvent) {
    if (!menu) return;
    if (!menu.contains(e.target as Node)) onClose();
  }
  function handleKey(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.preventDefault();
      onClose();
    }
  }

  onMount(() => {
    void place();
    document.addEventListener("pointerdown", handleDocPointerDown, true);
    document.addEventListener("keydown", handleKey);
  });

  onDestroy(() => {
    document.removeEventListener("pointerdown", handleDocPointerDown, true);
    document.removeEventListener("keydown", handleKey);
  });

  async function pick(i: Exclude<MenuItem, { separator: true }>) {
    if (i.disabled) return;
    onClose();
    await i.action();
  }
</script>

<div
  class="ctx-menu"
  bind:this={menu}
  style:top="{placedY}px"
  style:left="{placedX}px"
  role="menu"
>
  {#each items as item}
    {#if isItem(item)}
      <button
        class="item"
        class:danger={item.danger}
        disabled={item.disabled}
        onclick={() => pick(item)}
      >
        <span class="label">{item.label}</span>
        {#if item.hint}<span class="hint">{item.hint}</span>{/if}
      </button>
    {:else}
      <div class="sep"></div>
    {/if}
  {/each}
</div>

<style>
  .ctx-menu {
    position: fixed;
    z-index: 400;
    background: var(--bg-3);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 4px;
    min-width: 180px;
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.5);
    user-select: none;
  }
  .item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    width: 100%;
    text-align: left;
    padding: 5px 9px;
    border-radius: 4px;
    color: var(--fg-dim);
    font-size: 12px;
  }
  .item:hover:not(:disabled) {
    background: var(--bg-hover);
    color: var(--fg);
  }
  .item:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .item.danger {
    color: var(--red);
  }
  .item.danger:hover:not(:disabled) {
    background: rgba(247, 118, 142, 0.12);
  }
  .hint {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--fg-faint);
  }
  .sep {
    height: 1px;
    background: var(--border);
    margin: 4px 6px;
  }
</style>
