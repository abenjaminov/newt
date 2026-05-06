<script lang="ts">
  import MarkdownIt from "markdown-it";

  type Props = {
    source: string;
    scrollPct?: number;
  };

  let { source, scrollPct = 0 }: Props = $props();

  const md = new MarkdownIt({
    html: false,
    linkify: true,
    typographer: true,
    breaks: false,
  });

  const rendered = $derived(md.render(source ?? ""));

  let host: HTMLDivElement | undefined = $state();
  let suppressNext = false;

  $effect(() => {
    void scrollPct;
    if (!host) return;
    const max = host.scrollHeight - host.clientHeight;
    if (max <= 0) return;
    const target = max * Math.max(0, Math.min(1, scrollPct));
    if (Math.abs(host.scrollTop - target) < 1) return;
    suppressNext = true;
    host.scrollTop = target;
  });

  function onUserScroll() {
    if (suppressNext) {
      suppressNext = false;
      return;
    }
    // User-initiated scroll — we don't propagate back yet (one-way for v1).
  }
</script>

<div class="md-preview" bind:this={host} onscroll={onUserScroll}>
  <div class="md-content">
    {@html rendered}
  </div>
</div>

<style>
  .md-preview {
    height: 100%;
    overflow: auto;
    background: var(--bg);
    border-left: 1px solid var(--border);
  }
  .md-content {
    max-width: 760px;
    margin: 0 auto;
    padding: 24px 32px 64px;
    color: var(--fg);
    font-size: 14px;
    line-height: 1.65;
    user-select: text;
  }
  .md-content :global(h1),
  .md-content :global(h2),
  .md-content :global(h3),
  .md-content :global(h4) {
    color: var(--fg);
    font-weight: 600;
    line-height: 1.3;
    margin: 1.6em 0 0.6em;
  }
  .md-content :global(h1) {
    font-size: 1.85em;
    border-bottom: 1px solid var(--border);
    padding-bottom: 0.3em;
    margin-top: 0;
  }
  .md-content :global(h2) {
    font-size: 1.45em;
    border-bottom: 1px solid var(--border);
    padding-bottom: 0.25em;
  }
  .md-content :global(h3) {
    font-size: 1.2em;
  }
  .md-content :global(p) {
    margin: 0 0 1em;
  }
  .md-content :global(a) {
    color: var(--accent);
    text-decoration: none;
  }
  .md-content :global(a:hover) {
    text-decoration: underline;
  }
  .md-content :global(code) {
    font-family: var(--font-mono);
    font-size: 0.9em;
    background: var(--bg-3);
    color: var(--accent-2);
    padding: 1px 5px;
    border-radius: 3px;
  }
  .md-content :global(pre) {
    background: var(--bg-2);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 12px 14px;
    overflow-x: auto;
    margin: 0 0 1em;
  }
  .md-content :global(pre code) {
    background: none;
    color: var(--fg);
    padding: 0;
    font-size: 12px;
    line-height: 1.55;
  }
  .md-content :global(blockquote) {
    margin: 0 0 1em;
    padding: 0 12px;
    color: var(--fg-dim);
    border-left: 3px solid var(--border);
  }
  .md-content :global(ul),
  .md-content :global(ol) {
    padding-left: 1.5em;
    margin: 0 0 1em;
  }
  .md-content :global(li) {
    margin: 0.2em 0;
  }
  .md-content :global(table) {
    border-collapse: collapse;
    margin: 0 0 1em;
    font-size: 13px;
  }
  .md-content :global(th),
  .md-content :global(td) {
    border: 1px solid var(--border);
    padding: 6px 10px;
    text-align: left;
  }
  .md-content :global(th) {
    background: var(--bg-2);
    font-weight: 600;
  }
  .md-content :global(hr) {
    border: 0;
    border-top: 1px solid var(--border);
    margin: 1.5em 0;
  }
  .md-content :global(img) {
    max-width: 100%;
    border-radius: 4px;
  }
</style>
