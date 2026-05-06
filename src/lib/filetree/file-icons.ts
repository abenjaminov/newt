export type FileIcon = { glyph: string; color: string };

const EXT_MAP: Record<string, FileIcon> = {
  ts: { glyph: "TS", color: "#3178c6" },
  tsx: { glyph: "TS", color: "#3178c6" },
  d_ts: { glyph: "TS", color: "#3178c6" }, // synthetic key for .d.ts
  js: { glyph: "JS", color: "#f7df1e" },
  jsx: { glyph: "JS", color: "#f7df1e" },
  mjs: { glyph: "JS", color: "#f7df1e" },
  cjs: { glyph: "JS", color: "#f7df1e" },
  json: { glyph: "{ }", color: "#cbd5e1" },
  json5: { glyph: "{ }", color: "#cbd5e1" },
  jsonc: { glyph: "{ }", color: "#cbd5e1" },
  css: { glyph: "#", color: "#bb9af7" },
  scss: { glyph: "$", color: "#bb9af7" },
  sass: { glyph: "$", color: "#bb9af7" },
  less: { glyph: "$", color: "#bb9af7" },
  html: { glyph: "< >", color: "#e34c26" },
  htm: { glyph: "< >", color: "#e34c26" },
  svelte: { glyph: "S", color: "#ff3e00" },
  vue: { glyph: "V", color: "#41b883" },
  astro: { glyph: "A", color: "#ff5d01" },
  md: { glyph: "MD", color: "#bb9af7" },
  markdown: { glyph: "MD", color: "#bb9af7" },
  mdx: { glyph: "MX", color: "#bb9af7" },
  py: { glyph: "PY", color: "#3776ab" },
  pyi: { glyph: "PY", color: "#3776ab" },
  rs: { glyph: "RS", color: "#dea584" },
  go: { glyph: "GO", color: "#00add8" },
  cpp: { glyph: "C++", color: "#00599c" },
  cxx: { glyph: "C++", color: "#00599c" },
  cc: { glyph: "C++", color: "#00599c" },
  hpp: { glyph: "H+", color: "#00599c" },
  c: { glyph: "C", color: "#a8b9cc" },
  h: { glyph: "h", color: "#a8b9cc" },
  java: { glyph: "J", color: "#e76f00" },
  kt: { glyph: "KT", color: "#7f52ff" },
  scala: { glyph: "SC", color: "#dc322f" },
  swift: { glyph: "SW", color: "#fa7343" },
  rb: { glyph: "RB", color: "#cc342d" },
  php: { glyph: "PHP", color: "#777bb4" },
  sh: { glyph: "$_", color: "#9ece6a" },
  bash: { glyph: "$_", color: "#9ece6a" },
  zsh: { glyph: "$_", color: "#9ece6a" },
  fish: { glyph: "$_", color: "#9ece6a" },
  ps1: { glyph: ">_", color: "#012456" },
  yml: { glyph: "Y", color: "#cb171e" },
  yaml: { glyph: "Y", color: "#cb171e" },
  toml: { glyph: "T", color: "#9c4221" },
  ini: { glyph: "i", color: "#9aa1ad" },
  xml: { glyph: "< >", color: "#9aa1ad" },
  sql: { glyph: "DB", color: "#7aa2f7" },
  prisma: { glyph: "PR", color: "#0c344b" },
  graphql: { glyph: "GQ", color: "#e10098" },
  gql: { glyph: "GQ", color: "#e10098" },
  txt: { glyph: "T", color: "#9aa1ad" },
  log: { glyph: "L", color: "#9aa1ad" },
  csv: { glyph: "CS", color: "#9ece6a" },
  tsv: { glyph: "TS", color: "#9ece6a" },
  png: { glyph: "▣", color: "#7aa2f7" },
  jpg: { glyph: "▣", color: "#7aa2f7" },
  jpeg: { glyph: "▣", color: "#7aa2f7" },
  webp: { glyph: "▣", color: "#7aa2f7" },
  gif: { glyph: "▣", color: "#7aa2f7" },
  bmp: { glyph: "▣", color: "#7aa2f7" },
  ico: { glyph: "▣", color: "#7aa2f7" },
  svg: { glyph: "▣", color: "#bb9af7" },
  avif: { glyph: "▣", color: "#7aa2f7" },
  pdf: { glyph: "PDF", color: "#cc342d" },
  zip: { glyph: "▤", color: "#9aa1ad" },
  tar: { glyph: "▤", color: "#9aa1ad" },
  gz: { glyph: "▤", color: "#9aa1ad" },
  "7z": { glyph: "▤", color: "#9aa1ad" },
  rar: { glyph: "▤", color: "#9aa1ad" },
  env: { glyph: "ENV", color: "#9ece6a" },
  lock: { glyph: "🔒", color: "#9aa1ad" },
  lockb: { glyph: "🔒", color: "#fbf0df" },
  editorconfig: { glyph: "EC", color: "#9aa1ad" },
};

