import type { LanguageSupport } from "@codemirror/language";

export type LangLoader = () => Promise<LanguageSupport>;

const ext = (e: string): LangLoader => {
  const map: Record<string, LangLoader> = {
    js: async () => (await import("@codemirror/lang-javascript")).javascript({ jsx: false }),
    jsx: async () => (await import("@codemirror/lang-javascript")).javascript({ jsx: true }),
    ts: async () =>
      (await import("@codemirror/lang-javascript")).javascript({ jsx: false, typescript: true }),
    tsx: async () =>
      (await import("@codemirror/lang-javascript")).javascript({ jsx: true, typescript: true }),
    mjs: async () => (await import("@codemirror/lang-javascript")).javascript({ jsx: false }),
    cjs: async () => (await import("@codemirror/lang-javascript")).javascript({ jsx: false }),
    json: async () => (await import("@codemirror/lang-json")).json(),
    css: async () => (await import("@codemirror/lang-css")).css(),
    html: async () => (await import("@codemirror/lang-html")).html(),
    htm: async () => (await import("@codemirror/lang-html")).html(),
    svelte: async () => (await import("@codemirror/lang-html")).html(),
    md: async () => (await import("@codemirror/lang-markdown")).markdown(),
    markdown: async () => (await import("@codemirror/lang-markdown")).markdown(),
    rs: async () => (await import("@codemirror/lang-rust")).rust(),
    py: async () => (await import("@codemirror/lang-python")).python(),
    pyi: async () => (await import("@codemirror/lang-python")).python(),
    go: async () => (await import("@codemirror/lang-go")).go(),
    yaml: async () => (await import("@codemirror/lang-yaml")).yaml(),
    yml: async () => (await import("@codemirror/lang-yaml")).yaml(),
    toml: async () => (await import("@codemirror/lang-yaml")).yaml(),
  };
  return map[e] ?? (async () => null as unknown as LanguageSupport);
};

const filenameMap: Record<string, LangLoader> = {
  Dockerfile: async () => (await import("@codemirror/lang-yaml")).yaml(),
  Makefile: async () => (await import("@codemirror/lang-yaml")).yaml(),
};

export function loaderForFilename(name: string): LangLoader {
  if (filenameMap[name]) return filenameMap[name];
  const dot = name.lastIndexOf(".");
  if (dot < 0) return async () => null as unknown as LanguageSupport;
  const e = name.slice(dot + 1).toLowerCase();
  return ext(e);
}

export function isMarkdown(name: string): boolean {
  const dot = name.lastIndexOf(".");
  if (dot < 0) return false;
  const e = name.slice(dot + 1).toLowerCase();
  return e === "md" || e === "markdown";
}

const BINARY_IMAGE_EXTS = new Set([
  "png", "jpg", "jpeg", "webp", "gif", "bmp", "ico", "avif", "tiff", "tif",
]);

export type FileKind = "text" | "image" | "svg";

export function fileKind(name: string): FileKind {
  const dot = name.lastIndexOf(".");
  if (dot < 0) return "text";
  const e = name.slice(dot + 1).toLowerCase();
  if (e === "svg") return "svg";
  if (BINARY_IMAGE_EXTS.has(e)) return "image";
  return "text";
}
