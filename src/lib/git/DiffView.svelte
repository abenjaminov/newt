<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { EditorState } from "@codemirror/state";
  import { EditorView, lineNumbers, drawSelection } from "@codemirror/view";
  import { MergeView, unifiedMergeView } from "@codemirror/merge";
  import { syntaxHighlighting } from "@codemirror/language";
  import { oneDarkHighlightStyle } from "@codemirror/theme-one-dark";
  import { editorTheme } from "../editor/theme";
  import { loaderForFilename } from "../editor/languages";
  import { fileAtHead } from "./git-store";

  type Props = {
    repo: string;
    path: string;
    isUntracked: boolean;
  };

  let { repo, path, isUntracked }: Props = $props();

  let host: HTMLDivElement;
  let view: EditorView | undefined;
  let mergeView: MergeView | undefined;
  let error = $state<string | null>(null);
  let oversize = $state<{ working: number; head: number } | null>(null);
  let forceRender = $state(false);
  let mode = $state<"side" | "unified">("side");

  // Diffing huge files via @codemirror/merge can stall the UI. Cap by default;
  // user can opt in via the "Render anyway" button.
  const MAX_DIFF_BYTES = 512 * 1024; // 512 KB

  function basename(p: string) {
    return p.split(/[\\/]/).filter(Boolean).pop() ?? p;
  }

  function destroy() {
    view?.destroy();
    view = undefined;
    mergeView?.destroy();
    mergeView = undefined;
    if (host) host.innerHTML = "";
  }

  async function build() {
    error = null;
    oversize = null;
    try {
      const absPath = `${repo}${repo.includes("/") ? "/" : "\\"}${path.replace(/\//g, repo.includes("/") ? "/" : "\\")}`;
      const [workingContent, headContent] = await Promise.all([
        invoke<string>("read_file", { path: absPath }).catch(() => ""),
        isUntracked ? Promise.resolve("") : fileAtHead(repo, path),
      ]);

      const workingBytes = new Blob([workingContent]).size;
      const headBytes = new Blob([headContent]).size;
      if (
        !forceRender &&
        (workingBytes > MAX_DIFF_BYTES || headBytes > MAX_DIFF_BYTES)
      ) {
        oversize = { working: workingBytes, head: headBytes };
        destroy();
        return;
      }

      const langLoader = loaderForFilename(basename(path));
      const lang = await langLoader();
      const baseExts = [
        EditorView.editable.of(false),
        lineNumbers(),
        drawSelection(),
        syntaxHighlighting(oneDarkHighlightStyle),
        editorTheme,
        ...(lang ? [lang] : []),
      ];

      destroy();

      if (mode === "side") {
        mergeView = new MergeView({
          a: { doc: headContent, extensions: baseExts },
          b: { doc: workingContent, extensions: baseExts },
          parent: host,
          highlightChanges: true,
          gutter: true,
          collapseUnchanged: { margin: 3, minSize: 4 },
        });
      } else {
        view = new EditorView({
          state: EditorState.create({
            doc: workingContent,
            extensions: [
              ...baseExts,
              unifiedMergeView({
                original: headContent,
                mergeControls: false,
                gutter: true,
                syntaxHighlightDeletions: true,
                collapseUnchanged: { margin: 3, minSize: 4 },
              }),
            ],
          }),
          parent: host,
        });
      }
    } catch (e) {
      error = String(e);
    }
  }

  function fmtSize(b: number): string {
    if (b < 1024) return `${b} B`;
    if (b < 1024 * 1024) return `${(b / 1024).toFixed(1)} KB`;
    return `${(b / (1024 * 1024)).toFixed(2)} MB`;
  }

  onMount(build);

  $effect(() => {
    // Rebuild when path/repo/mode changes.
    void path;
    void repo;
    void isUntracked;
    void mode;
    forceRender = false;
    if (host) build();
  });

  onDestroy(destroy);
</script>

