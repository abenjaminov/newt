<script lang="ts">
  import { onMount, tick } from "svelte";
  import { activeTab } from "../editor/tabs-store";
  import { openFileAt } from "../editor/goto";
  import { fuzzyScore, type Match } from "./fuzzy";
  import { extractSymbols, type Symbol } from "./symbols";

  type Props = {
    onClose: () => void;
  };

  let { onClose }: Props = $props();

  let input = $state<HTMLInputElement | undefined>(undefined);
  let listEl = $state<HTMLDivElement | undefined>(undefined);
  let query = $state("");
  let selected = $state(0);

  const symbols = $derived.by((): Symbol[] => {
    const t = $activeTab;
    if (!t || t.kind !== "text") return [];
    return extractSymbols(t.name, t.content);
  });

  type Result = { symbol: Symbol; match: Match };
  const results = $derived.by((): Result[] => {
    const q = query.trim();
    const out: Result[] = [];
    for (const s of symbols) {
      const m = q ? fuzzyScore(q, s.name) : { score: 50, ranges: [] };
      if (m === null) continue;
      out.push({ symbol: s, match: m });
    }
    if (!q) {
      // Preserve source order when there's no query.
      out.sort((a, b) => a.symbol.line - b.symbol.line);
    } else {
      out.sort((a, b) => b.match.score - a.match.score);
    }
    return out.slice(0, 200);
  });

  $effect(() => {
    void results;
    selected = 0;
    void scrollIntoView();
  });

  async function scrollIntoView() {
    await tick();
    if (!listEl) return;
    const item = listEl.children[selected] as HTMLElement | undefined;
    item?.scrollIntoView({ block: "nearest" });
  }

  function jump(r: Result) {
    onClose();
    const t = $activeTab;
    if (!t) return;
    void openFileAt(t.path, r.symbol.line);
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      selected = Math.min(selected + 1, results.length - 1);
      void scrollIntoView();
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      selected = Math.max(selected - 1, 0);
      void scrollIntoView();
    } else if (e.key === "Enter") {
      e.preventDefault();
      const r = results[selected];
      if (r) jump(r);
    } else if (e.key === "Escape") {
      e.preventDefault();
      onClose();
    }
  }

  function highlight(text: string, ranges: Array<[number, number]>): Array<{ s: string; on: boolean }> {
    if (ranges.length === 0) return [{ s: text, on: false }];
    const out: Array<{ s: string; on: boolean }> = [];
    let cursor = 0;
    for (const [a, b] of ranges) {
      if (cursor < a) out.push({ s: text.slice(cursor, a), on: false });
      out.push({ s: text.slice(a, b), on: true });
      cursor = b;
    }
    if (cursor < text.length) out.push({ s: text.slice(cursor), on: false });
    return out;
  }

  onMount(() => input?.focus());
</script>

<div class="backdrop" onclick={onClose} role="presentation"></div>
<div class="palette" role="dialog" aria-modal="true" aria-label="Outline">
  <input
    bind:this={input}
    type="text"
    placeholder="Symbols in this file…"
    bind:value={query}
    onkeydown={onKey}
    spellcheck="false"
    autocomplete="off"
  />

  <div class="results" bind:this={listEl}>
    {#each results as r, i (r.symbol.line + ":" + r.symbol.name)}
      <button
        class="row"
        class:selected={i === selected}
        onmouseenter={() => (selected = i)}
        onclick={() => jump(r)}
      >
        <span class="kind">{r.symbol.kind}</span>
        <span class="name">
          {#each highlight(r.symbol.name, r.match.ranges) as part}
            <span class:hl={part.on}>{part.s}</span>
          {/each}
        </span>
        <span class="line">:{r.symbol.line}</span>
      </button>
    {/each}
    {#if results.length === 0}
      <div class="empty">
        {symbols.length === 0
          ? "No outline available for this file"
          : "No matches"}
      </div>
    {/if}
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.45);
    z-index: 300;
  }
  .palette {
    position: fixed;
    top: 12vh;
    left: 50%;
    transform: translateX(-50%);
    width: min(560px, 90vw);
    max-height: 70vh;
    background: var(--bg-2);
    border: 1px solid var(--border);
    border-radius: 10px;
    z-index: 301;
    box-shadow: 0 24px 64px rgba(0, 0, 0, 0.6);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  input {
    background: transparent;
    border: none;
    border-bottom: 1px solid var(--border);
    border-radius: 0;
    padding: 14px 16px;
    color: var(--fg);
    font-size: 14px;
    font-family: var(--font-ui);
  }
  input:focus {
    outline: none;
    border-bottom-color: var(--accent);
  }
  .results {
    flex: 1;
    overflow-y: auto;
    padding: 4px;
    min-height: 0;
  }
  .row {
    display: grid;
    grid-template-columns: 50px 1fr auto;
    gap: 10px;
    align-items: center;
    width: 100%;
    text-align: left;
    padding: 6px 10px;
    border-radius: 6px;
    color: var(--fg-dim);
    font-size: 12px;
  }
  .row.selected {
    background: rgba(232, 145, 90, 0.12);
    color: var(--fg);
  }
  .kind {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--fg-faint);
    text-transform: uppercase;
    letter-spacing: 0.4px;
  }
  .row.selected .kind {
    color: var(--accent);
  }
  .name {
    font-family: var(--font-mono);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .line {
    color: var(--fg-faint);
    font-family: var(--font-mono);
    font-size: 11px;
  }
  .hl {
    color: var(--accent);
    font-weight: 600;
  }
  .empty {
    padding: 24px;
    text-align: center;
    color: var(--fg-faint);
    font-size: 12px;
  }
</style>
