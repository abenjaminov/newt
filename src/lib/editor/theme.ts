import { EditorView } from "@codemirror/view";

export const editorTheme = EditorView.theme(
  {
    "&": {
      height: "100%",
      backgroundColor: "var(--bg)",
      color: "var(--fg)",
      fontSize: "var(--editor-font-size)",
      fontFamily: "var(--font-mono)",
    },
    ".cm-scroller": {
      fontFamily: "var(--font-mono)",
      fontSize: "var(--editor-font-size)",
      lineHeight: "1.55",
    },
    ".cm-content": {
      caretColor: "var(--accent)",
      padding: "8px 0",
    },
    ".cm-gutters": {
      backgroundColor: "var(--bg)",
      color: "var(--fg-faint)",
      border: "none",
    },
    ".cm-activeLineGutter": {
      backgroundColor: "transparent",
      color: "var(--fg-dim)",
    },
    ".cm-activeLine": {
      backgroundColor: "rgba(255,255,255,0.025)",
    },
    ".cm-cursor, .cm-dropCursor": {
      borderLeftColor: "var(--accent)",
    },
    "&.cm-focused .cm-selectionBackground, .cm-selectionBackground, .cm-content ::selection":
      {
        backgroundColor: "rgba(232,145,90,0.45)",
      },
    ".cm-selectionMatch": {
      backgroundColor: "rgba(232,145,90,0.28)",
      outline: "1px solid rgba(232,145,90,0.55)",
    },
    ".cm-searchMatch": {
      backgroundColor: "rgba(232,145,90,0.35)",
      outline: "1px solid rgba(232,145,90,0.7)",
    },
    ".cm-searchMatch.cm-searchMatch-selected": {
      backgroundColor: "rgba(232,145,90,0.65)",
    },
    ".cm-matchingBracket, .cm-nonmatchingBracket": {
      backgroundColor: "rgba(232,145,90,0.2)",
      color: "inherit",
    },
    ".cm-tooltip": {
      backgroundColor: "var(--bg-3)",
      border: "1px solid var(--border)",
      color: "var(--fg)",
    },
    ".cm-foldPlaceholder": {
      backgroundColor: "var(--bg-3)",
      color: "var(--fg-dim)",
      border: "none",
    },
    ".cm-panels": {
      backgroundColor: "var(--bg-2)",
      color: "var(--fg)",
      borderBottom: "1px solid var(--border)",
    },
    ".cm-panels.cm-panels-top": {
      borderBottom: "1px solid var(--border)",
    },
    ".cm-panel.cm-search": {
      padding: "6px 8px",
    },
    ".cm-panel.cm-search input, .cm-panel.cm-search button, .cm-panel.cm-search label":
      {
        fontSize: "12px",
      },
    ".cm-panel.cm-search input": {
      backgroundColor: "var(--bg-3)",
      color: "var(--fg)",
      border: "1px solid var(--border)",
      borderRadius: "3px",
      padding: "2px 6px",
    },
    ".cm-panel.cm-search input:focus": {
      outline: "none",
      borderColor: "var(--accent)",
    },
    ".cm-panel.cm-search button": {
      backgroundColor: "var(--bg-3)",
      color: "var(--fg-dim)",
      border: "1px solid var(--border)",
      borderRadius: "3px",
      padding: "2px 8px",
      cursor: "pointer",
    },
    ".cm-panel.cm-search button:hover": {
      backgroundColor: "var(--bg-hover)",
      color: "var(--fg)",
    },
    ".cm-panel.cm-search [name=close]": {
      color: "var(--fg-faint)",
    },
  },
  { dark: true },
);
