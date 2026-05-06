<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { workspace } from "../workspace/workspace-store";
  import { openFileAt } from "../editor/goto";

  type Hit = {
    path: string;
    rel: string;
    line: number;
    col_start: number;
    col_end: number;
    preview: string;
  };
  type Result = {
    hits: Hit[];
    truncated: boolean;
    files_scanned: number;
  };
  type FileGroup = {
    path: string;
    rel: string;
    hits: Hit[];
    expanded: boolean;
  };

  let query = $state("");
  let caseSensitive = $state(false);
  let wholeWord = $state(false);
  let isRegex = $state(false);
  let includeHidden = $state(false);
  let respectGitignore = $state(true);

  let groups = $state<FileGroup[]>([]);
  let totalHits = $state(0);
  let truncated = $state(false);
  let filesScanned = $state(0);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let inputEl = $state<HTMLInputElement | undefined>(undefined);

  let timer: ReturnType<typeof setTimeout> | undefined;
  let runId = 0;
  const DEBOUNCE_MS = 300;

  function scheduleSearch(immediate = false) {
    if (timer) clearTimeout(timer);
    if (immediate) {
      void runSearch();
    } else {
      timer = setTimeout(runSearch, DEBOUNCE_MS);
    }
  }

  async function runSearch() {
    const ws = $workspace;
    if (!ws) return;
    if (query.trim().length < 2 && !isRegex) {
      groups = [];
      totalHits = 0;
      truncated = false;
      filesScanned = 0;
      error = null;
      return;
    }
    const id = ++runId;
    loading = true;
    error = null;
    try {
      const res = await invoke<Result>("search_in_files", {
        root: ws.rootPath,
        query,
        caseSensitive,
        wholeWord,
        isRegex,
        includeHidden,
        respectGitignore,
      });
      if (id !== runId) return;
      const byFile = new Map<string, FileGroup>();
      for (const h of res.hits) {
        let g = byFile.get(h.path);
        if (!g) {
          g = { path: h.path, rel: h.rel, hits: [], expanded: true };
          byFile.set(h.path, g);
        }
        g.hits.push(h);
      }
      groups = Array.from(byFile.values());
      totalHits = res.hits.length;
      truncated = res.truncated;
      filesScanned = res.files_scanned;
    } catch (e) {
      if (id === runId) error = String(e);
    } finally {
      if (id === runId) loading = false;
    }
  }

  function jumpTo(h: Hit) {
    void openFileAt(h.path, h.line, h.col_start, h.col_end);
  }

  function toggleGroup(g: FileGroup) {
    g.expanded = !g.expanded;
    groups = [...groups];
  }

  function onQueryInput(e: Event) {
    query = (e.currentTarget as HTMLInputElement).value;
    scheduleSearch(false);
  }

  function flip(name: "case" | "word" | "regex" | "hidden" | "git") {
    if (name === "case") caseSensitive = !caseSensitive;
    else if (name === "word") wholeWord = !wholeWord;
    else if (name === "regex") isRegex = !isRegex;
    else if (name === "hidden") includeHidden = !includeHidden;
    else respectGitignore = !respectGitignore;
    scheduleSearch(true);
  }

  export function focusInput() {
    inputEl?.focus();
    inputEl?.select();
  }

  function highlight(line: string, col: number, end: number): { before: string; mid: string; after: string } {
    const safeStart = Math.max(0, Math.min(line.length, col));
    const safeEnd = Math.max(safeStart, Math.min(line.length, end));
    return {
      before: line.slice(0, safeStart),
      mid: line.slice(safeStart, safeEnd),
      after: line.slice(safeEnd),
    };
  }
</script>