<div class="diff-wrap">
  <div class="diff-header">
    <span class="path">{path}</span>
    {#if isUntracked}<span class="badge new">new file</span>{/if}
    <div class="grow"></div>
    <div class="legend" aria-hidden="true">
      <span class="lg add">+ added</span>
      <span class="lg del">− removed</span>
    </div>
    <div class="mode-toggle" role="tablist">
      <button
        class:active={mode === "side"}
        onclick={() => (mode = "side")}
        title="Side-by-side"
        role="tab"
        aria-selected={mode === "side"}
      >
        Side-by-side
      </button>
      <button
        class:active={mode === "unified"}
        onclick={() => (mode = "unified")}
        title="Unified"
        role="tab"
        aria-selected={mode === "unified"}
      >
        Unified
      </button>
    </div>
  </div>
  {#if error}
    <div class="error">{error}</div>
  {/if}
  {#if oversize}
    <div class="oversize">
      <div class="oversize-title">File is large — diff suppressed for performance</div>
      <div class="oversize-meta">
        Working: {fmtSize(oversize.working)} · HEAD: {fmtSize(oversize.head)}
      </div>
      <button
        class="render-btn"
        onclick={() => {
          forceRender = true;
          void build();
        }}
      >
        Render anyway
      </button>
    </div>
  {/if}
  <div class="diff-host" bind:this={host} class:hidden={oversize !== null}></div>
</div>

<style>
  .diff-wrap {
    height: 100%;
    display: flex;
    flex-direction: column;
    background: var(--bg);
  }
  .diff-header {
    height: 32px;
    padding: 0 12px;
    display: flex;
    align-items: center;
    gap: 12px;
    background: var(--bg-2);
    border-bottom: 1px solid var(--border);
    font-size: 12px;
    color: var(--fg-dim);
    flex-shrink: 0;
  }
  .path {
    font-family: var(--font-mono);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .grow {
    flex: 1;
  }
  .badge {
    padding: 1px 6px;
    border-radius: 3px;
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.4px;
  }
  .badge.new {
    background: rgba(158, 206, 106, 0.15);
    color: var(--green);
  }
  .legend {
    display: flex;
    gap: 8px;
    font-size: 10px;
    font-family: var(--font-mono);
    color: var(--fg-faint);
  }
  .lg {
    padding: 1px 5px;
    border-radius: 3px;
  }
  .lg.add {
    color: var(--green);
    background: rgba(143, 184, 119, 0.12);
  }
  .lg.del {
    color: var(--accent);
    background: rgba(232, 145, 90, 0.12);
  }
  .mode-toggle {
    display: flex;
    border: 1px solid var(--border);
    border-radius: 5px;
    overflow: hidden;
  }
  .mode-toggle button {
    padding: 3px 9px;
    font-size: 11px;
    color: var(--fg-faint);
    background: transparent;
    border-radius: 0;
  }
  .mode-toggle button:hover {
    background: var(--bg-hover);
    color: var(--fg);
  }
  .mode-toggle button.active {
    background: var(--bg-3);
    color: var(--fg);
  }
  .error {
    padding: 8px 12px;
    color: var(--red);
    font-size: 12px;
    white-space: pre-wrap;
  }
  .diff-host {
    flex: 1;
    overflow: auto;
    min-height: 0;
  }
  .diff-host.hidden {
    display: none;
  }
  .oversize {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    gap: 6px;
    color: var(--fg-dim);
    padding: 24px;
  }
  .oversize-title {
    font-size: 13px;
    color: var(--fg);
  }
  .oversize-meta {
    font-size: 11px;
    color: var(--fg-faint);
    font-family: var(--font-mono);
  }
  .render-btn {
    margin-top: 10px;
    padding: 6px 14px;
    border-radius: 4px;
    background: var(--bg-3);
    border: 1px solid var(--border);
    color: var(--fg-dim);
    font-size: 12px;
  }
  .render-btn:hover {
    background: var(--bg-hover);
    color: var(--fg);
  }

  /*
   * Diff styling — uses BOTH color and a non-color cue (text-decoration /
   * font-weight) so it's distinguishable for color-blind users.
   * Removed: warm orange + strikethrough, "−" gutter
   * Added:   green + bold underline, "+" gutter
   */
  :global(.cm-merge-deleted) {
    background-color: rgba(232, 145, 90, 0.18) !important;
    text-decoration: line-through;
    text-decoration-color: rgba(232, 145, 90, 0.6);
    text-decoration-thickness: 1.5px;
  }
  :global(.cm-deletedChunk) {
    background-color: rgba(232, 145, 90, 0.08) !important;
    border-left: 3px solid var(--accent);
  }
  :global(.cm-changedLine .cm-deletedText),
  :global(.cm-deletedChunk .cm-deletedText) {
    background-color: rgba(232, 145, 90, 0.28) !important;
    text-decoration: line-through;
  }
  :global(.cm-merge-inserted),
  :global(.cm-changedText) {
    background-color: rgba(143, 184, 119, 0.22) !important;
    font-weight: 600;
  }
  :global(.cm-changedLine) {
    background-color: rgba(143, 184, 119, 0.08) !important;
    border-left: 3px solid var(--green);
  }
  /* Side-by-side: force the merge view + its two editors to fill the host
   * width. CodeMirror's MergeView defaults don't stretch the inner editors;
   * we explicitly flex them 1:1. The selectors below cover both possible
   * @codemirror/merge DOM structures across versions. */
  .diff-wrap {
    width: 100%;
  }
  .diff-host {
    width: 100%;
  }
  :global(.cm-mergeView) {
    width: 100% !important;
    height: 100% !important;
    display: flex !important;
    flex-direction: row !important;
    align-items: stretch !important;
    box-sizing: border-box;
  }
  /* Direct children of cm-mergeView (whatever their class name) share width. */
  :global(.cm-mergeView > *) {
    flex: 1 1 0 !important;
    min-width: 0 !important;
    width: auto !important;
    height: 100%;
  }
  :global(.cm-mergeView > *:not(:last-child)) {
    border-right: 1px solid var(--border);
  }
  /* Some versions wrap each pane in cm-mergeViewEditors / cm-mergeViewEditor. */
  :global(.cm-mergeViewEditors) {
    width: 100% !important;
    height: 100% !important;
    display: flex !important;
    flex: 1 1 0 !important;
    min-width: 0 !important;
  }
  :global(.cm-mergeViewEditors > *) {
    flex: 1 1 0 !important;
    min-width: 0 !important;
    width: auto !important;
  }
  /* The actual CodeMirror editor instances inside the merge view. */
  :global(.cm-mergeView .cm-editor) {
    height: 100% !important;
    width: 100% !important;
    flex: 1 1 0;
  }
  :global(.cm-mergeView .cm-scroller) {
    width: 100%;
  }
  /* Gutter symbols (text marker, not just color). */
  :global(.cm-gutter.cm-changeGutter) {
    width: 14px;
    background: var(--bg-2);
  }
  :global(.cm-changeGutter .cm-gutterElement) {
    width: 14px;
    text-align: center;
    font-family: var(--font-mono);
    font-size: 10px;
    line-height: inherit;
  }
</style>
