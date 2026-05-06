<script lang="ts">
  import { onMount, tick } from "svelte";
  import { openFileAtPath } from "../editor/open-file";
  import {
    commandRegistry,
    fileIndex,
    paletteOpen,
    type Command,
    type FileEntry,
  } from "./palette-store";
  import { fuzzyScore, type Match } from "./fuzzy";
  import { keymap } from "../keybindings/keybinding-store";

  type Result =
    | { kind: "command"; command: Command; match: Match }
    | { kind: "file"; file: FileEntry; match: Match };

  const MAX_RESULTS = 60;

  let input: HTMLInputElement | undefined = $state();
  let listEl: HTMLDivElement | undefined = $state();
  let query = $state("");
  let selected = $state(0);

  const results = $derived.by((): Result[] => {
    const q = query.trim();
    const out: Result[] = [];

    // Commands
    for (const cmd of $commandRegistry) {
      const text = `${cmd.group}: ${cmd.title}`;
      const m = q ? fuzzyScore(q, text) : { score: 50, ranges: [] };
      if (m === null) continue;
      out.push({ kind: "command", command: cmd, match: m });
    }

    // Files (only when there's a query — avoids huge lists on empty input)
    if (q) {
      for (const f of $fileIndex) {
        const m = fuzzyScore(q, f.rel);
        if (m === null) continue;
        // Bonus for matching the basename specifically
        const nameMatch = fuzzyScore(q, f.name);
        const score =
          m.score + (nameMatch ? Math.max(0, nameMatch.score) * 0.3 : 0);
        out.push({ kind: "file", file: f, match: { ...m, score } });
      }
    }

    out.sort((a, b) => b.match.score - a.match.score);
    return out.slice(0, MAX_RESULTS);
  });

  $effect(() => {
    // Reset selection when results change.
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

  function close() {
    paletteOpen.set(false);
    query = "";
  }

  async function run(r: Result) {
    close();
    if (r.kind === "command") {
      await r.command.run();
    } else {
      const res = await openFileAtPath(r.file.path);
      if (!res.ok) console.error(res.error);
    }
  }

  function onKey(e: KeyboardEvent) {
    // Ctrl+P / Ctrl+Shift+P toggles palette closed when already open.
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "p") {
      e.preventDefault();
      close();
      return;
    }
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
      if (r) void run(r);
    } else if (e.key === "Escape") {
      e.preventDefault();
      close();
    } else if (e.key === "Home" || (e.key === "PageUp" && (e.metaKey || e.ctrlKey))) {
      e.preventDefault();
      selected = 0;
      void scrollIntoView();
    } else if (e.key === "End" || (e.key === "PageDown" && (e.metaKey || e.ctrlKey))) {
      e.preventDefault();
      selected = results.length - 1;
      void scrollIntoView();
    }
  }

  onMount(() => {
    input?.focus();
  });

  // Highlight the matched ranges in a result label.
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

  function iconFor(r: Result): string {
    if (r.kind === "file") return "📄";
    switch (r.command.group) {
      case "Settings": return "⚙";
      case "View": return "▤";
      case "Worktree": return "⎇";
      case "Terminal": return "❯_";
      case "Git": return "⎇";
      case "File": return "📄";
      default: return "›";
    }
  }
</script>

<div
  class="backdrop"
  onclick={close}
  role="presentation"
></div>
<div class="palette" role="dialog" aria-modal="true" aria-label="Command palette">
  <input
    bind:this={input}
    type="text"
    placeholder="Search files, commands, settings…"
    bind:value={query}
    onkeydown={onKey}
    spellcheck="false"
    autocomplete="off"
  />

  <div class="results" bind:this={listEl}>
    {#each results as r, i (r.kind === "command" ? `c:${r.command.id}` : `f:${r.file.path}`)}
      <button
        class="row"
        class:selected={i === selected}
        onmouseenter={() => (selected = i)}
        onclick={() => run(r)}
      >
        <span class="icon">{iconFor(r)}</span>
        <span class="main">
          {#if r.kind === "command"}
            <span class="title">
              {#each highlight(`${r.command.group}: ${r.command.title}`, r.match.ranges) as part}
                <span class:hl={part.on}>{part.s}</span>
              {/each}
            </span>
          {:else}
            <span class="title">
              {#each highlight(r.file.name, r.match.ranges.map(([a, b]) => {
                const offset = r.file.rel.length - r.file.name.length;
                return [Math.max(0, a - offset), Math.max(0, b - offset)] as [number, number];
              }).filter(([a, b]) => b > a)) as part}
                <span class:hl={part.on}>{part.s}</span>
              {/each}
            </span>
            <span class="sub">
              {#each highlight(r.file.rel, r.match.ranges) as part}
                <span class:hl={part.on}>{part.s}</span>
              {/each}
            </span>
          {/if}
        </span>
        {#if r.kind === "command" && ($keymap[r.command.id] || r.command.hint)}
          <span class="hint">{$keymap[r.command.id] ?? r.command.hint}</span>
        {/if}
      </button>
    {/each}

    {#if results.length === 0}
      <div class="empty">No results</div>
    {/if}
  </div>

  <div class="footer">
    <span><kbd>↑</kbd><kbd>↓</kbd> navigate</span>
    <span><kbd>↵</kbd> open</span>
    <span><kbd>Esc</kbd> close</span>
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
    width: min(640px, 90vw);
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
    grid-template-columns: 22px minmax(0, 1fr) auto;
    gap: 10px;
    align-items: center;
    width: 100%;
    text-align: left;
    padding: 8px 10px;
    border-radius: 6px;
    color: var(--fg-dim);
    font-size: 12px;
  }
  .row.selected {
    background: rgba(232, 145, 90, 0.12);
    color: var(--fg);
  }
  .row.selected::before {
    content: "";
    position: absolute;
  }
  .icon {
    font-size: 13px;
    color: var(--fg-faint);
    text-align: center;
  }
  .row.selected .icon {
    color: var(--accent);
  }
  .main {
    display: flex;
    flex-direction: column;
    gap: 1px;
    min-width: 0;
  }
  .title {
    color: var(--fg);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .sub {
    color: var(--fg-faint);
    font-family: var(--font-mono);
    font-size: 10px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .hl {
    color: var(--accent);
    font-weight: 600;
  }
  .row.selected .hl {
    color: var(--accent-2);
  }
  .hint {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--fg-faint);
  }
  .empty {
    padding: 24px;
    text-align: center;
    color: var(--fg-faint);
    font-size: 12px;
  }
  .footer {
    display: flex;
    gap: 12px;
    padding: 6px 12px;
    border-top: 1px solid var(--border);
    background: var(--bg);
    color: var(--fg-faint);
    font-size: 10px;
  }
  .footer kbd {
    background: var(--bg-2);
    border: 1px solid var(--border);
    padding: 0 4px;
    border-radius: 3px;
    font-family: var(--font-mono);
    font-size: 9px;
    margin-right: 2px;
  }
</style>
