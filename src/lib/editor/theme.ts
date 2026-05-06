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
    "&.cm-focused .cm-selectionBackground, ::selection, .cm-selectionBackground":
      {
        backgroundColor: "rgba(232,145,90,0.25)",
      },
    ".cm-selectionMatch": {
      backgroundColor: "rgba(232,145,90,0.15)",
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
  },
  { dark: true },
);