const NAME_MAP: Record<string, FileIcon> = {
  Dockerfile: { glyph: "🐳", color: "#0db7ed" },
  "docker-compose.yml": { glyph: "🐳", color: "#0db7ed" },
  "docker-compose.yaml": { glyph: "🐳", color: "#0db7ed" },
  Makefile: { glyph: "MK", color: "#427819" },
  GNUmakefile: { glyph: "MK", color: "#427819" },
  Procfile: { glyph: "PR", color: "#79589f" },
  Vagrantfile: { glyph: "VG", color: "#1563ff" },

  // Git
  ".gitignore": { glyph: "⎇", color: "#f05033" },
  ".gitattributes": { glyph: "⎇", color: "#f05033" },
  ".gitmodules": { glyph: "⎇", color: "#f05033" },
  ".gitkeep": { glyph: "⎇", color: "#f05033" },

  // Env / config dotfiles
  ".env": { glyph: "ENV", color: "#9ece6a" },
  ".dockerignore": { glyph: "🐳", color: "#0db7ed" },
  ".prettierrc": { glyph: "P", color: "#bb9af7" },
  ".prettierrc.json": { glyph: "P", color: "#bb9af7" },
  ".prettierrc.js": { glyph: "P", color: "#bb9af7" },
  ".prettierrc.cjs": { glyph: "P", color: "#bb9af7" },
  ".prettierrc.yml": { glyph: "P", color: "#bb9af7" },
  ".prettierrc.yaml": { glyph: "P", color: "#bb9af7" },
  ".prettierignore": { glyph: "P", color: "#bb9af7" },
  ".eslintrc": { glyph: "ES", color: "#4b32c3" },
  ".eslintrc.js": { glyph: "ES", color: "#4b32c3" },
  ".eslintrc.cjs": { glyph: "ES", color: "#4b32c3" },
  ".eslintrc.json": { glyph: "ES", color: "#4b32c3" },
  ".eslintrc.yml": { glyph: "ES", color: "#4b32c3" },
  ".eslintignore": { glyph: "ES", color: "#4b32c3" },
  ".biomeignore": { glyph: "B", color: "#60a5fa" },
  "biome.json": { glyph: "B", color: "#60a5fa" },
  "biome.jsonc": { glyph: "B", color: "#60a5fa" },
  ".editorconfig": { glyph: "EC", color: "#9aa1ad" },
  ".npmrc": { glyph: "📦", color: "#cb3837" },
  ".yarnrc": { glyph: "📦", color: "#2c8ebb" },
  ".yarnrc.yml": { glyph: "📦", color: "#2c8ebb" },
  ".nvmrc": { glyph: "JS", color: "#f7df1e" },
  ".tool-versions": { glyph: "TV", color: "#9aa1ad" },
  ".python-version": { glyph: "PY", color: "#3776ab" },
  ".ruby-version": { glyph: "RB", color: "#cc342d" },
  ".node-version": { glyph: "JS", color: "#83cd29" },
  ".replit": { glyph: "R", color: "#f26207" },
  ".devcontainer.json": { glyph: "🐳", color: "#0db7ed" },

  // npm / yarn / pnpm / bun
  "package.json": { glyph: "📦", color: "#cb3837" },
  "package-lock.json": { glyph: "🔒", color: "#cb3837" },
  "pnpm-lock.yaml": { glyph: "🔒", color: "#f7df1e" },
  "pnpm-workspace.yaml": { glyph: "PN", color: "#f7df1e" },
  "yarn.lock": { glyph: "🔒", color: "#2c8ebb" },
  "bun.lockb": { glyph: "🔒", color: "#fbf0df" },
  "bun.lock": { glyph: "🔒", color: "#fbf0df" },
  "bunfig.toml": { glyph: "BN", color: "#fbf0df" },

  // Rust
  "Cargo.toml": { glyph: "RS", color: "#dea584" },
  "Cargo.lock": { glyph: "🔒", color: "#dea584" },

  // Python
  "pyproject.toml": { glyph: "PY", color: "#3776ab" },
  "Pipfile": { glyph: "PY", color: "#3776ab" },
  "Pipfile.lock": { glyph: "🔒", color: "#3776ab" },
  "poetry.lock": { glyph: "🔒", color: "#3776ab" },
  "requirements.txt": { glyph: "PY", color: "#3776ab" },
  "setup.py": { glyph: "PY", color: "#3776ab" },
  "setup.cfg": { glyph: "PY", color: "#3776ab" },

  // Go / Ruby / PHP / Composer
  "go.mod": { glyph: "GO", color: "#00add8" },
  "go.sum": { glyph: "🔒", color: "#00add8" },
  Gemfile: { glyph: "RB", color: "#cc342d" },
  "Gemfile.lock": { glyph: "🔒", color: "#cc342d" },
  Rakefile: { glyph: "RB", color: "#cc342d" },
  "composer.json": { glyph: "PHP", color: "#777bb4" },
  "composer.lock": { glyph: "🔒", color: "#777bb4" },

  // Deno
  "deno.json": { glyph: "DE", color: "#70ffaf" },
  "deno.jsonc": { glyph: "DE", color: "#70ffaf" },
  "deno.lock": { glyph: "🔒", color: "#70ffaf" },

  // TS configs
  "tsconfig.json": { glyph: "TS", color: "#3178c6" },
  "tsconfig.app.json": { glyph: "TS", color: "#3178c6" },
  "tsconfig.node.json": { glyph: "TS", color: "#3178c6" },
  "tsconfig.base.json": { glyph: "TS", color: "#3178c6" },
  "tsconfig.build.json": { glyph: "TS", color: "#3178c6" },
  "jsconfig.json": { glyph: "JS", color: "#f7df1e" },

  // Build / framework configs
  "vite.config.ts": { glyph: "VT", color: "#bd34fe" },
  "vite.config.js": { glyph: "VT", color: "#bd34fe" },
  "vite.config.mjs": { glyph: "VT", color: "#bd34fe" },
  "vitest.config.ts": { glyph: "VI", color: "#fcc72b" },
  "vitest.config.js": { glyph: "VI", color: "#fcc72b" },
  "svelte.config.js": { glyph: "S", color: "#ff3e00" },
  "svelte.config.ts": { glyph: "S", color: "#ff3e00" },
  "next.config.js": { glyph: "NX", color: "#cbd5e1" },
  "next.config.ts": { glyph: "NX", color: "#cbd5e1" },
  "next.config.mjs": { glyph: "NX", color: "#cbd5e1" },
  "nuxt.config.ts": { glyph: "NU", color: "#00dc82" },
  "nuxt.config.js": { glyph: "NU", color: "#00dc82" },
  "astro.config.mjs": { glyph: "A", color: "#ff5d01" },
  "astro.config.ts": { glyph: "A", color: "#ff5d01" },
  "remix.config.js": { glyph: "RM", color: "#cbd5e1" },
  "tailwind.config.js": { glyph: "TW", color: "#38bdf8" },
  "tailwind.config.ts": { glyph: "TW", color: "#38bdf8" },
  "tailwind.config.cjs": { glyph: "TW", color: "#38bdf8" },
  "postcss.config.js": { glyph: "PC", color: "#dd3a0a" },
  "postcss.config.cjs": { glyph: "PC", color: "#dd3a0a" },
  "webpack.config.js": { glyph: "WP", color: "#8dd6f9" },
  "rollup.config.js": { glyph: "RO", color: "#ec4a3f" },
  "rollup.config.ts": { glyph: "RO", color: "#ec4a3f" },
  "esbuild.config.js": { glyph: "EB", color: "#fbcb39" },
  "jest.config.js": { glyph: "JE", color: "#c21325" },
  "jest.config.ts": { glyph: "JE", color: "#c21325" },
  "playwright.config.ts": { glyph: "PW", color: "#2ead33" },
  "cypress.config.ts": { glyph: "CY", color: "#04c38e" },
  "turbo.json": { glyph: "TU", color: "#ee0a87" },
  "nx.json": { glyph: "NX", color: "#143055" },
  "lerna.json": { glyph: "LE", color: "#9333ea" },
  "tauri.conf.json": { glyph: "TR", color: "#ffc131" },

  // Docs
  "README.md": { glyph: "📖", color: "#bb9af7" },
  README: { glyph: "📖", color: "#bb9af7" },
  LICENSE: { glyph: "⚖", color: "#9aa1ad" },
  "LICENSE.md": { glyph: "⚖", color: "#9aa1ad" },
  CHANGELOG: { glyph: "📜", color: "#9aa1ad" },
  "CHANGELOG.md": { glyph: "📜", color: "#bb9af7" },
  CONTRIBUTING: { glyph: "📜", color: "#9aa1ad" },
  "CONTRIBUTING.md": { glyph: "📜", color: "#bb9af7" },
  CODEOWNERS: { glyph: "👥", color: "#9aa1ad" },
};

const DEFAULT: FileIcon = { glyph: "·", color: "#6b7280" };

export function fileIcon(name: string): FileIcon {
  if (NAME_MAP[name]) return NAME_MAP[name];
  // Dotfile family fallbacks
  if (name.startsWith(".env")) return EXT_MAP.env;
  if (name.startsWith(".eslint")) return { glyph: "ES", color: "#4b32c3" };
  if (name.startsWith(".prettier")) return { glyph: "P", color: "#bb9af7" };
  if (name.endsWith(".d.ts")) return EXT_MAP.d_ts;
  const dot = name.lastIndexOf(".");
  if (dot < 0) return DEFAULT;
  const ext = name.slice(dot + 1).toLowerCase();
  return EXT_MAP[ext] ?? DEFAULT;
}

export function folderIcon(name: string): FileIcon {
  // Could differentiate well-known folders here; for v1 a single style.
  void name;
  return { glyph: "▾", color: "#e0af68" };
}
