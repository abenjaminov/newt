<script lang="ts">
  import { revealItemInDir } from "@tauri-apps/plugin-opener";

  type Props = {
    path: string;
    rootPath: string | null;
  };

  let { path, rootPath }: Props = $props();

  function pathSeparator(p: string): string {
    return p.includes("\\") && !p.includes("/") ? "\\" : "/";
  }

  function relativePath(full: string, root: string | null): string | null {
    if (!root) return null;
    const norm = (s: string) => s.replace(/\\/g, "/").replace(/\/$/, "");
    const nFull = norm(full);
    const nRoot = norm(root);
    if (nFull === nRoot) return "";
    if (!nFull.startsWith(nRoot + "/")) return null;
    return nFull.slice(nRoot.length + 1);
  }

  const segments = $derived.by(() => {
    const sep = pathSeparator(path);
    const rel = relativePath(path, rootPath);
    if (rel !== null) {
      return rel.split(/[\\/]/).filter(Boolean);
    }
    return path.split(/[\\/]/).filter(Boolean);
  });

  const isOutsideRoot = $derived(relativePath(path, rootPath) === null);
</script>

<div class="breadcrumbs" title={path}>
  {#if isOutsideRoot}
    <span class="ext" title="Outside the open folder">↗</span>
  {/if}
  {#each segments as seg, i (i)}
    {#if i > 0}<span class="sep">›</span>{/if}
    <span class="seg" class:file={i === segments.length - 1}>{seg}</span>
  {/each}
  <div class="grow"></div>
  <button
    class="reveal"
    title="Reveal in File Explorer"
    onclick={() => revealItemInDir(path).catch(() => {})}
  >
    ↗
  </button>
</div>

<style>
  .breadcrumbs {
    display: flex;
    align-items: center;
    gap: 4px;
    height: 24px;
    padding: 0 4px 0 12px;
    background: var(--bg);
    border-bottom: 1px solid var(--border);
    font-size: 11px;
    font-family: var(--font-mono);
    color: var(--fg-faint);
    overflow: hidden;
    white-space: nowrap;
    flex-shrink: 0;
    user-select: text;
  }
  .seg {
    color: var(--fg-faint);
    overflow: hidden;
    text-overflow: ellipsis;
    flex-shrink: 1;
    min-width: 0;
  }
  .seg.file {
    color: var(--fg-dim);
    flex-shrink: 0;
  }
  .sep {
    color: var(--fg-faint);
    opacity: 0.5;
    margin: 0 2px;
    flex-shrink: 0;
  }
  .ext {
    color: var(--accent-2);
    font-style: italic;
    margin-right: 4px;
    flex-shrink: 0;
  }
  .grow {
    flex: 1;
    min-width: 8px;
  }
  .reveal {
    width: 22px;
    height: 20px;
    border-radius: 3px;
    color: var(--fg-faint);
    font-size: 11px;
    line-height: 1;
    user-select: none;
  }
  .reveal:hover {
    background: var(--bg-hover);
    color: var(--fg);
  }
</style>
