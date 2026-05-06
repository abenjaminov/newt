export type Symbol = {
  name: string;
  kind: string; // "fn", "class", "type", "const", "section", etc.
  line: number; // 1-based
};

type Pattern = { kind: string; re: RegExp };

const TS_JS: Pattern[] = [
  { kind: "fn", re: /^\s*(?:export\s+)?(?:async\s+)?function\s+([A-Za-z_$][\w$]*)/ },
  { kind: "class", re: /^\s*(?:export\s+)?(?:abstract\s+)?class\s+([A-Za-z_$][\w$]*)/ },
  { kind: "type", re: /^\s*(?:export\s+)?(?:interface|type|enum)\s+([A-Za-z_$][\w$]*)/ },
  { kind: "const", re: /^\s*(?:export\s+)?(?:const|let|var)\s+([A-Za-z_$][\w$]*)\s*=\s*(?:\(|async\s*\(|function\b)/ },
  { kind: "method", re: /^\s{2,}(?:async\s+)?(?:static\s+)?([A-Za-z_$][\w$]*)\s*\([^)]*\)\s*\{/ },
];

const RUST: Pattern[] = [
  { kind: "fn", re: /^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+([A-Za-z_][\w]*)/ },
  { kind: "struct", re: /^\s*(?:pub(?:\([^)]*\))?\s+)?struct\s+([A-Za-z_][\w]*)/ },
  { kind: "enum", re: /^\s*(?:pub(?:\([^)]*\))?\s+)?enum\s+([A-Za-z_][\w]*)/ },
  { kind: "trait", re: /^\s*(?:pub(?:\([^)]*\))?\s+)?trait\s+([A-Za-z_][\w]*)/ },
  { kind: "impl", re: /^\s*impl(?:<[^>]+>)?\s+(?:[A-Za-z_][\w:<>,'\s]*\s+for\s+)?([A-Za-z_][\w<>,'\s:]*)/ },
  { kind: "mod", re: /^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+([A-Za-z_][\w]*)/ },
  { kind: "type", re: /^\s*(?:pub(?:\([^)]*\))?\s+)?type\s+([A-Za-z_][\w]*)/ },
];

const PY: Pattern[] = [
  { kind: "fn", re: /^\s*(?:async\s+)?def\s+([A-Za-z_]\w*)/ },
  { kind: "class", re: /^\s*class\s+([A-Za-z_]\w*)/ },
];

const GO: Pattern[] = [
  { kind: "fn", re: /^\s*func\s+(?:\([^)]+\)\s+)?([A-Za-z_]\w*)/ },
  { kind: "type", re: /^\s*type\s+([A-Za-z_]\w*)/ },
];

const SVELTE: Pattern[] = [
  ...TS_JS,
];

const MARKDOWN: Pattern[] = [
  { kind: "section", re: /^(#{1,6})\s+(.+)$/ },
];

const LANGS: Record<string, Pattern[]> = {
  ts: TS_JS,
  tsx: TS_JS,
  js: TS_JS,
  jsx: TS_JS,
  mjs: TS_JS,
  cjs: TS_JS,
  rs: RUST,
  py: PY,
  go: GO,
  svelte: SVELTE,
  md: MARKDOWN,
  markdown: MARKDOWN,
};

function extOf(filename: string): string {
  const dot = filename.lastIndexOf(".");
  return dot > 0 ? filename.slice(dot + 1).toLowerCase() : "";
}

export function extractSymbols(filename: string, content: string): Symbol[] {
  const ext = extOf(filename);
  const patterns = LANGS[ext];
  if (!patterns) return [];
  const lines = content.split(/\r?\n/);
  const out: Symbol[] = [];
  const seen = new Set<string>();
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    if (!line) continue;
    if (line.length > 400) continue; // skip likely-minified rows
    for (const p of patterns) {
      const m = line.match(p.re);
      if (!m) continue;
      let name: string;
      let kind = p.kind;
      if (ext === "md" || ext === "markdown") {
        kind = `h${m[1].length}`;
        name = m[2].trim();
      } else {
        name = m[1].trim();
      }
      if (!name) continue;
      const key = `${i}:${name}`;
      if (seen.has(key)) continue;
      seen.add(key);
      out.push({ name, kind, line: i + 1 });
      break;
    }
  }
  return out;
}