<div class="search-panel">
  <div class="input-row">
    <input
      bind:this={inputEl}
      type="text"
      placeholder="Search…"
      value={query}
      oninput={onQueryInput}
      spellcheck="false"
      autocomplete="off"
    />
  </div>
  <div class="toggles">
    <button
      class="t"
      class:on={caseSensitive}
      title="Match case"
      onclick={() => flip("case")}
    >Aa</button>
    <button
      class="t"
      class:on={wholeWord}
      title="Whole word"
      onclick={() => flip("word")}
    >\b</button>
    <button
      class="t"
      class:on={isRegex}
      title="Regex"
      onclick={() => flip("regex")}
    >.*</button>
    <span class="spacer"></span>
    <button
      class="t"
      class:on={!respectGitignore}
      title={respectGitignore ? "Skipping .gitignored files" : "Searching .gitignored too"}
      onclick={() => flip("git")}
    >git</button>
    <button
      class="t"
      class:on={includeHidden}
      title={includeHidden ? "Including hidden files" : "Skipping hidden files"}
      onclick={() => flip("hidden")}
    >.h</button>
  </div>

  {#if loading}
    <div class="status">Searching…</div>
  {:else if error}
    <div class="error">{error}</div>
  {:else if query && totalHits === 0}
    <div class="status">No results ({filesScanned} files)</div>
  {:else if totalHits > 0}
    <div class="status">
      {totalHits} hit{totalHits === 1 ? "" : "s"} in {groups.length} file{groups.length === 1 ? "" : "s"}
      {#if truncated}<span class="trunc">(truncated)</span>{/if}
    </div>
    <div class="results">
      {#each groups as g (g.path)}
        <div class="group">
          <button class="group-head" onclick={() => toggleGroup(g)} title={g.path}>
            <span class="chev">{g.expanded ? "▾" : "▸"}</span>
            <span class="rel">{g.rel}</span>
            <span class="count">{g.hits.length}</span>
          </button>
          {#if g.expanded}
            {#each g.hits as h (h.line + ":" + h.col_start + ":" + h.preview)}
              {@const parts = highlight(h.preview, h.col_start, h.col_end)}
              <button class="hit" onclick={() => jumpTo(h)}>
                <span class="ln">{h.line}</span>
                <span class="prev"
                  >{parts.before}<mark>{parts.mid}</mark>{parts.after}</span
                >
              </button>
            {/each}
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .search-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
    font-size: 12px;
  }
  .input-row {
    padding: 6px 8px 4px;
  }
  .input-row input {
    width: 100%;
    height: 26px;
    padding: 0 8px;
    background: var(--bg-3);
    border: 1px solid var(--border);
    border-radius: 4px;
    color: var(--fg);
    font-size: 12px;
  }
  .input-row input:focus {
    outline: none;
    border-color: var(--accent);
  }
  .toggles {
    display: flex;
    gap: 2px;
    padding: 0 8px 6px;
    align-items: center;
    border-bottom: 1px solid var(--border);
  }
  .toggles .spacer {
    flex: 1;
  }
  .t {
    height: 22px;
    min-width: 26px;
    padding: 0 6px;
    border-radius: 4px;
    color: var(--fg-faint);
    font-family: var(--font-mono);
    font-size: 11px;
  }
  .t:hover {
    background: var(--bg-hover);
    color: var(--fg);
  }
  .t.on {
    background: rgba(232, 145, 90, 0.15);
    color: var(--accent);
  }
  .status {
    padding: 6px 12px;
    color: var(--fg-faint);
    font-size: 11px;
  }
  .trunc {
    color: var(--yellow);
    margin-left: 4px;
  }
  .error {
    padding: 8px 12px;
    color: var(--red);
    font-size: 11px;
    white-space: pre-wrap;
  }
  .results {
    flex: 1;
    overflow: auto;
  }
  .group {
    margin-bottom: 2px;
  }
  .group-head {
    display: flex;
    align-items: center;
    gap: 4px;
    width: 100%;
    height: 22px;
    padding: 0 8px;
    text-align: left;
    color: var(--fg-dim);
    font-size: 12px;
  }
  .group-head:hover {
    background: var(--bg-hover);
    color: var(--fg);
  }
  .chev {
    width: 12px;
    color: var(--fg-faint);
    font-size: 10px;
    text-align: center;
  }
  .rel {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .count {
    color: var(--fg-faint);
    font-size: 11px;
  }
  .hit {
    display: flex;
    gap: 8px;
    width: 100%;
    padding: 2px 8px 2px 28px;
    text-align: left;
    color: var(--fg-dim);
    font-family: var(--font-mono);
    font-size: 11px;
    line-height: 1.5;
    align-items: flex-start;
  }
  .hit:hover {
    background: var(--bg-hover);
    color: var(--fg);
  }
  .ln {
    color: var(--fg-faint);
    min-width: 32px;
    text-align: right;
    flex-shrink: 0;
  }
  .prev {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
  }
  .hit mark {
    background: rgba(232, 145, 90, 0.4);
    color: inherit;
    padding: 0;
    border-radius: 2px;
  }
</style>
