<script lang="ts">
  import { eventToChord } from "./keybinding-store";

  type Props = {
    chord: string | null;
    onChange: (next: string | null) => void;
  };

  let { chord, onChange }: Props = $props();

  let recording = $state(false);
  let host: HTMLDivElement | undefined = $state();

  function start() {
    recording = true;
    host?.focus();
  }

  function cancel() {
    recording = false;
  }

  function clear(e: MouseEvent) {
    e.stopPropagation();
    onChange(null);
    recording = false;
  }

  function onKey(e: KeyboardEvent) {
    if (!recording) return;
    e.preventDefault();
    e.stopPropagation();
    if (e.key === "Escape") {
      recording = false;
      return;
    }
    if (e.key === "Backspace") {
      onChange(null);
      recording = false;
      return;
    }
    const next = eventToChord(e);
    if (next) {
      onChange(next);
      recording = false;
    }
  }
</script>

<div
  class="recorder"
  class:recording
  bind:this={host}
  onclick={start}
  onkeydown={onKey}
  role="button"
  tabindex="0"
>
  {#if recording}
    <span class="prompt">Press a key chord… <span class="muted">(Esc cancel · Backspace clear)</span></span>
  {:else if chord}
    <span class="chord">{chord}</span>
    <button class="x" title="Clear" onclick={clear}>×</button>
  {:else}
    <span class="empty">Click to assign</span>
  {/if}
</div>

<style>
  .recorder {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 4px 8px;
    border: 1px dashed var(--border);
    border-radius: 5px;
    color: var(--fg-dim);
    font-size: 11px;
    cursor: pointer;
    min-width: 110px;
    background: var(--bg-3);
    outline: none;
  }
  .recorder:hover {
    border-color: var(--fg-faint);
  }
  .recorder.recording {
    border-style: solid;
    border-color: var(--accent);
    color: var(--accent);
    background: rgba(232, 145, 90, 0.08);
  }
  .chord {
    font-family: var(--font-mono);
    color: var(--fg);
  }
  .empty {
    color: var(--fg-faint);
    font-style: italic;
  }
  .prompt {
    font-style: italic;
  }
  .muted {
    color: var(--fg-faint);
    font-style: normal;
  }
  .x {
    margin-left: auto;
    width: 18px;
    height: 18px;
    border-radius: 3px;
    color: var(--fg-faint);
    font-size: 13px;
    line-height: 1;
  }
  .x:hover {
    background: var(--bg-hover);
    color: var(--fg);
  }
</style>
