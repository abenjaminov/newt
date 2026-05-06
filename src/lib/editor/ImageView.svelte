<script lang="ts">
  type Props = {
    src: string;
    name: string;
    size: number | null;
  };

  let { src, name, size }: Props = $props();

  let img: HTMLImageElement | undefined = $state();
  let natural = $state<{ w: number; h: number } | null>(null);
  let zoom = $state<"fit" | "actual">("fit");

  function onLoad() {
    if (img) natural = { w: img.naturalWidth, h: img.naturalHeight };
  }

  function fmtSize(b: number | null): string {
    if (b === null) return "";
    if (b < 1024) return `${b} B`;
    if (b < 1024 * 1024) return `${(b / 1024).toFixed(1)} KB`;
    return `${(b / (1024 * 1024)).toFixed(2)} MB`;
  }
</script>

<div class="image-view">
  <div class="meta">
    <span class="name">{name}</span>
    {#if natural}
      <span class="dim">{natural.w} × {natural.h}</span>
    {/if}
    {#if size !== null}
      <span class="size">{fmtSize(size)}</span>
    {/if}
    <div class="grow"></div>
    <div class="zoom">
      <button class:active={zoom === "fit"} onclick={() => (zoom = "fit")}>Fit</button>
      <button class:active={zoom === "actual"} onclick={() => (zoom = "actual")}>1:1</button>
    </div>
  </div>
  <div class="canvas" class:checker={true}>
    <img bind:this={img} class={zoom} {src} alt={name} onload={onLoad} />
  </div>
</div>

<style>
  .image-view {
    height: 100%;
    display: flex;
    flex-direction: column;
    background: var(--bg);
  }
  .meta {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 6px 12px;
    border-bottom: 1px solid var(--border);
    background: var(--bg-2);
    color: var(--fg-faint);
    font-size: 11px;
    flex-shrink: 0;
  }
  .name {
    color: var(--fg-dim);
    font-family: var(--font-mono);
  }
  .dim {
    color: var(--accent-2);
    font-family: var(--font-mono);
  }
  .grow {
    flex: 1;
  }
  .zoom {
    display: flex;
    gap: 2px;
  }
  .zoom button {
    padding: 3px 8px;
    border-radius: 4px;
    color: var(--fg-faint);
    font-size: 10px;
  }
  .zoom button:hover {
    background: var(--bg-hover);
    color: var(--fg);
  }
  .zoom button.active {
    background: var(--bg-3);
    color: var(--fg);
  }
  .canvas {
    flex: 1;
    overflow: auto;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 24px;
    min-height: 0;
  }
  /* Subtle checker pattern for transparency */
  .canvas.checker {
    background-image:
      linear-gradient(45deg, rgba(255, 255, 255, 0.02) 25%, transparent 25%),
      linear-gradient(-45deg, rgba(255, 255, 255, 0.02) 25%, transparent 25%),
      linear-gradient(45deg, transparent 75%, rgba(255, 255, 255, 0.02) 75%),
      linear-gradient(-45deg, transparent 75%, rgba(255, 255, 255, 0.02) 75%);
    background-size: 20px 20px;
    background-position: 0 0, 0 10px, 10px -10px, -10px 0px;
  }
  img.fit {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
  }
  img.actual {
    max-width: none;
    max-height: none;
  }
</style>
