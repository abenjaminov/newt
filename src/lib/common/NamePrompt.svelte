<script lang="ts">
  import { onMount } from "svelte";

  type Props = {
    title: string;
    placeholder?: string;
    initial?: string;
    confirmLabel?: string;
    /** If provided, called synchronously to validate the name. Return a string
     * to show inline as an error (which blocks confirm), or null to allow. */
    validate?: (name: string) => string | null;
    onConfirm: (name: string) => void | Promise<void>;
    onCancel: () => void;
  };

  let {
    title,
    placeholder = "",
    initial = "",
    confirmLabel = "Create",
    validate,
    onConfirm,
    onCancel,
  }: Props = $props();

  let value = $state("");
  let busy = $state(false);
  let inputEl: HTMLInputElement | undefined = $state();

  const error = $derived.by(() => {
    if (!validate) return null;
    return validate(value.trim());
  });

  async function confirm() {
    const v = value.trim();
    if (!v || error) return;
    busy = true;
    try {
      await onConfirm(v);
    } finally {
      busy = false;
    }
  }

  function onKeyDown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.preventDefault();
      onCancel();
    } else if (e.key === "Enter") {
      e.preventDefault();
      void confirm();
    }
  }

  onMount(() => {
    value = initial;
    queueMicrotask(() => {
      inputEl?.focus();
      inputEl?.select();
    });
  });
</script>

<div class="backdrop" onclick={onCancel} role="presentation"></div>
<div class="prompt" role="dialog" aria-modal="true" aria-label={title}>
  <div class="title">{title}</div>
  <input
    bind:this={inputEl}
    type="text"
    bind:value
    {placeholder}
    onkeydown={onKeyDown}
    spellcheck="false"
    autocomplete="off"
  />
  {#if error}
    <div class="error">{error}</div>
  {/if}
  <div class="actions">
    <button class="btn" onclick={onCancel} disabled={busy}>Cancel</button>
    <button
      class="btn primary"
      onclick={confirm}
      disabled={busy || !value.trim() || error !== null}
    >
      {busy ? "…" : confirmLabel}
    </button>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    z-index: 250;
  }
  .prompt {
    position: fixed;
    top: 25%;
    left: 50%;
    transform: translateX(-50%);
    background: var(--bg-2);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 16px 18px 14px;
    width: min(420px, 90vw);
    z-index: 251;
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.45);
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .title {
    color: var(--fg);
    font-size: 13px;
    font-weight: 500;
  }
  input {
    width: 100%;
    font-size: 13px;
    padding: 7px 10px;
    background: var(--bg-3);
    border: 1px solid var(--border);
    border-radius: 5px;
    color: var(--fg);
    font-family: var(--font-mono);
  }
  input:focus {
    outline: none;
    border-color: var(--accent);
  }
  .error {
    color: var(--red);
    font-size: 11px;
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 6px;
    margin-top: 4px;
  }
  .btn {
    padding: 5px 12px;
    border-radius: 5px;
    color: var(--fg-dim);
    font-size: 12px;
  }
  .btn:hover:not(:disabled) {
    background: var(--bg-hover);
    color: var(--fg);
  }
  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .btn.primary {
    background: var(--accent);
    color: #1a0e08;
    font-weight: 500;
  }
  .btn.primary:hover:not(:disabled) {
    filter: brightness(1.08);
    background: var(--accent);
    color: #1a0e08;
  }
</style>
