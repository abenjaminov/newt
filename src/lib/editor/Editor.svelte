<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { EditorState, Compartment } from "@codemirror/state";
  import { EditorView, keymap, lineNumbers, highlightActiveLine, highlightActiveLineGutter, drawSelection } from "@codemirror/view";
  import { settings } from "../settings/settings-store";
  import { defaultKeymap, history, historyKeymap, indentWithTab } from "@codemirror/commands";
  import { searchKeymap, search } from "@codemirror/search";
  import { autocompletion, completionKeymap, closeBrackets, closeBracketsKeymap } from "@codemirror/autocomplete";
  import {
    bracketMatching,
    foldGutter,
    foldKeymap,
    indentOnInput,
    syntaxHighlighting,
    defaultHighlightStyle,
  } from "@codemirror/language";
  import { lintKeymap } from "@codemirror/lint";
  import { oneDarkHighlightStyle } from "@codemirror/theme-one-dark";
  import { editorTheme } from "./theme";
  import { loaderForFilename } from "./languages";

  type Props = {
    path: string;
    initialContent: string;
    onChange: (next: string) => void;
    onSave: () => void;
    onScroll?: (pct: number) => void;
  };

  let { path, initialContent, onChange, onSave, onScroll }: Props = $props();

  let host: HTMLDivElement;
  let view: EditorView | undefined;
  const langCompartment = new Compartment();
  const tabSizeCompartment = new Compartment();
  const wrapCompartment = new Compartment();

  function buildExtensions(initialTabSize: number, initialWrap: boolean) {
    return [
      lineNumbers(),
      highlightActiveLine(),
      highlightActiveLineGutter(),
      foldGutter(),
      drawSelection(),
      history(),
      indentOnInput(),
      bracketMatching(),
      closeBrackets(),
      autocompletion(),
      search({ top: true }),
      syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
      syntaxHighlighting(oneDarkHighlightStyle),
      langCompartment.of([]),
      tabSizeCompartment.of(EditorState.tabSize.of(initialTabSize)),
      wrapCompartment.of(initialWrap ? [EditorView.lineWrapping] : []),
      editorTheme,
      keymap.of([
        ...closeBracketsKeymap,
        ...defaultKeymap,
        ...historyKeymap,
        ...foldKeymap,
        ...completionKeymap,
        ...lintKeymap,
        ...searchKeymap,
        indentWithTab,
        {
          key: "Mod-s",
          preventDefault: true,
          run: () => {
            onSave();
            return true;
          },
        },
      ]),
      EditorView.updateListener.of((u) => {
        if (u.docChanged) onChange(u.state.doc.toString());
      }),
    ];
  }

  async function applyLanguage(filename: string) {
    if (!view) return;
    const loader = loaderForFilename(filename);
    const lang = await loader();
    if (!view) return;
    view.dispatch({
      effects: langCompartment.reconfigure(lang ? [lang] : []),
    });
  }

  onMount(() => {
    let initialTabSize = 2;
    let initialWrap = true;
    const probe = settings.subscribe((s) => {
      initialTabSize = s.editorTabSize;
      initialWrap = s.editorLineWrap;
    });
    probe();
    view = new EditorView({
      state: EditorState.create({
        doc: initialContent,
        extensions: buildExtensions(initialTabSize, initialWrap),
      }),
      parent: host,
    });
    void applyLanguage(filenameOf(path));

    if (onScroll && view) {
      const scroller = view.scrollDOM;
      const handler = () => {
        const max = scroller.scrollHeight - scroller.clientHeight;
        onScroll!(max > 0 ? scroller.scrollTop / max : 0);
      };
      scroller.addEventListener("scroll", handler, { passive: true });
    }
  });

  // Live-reconfigure tab size and line wrap when settings change.
  $effect(() => {
    const tabSize = $settings.editorTabSize;
    if (!view) return;
    view.dispatch({
      effects: tabSizeCompartment.reconfigure(EditorState.tabSize.of(tabSize)),
    });
  });
  $effect(() => {
    const wrap = $settings.editorLineWrap;
    if (!view) return;
    view.dispatch({
      effects: wrapCompartment.reconfigure(wrap ? [EditorView.lineWrapping] : []),
    });
  });

  onDestroy(() => {
    view?.destroy();
    view = undefined;
  });

  function filenameOf(p: string) {
    return p.split(/[\\/]/).filter(Boolean).pop() ?? p;
  }

  // Reset doc + language when a different file is opened. Skips the dispatch
  // when the doc already matches initialContent (the common first-mount case),
  // otherwise CodeMirror reports a docChange and the tab spuriously goes dirty.
  let lastPath = "";
  $effect(() => {
    const p = path;
    if (p === lastPath) return;
    lastPath = p;
    if (!view) return;
    if (view.state.doc.toString() !== initialContent) {
      view.dispatch({
        changes: { from: 0, to: view.state.doc.length, insert: initialContent },
      });
    }
    void applyLanguage(filenameOf(p));
  });
</script>

<div class="editor" bind:this={host}></div>

<style>
  .editor {
    height: 100%;
    width: 100%;
    overflow: hidden;
    background: var(--bg);
  }
  :global(.cm-editor) {
    height: 100%;
  }
  :global(.cm-editor.cm-focused) {
    outline: none;
  }
</style>
