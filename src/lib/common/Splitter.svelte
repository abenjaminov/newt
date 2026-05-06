<script lang="ts">
  type Props = {
    /** "vertical" = vertical line splitting L/R panes (drag horizontally). */
    orientation: "vertical" | "horizontal";
    /** The grid container whose space is being divided. */
    container: HTMLElement | undefined;
    /** Called with a fraction 0..1 representing the splitter position. */
    onChange: (fraction: number) => void;
    /** Min/max fractions (default 0.1 .. 0.9). */
    min?: number;
    max?: number;
  };

  let { orientation, container, onChange, min = 0.1, max = 0.9 }: Props =
    $props();

  let dragging = $state(false);

  function onPointerDown(e: PointerEvent) {
    if (!container) return;
    if (e.button !== 0) return;
    e.preventDefault();
    dragging = true;
    const rect = container.getBoundingClientRect();
    const sizeKey = orientation === "vertical" ? "width" : "height";
    const posKey = orientation === "vertical" ? "left" : "top";
    const cursorKey = orientation === "vertical" ? "col-resize" : "row-resize";

    const move = (ev: PointerEvent) => {
      const cur = orientation === "vertical" ? ev.clientX : ev.clientY;
      const start = (rect as unknown as Record<string, number>)[posKey];
      const span = (rect as unknown as Record<string, number>)[sizeKey];
      let frac = (cur - start) / span;
      frac = Math.max(min, Math.min(max, frac));
      onChange(frac);
    };
    const up = () => {
      dragging = false;
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", up);
      document.body.style.cursor = "";
      document.body.style.userSelect = "";
    };
    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", up);
    document.body.style.cursor = cursorKey;
    document.body.style.userSelect = "none";
  }

  function onDoubleClick() {
    onChange(0.5);
  }
</script>

<div
  class="splitter"
  class:v={orientation === "vertical"}
  class:h={orientation === "horizontal"}
  class:dragging
  onpointerdown={onPointerDown}
  ondblclick={onDoubleClick}
  role="separator"
  aria-orientation={orientation}
></div>

<style>
  .splitter {
    position: relative;
    background: var(--border);
    transition: background 100ms ease;
  }
  .splitter.v {
    width: 1px;
    cursor: col-resize;
  }
  .splitter.h {
    height: 1px;
    cursor: row-resize;
  }
  /* Wider hit-area without changing visual size. */
  .splitter::before {
    content: "";
    position: absolute;
    inset: 0;
  }
  .splitter.v::before {
    inset: 0 -3px;
  }
  .splitter.h::before {
    inset: -3px 0;
  }
  .splitter:hover,
  .splitter.dragging {
    background: var(--accent);
  }
</style>
