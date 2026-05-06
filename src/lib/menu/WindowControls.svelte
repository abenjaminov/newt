<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onMount, onDestroy } from "svelte";

  let isMaximized = $state(false);
  let unlisten: (() => void) | undefined;

  async function refresh() {
    try {
      isMaximized = await getCurrentWindow().isMaximized();
    } catch {
      // ignored — running outside Tauri
    }
  }

  onMount(async () => {
    void refresh();
    try {
      const win = getCurrentWindow();
      const u = await win.onResized(() => void refresh());
      unlisten = u;
    } catch {
      // ignored
    }
  });

  onDestroy(() => unlisten?.());

  async function minimize() {
    try {
      await getCurrentWindow().minimize();
    } catch {}
  }
  async function toggleMaximize() {
    try {
      await getCurrentWindow().toggleMaximize();
      void refresh();
    } catch {}
  }
  async function close() {
    try {
      await getCurrentWindow().close();
    } catch {}
  }
</script>

<div class="controls">
  <button class="ctl" title="Minimize" onclick={minimize} aria-label="Minimize">
    <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
      <line x1="0" y1="5" x2="10" y2="5" stroke="currentColor" stroke-width="1" />
    </svg>
  </button>
  <button
    class="ctl"
    title={isMaximized ? "Restore" : "Maximize"}
    onclick={toggleMaximize}
    aria-label={isMaximized ? "Restore" : "Maximize"}
  >
    {#if isMaximized}
      <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
        <rect x="2" y="0" width="8" height="8" stroke="currentColor" stroke-width="1" fill="none" />
        <rect x="0" y="2" width="8" height="8" stroke="currentColor" stroke-width="1" fill="var(--bg-2)" />
      </svg>
    {:else}
      <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
        <rect x="0" y="0" width="10" height="10" stroke="currentColor" stroke-width="1" fill="none" />
      </svg>
    {/if}
  </button>
  <button class="ctl close" title="Close" onclick={close} aria-label="Close">
    <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
      <line x1="0" y1="0" x2="10" y2="10" stroke="currentColor" stroke-width="1" />
      <line x1="10" y1="0" x2="0" y2="10" stroke="currentColor" stroke-width="1" />
    </svg>
  </button>
</div>

<style>
  .controls {
    display: flex;
    align-items: center;
    height: 100%;
    -webkit-app-region: no-drag;
  }
  .ctl {
    width: 46px;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--fg-dim);
    border-radius: 0;
  }
  .ctl:hover {
    background: var(--bg-hover);
    color: var(--fg);
  }
  .ctl.close:hover {
    background: #e81123;
    color: white;
  }
</style>
