<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { Terminal } from "@xterm/xterm";
  import { FitAddon } from "@xterm/addon-fit";
  import { SearchAddon } from "@xterm/addon-search";
  import { Unicode11Addon } from "@xterm/addon-unicode11";
  import "@xterm/xterm/css/xterm.css";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { invoke } from "@tauri-apps/api/core";
  import { spawnPty, type PtyHandle, type ShellInfo } from "./pty-client";
  import { terminals } from "./terminal-store";
  import { xtermTheme } from "./terminal-theme";
  import { settings } from "../settings/settings-store";
  import { get } from "svelte/store";

  const clipboardReadText = (): Promise<string> => invoke<string>("clipboard_read_text");
  const clipboardWriteText = (text: string): Promise<void> =>
    invoke<void>("clipboard_write_text", { text });

  type Props = {
    uid: number;
    shell: ShellInfo;
    cwd: string | null;
    visible: boolean;
  };

  let { uid, shell, cwd, visible }: Props = $props();

  let host: HTMLDivElement;
  let term: Terminal | undefined;
  let fit: FitAddon | undefined;
  let search: SearchAddon | undefined;
  let pty: PtyHandle | undefined;
  let resizeObserver: ResizeObserver | undefined;
  let resizeTimer: ReturnType<typeof setTimeout> | undefined;

  // Context menu state
  let menu = $state<{ x: number; y: number; linkUri: string | null } | null>(null);

  // URL regex used by both link provider and context-menu URL detection
  const URL_REGEX = /(https?:\/\/[^\s<>"'`]+[^\s<>"'`.,;:!?)\]}])/g;

  async function copySelection(): Promise<boolean> {
    const sel = term?.getSelection();
    if (sel && sel.length > 0) {
      try {
        await clipboardWriteText(sel);
        return true;
      } catch {
        // ignored
      }
    }
    return false;
  }

  async function pasteFromClipboard() {
    try {
      const txt = await clipboardReadText();
      if (txt) await pty?.write(txt);
    } catch {
      // ignored — clipboard may be empty or hold non-text
    }
  }

  function urlAtCell(col: number, row: number): string | null {
    if (!term) return null;
    const buf = term.buffer.active;
    const line = buf.getLine(buf.viewportY + row - 1);
    if (!line) return null;
    const text = line.translateToString(true);
    URL_REGEX.lastIndex = 0;
    let m: RegExpExecArray | null;
    while ((m = URL_REGEX.exec(text)) !== null) {
      const start = m.index;
      const end = start + m[0].length;
      if (col - 1 >= start && col - 1 < end) return m[0];
    }
    return null;
  }

  onMount(async () => {
    const s = get(settings);
    term = new Terminal({
      theme: xtermTheme,
      fontFamily: s.terminalFontFamily,
      fontSize: s.terminalFontSize,
      lineHeight: 1.0,
      cursorBlink: true,
      cursorStyle: "bar",
      allowProposedApi: true,
      scrollback: 5000,
      drawBoldTextInBrightColors: false,
      rightClickSelectsWord: false,
    });

    fit = new FitAddon();
    search = new SearchAddon();
    const unicode = new Unicode11Addon();

    term.loadAddon(fit);
    term.loadAddon(search);
    term.loadAddon(unicode);
    term.unicode.activeVersion = "11";

    // Wait for the actual terminal font to load before opening xterm, so cell
    // metrics are measured against the real font (not a fallback). Hard timeout
    // so we never hang the UI if fonts.ready rejects or stalls.
    try {
      if (document.fonts && document.fonts.ready) {
        await Promise.race([
          document.fonts.ready,
          new Promise<void>((r) => setTimeout(r, 250)),
        ]);
      }
    } catch {
      // ignored
    }

    term.open(host);

    // Plain-click link provider — registered *after* open so xterm's link
    // services are initialized.
    try {
      term.registerLinkProvider({
        provideLinks(bufferLineNumber, callback) {
          if (!term) {
            callback(undefined);
            return;
          }
          const line = term.buffer.active.getLine(bufferLineNumber - 1);
          if (!line) {
            callback(undefined);
            return;
          }
          const text = line.translateToString(true);
          const links: {
            range: { start: { x: number; y: number }; end: { x: number; y: number } };
            text: string;
            activate: (event: MouseEvent, uri: string) => void;
          }[] = [];
          URL_REGEX.lastIndex = 0;
          let m: RegExpExecArray | null;
          while ((m = URL_REGEX.exec(text)) !== null) {
            const startCol = m.index + 1;
            const endCol = m.index + m[0].length;
            links.push({
              range: {
                start: { x: startCol, y: bufferLineNumber },
                end: { x: endCol, y: bufferLineNumber },
              },
              text: m[0],
              activate: (_e, uri) => {
                void openUrl(uri).catch(() => {});
              },
            });
          }
          callback(links);
        },
      });
    } catch (e) {
      console.warn("link provider registration failed:", e);
    }

    // Wait for layout so fit() sees the real container height. Without this,
    // host.clientHeight can be 0 at onMount, fit() falls back to xterm's 80x24
    // default, and the shell paints a 24-row banner — leaving an empty band
    // above the prompt once the resize observer trims rows down.
    await new Promise<void>((r) => requestAnimationFrame(() => r()));
    if (host.clientHeight < 4 || host.clientWidth < 4) {
      await new Promise<void>((r) => requestAnimationFrame(() => r()));
    }
    fit.fit();
    const { cols, rows } = term;

    pty = await spawnPty({
      program: shell.program,
      args: shell.args,
      cwd,
      cols,
      rows,
      label: shell.label,
      onData: (data) => term?.write(data),
      onExit: (code) => {
        terminals.markExit(uid, code);
        if (term) {
          term.write(`\r\n\x1b[2m[process exited with code ${code}]\x1b[0m\r\n`);
        }
      },
    });

    term.onData((data) => {
      pty?.write(data).catch(() => {});
    });

    // Auto-copy on selection (mouseup). Matches modern terminal UX.
    term.onSelectionChange(() => {
      // Defer so xterm has a finalized selection
      requestAnimationFrame(() => {
        const sel = term?.getSelection();
        if (sel && sel.length > 0 && !sel.includes("\x00")) {
          void clipboardWriteText(sel).catch(() => {});
        }
      });
    });

    // Debounce SIGWINCH: every dragged row would otherwise call pty.resize,
    // each one nudging bash/readline to redraw the prompt. The result is
    // stacked prompts in the buffer during a drag.
    term.onResize(({ cols, rows }) => {
      if (resizeTimer) clearTimeout(resizeTimer);
      resizeTimer = setTimeout(() => {
        pty?.resize(cols, rows).catch(() => {});
      }, 120);
    });

    resizeObserver = new ResizeObserver(() => {
      try {
        fit?.fit();
      } catch {
        // ignored — happens when the host is detached
      }
    });
    resizeObserver.observe(host);

    // Keyboard: copy/paste/find. Returning false swallows the event.
    term.attachCustomKeyEventHandler((e) => {
      if (e.type !== "keydown") return true;
      const cmd = e.metaKey || e.ctrlKey;
      const key = e.key.toLowerCase();

      if (cmd && key === "f") {
        e.preventDefault();
        const q = window.prompt("Find in terminal:");
        if (q) search?.findNext(q);
        return false;
      }

      // Ctrl+C: copy if there's a selection, otherwise let it through as SIGINT.
      // Cmd+C on macOS: always copy (no SIGINT collision).
      if (cmd && key === "c") {
        const sel = term?.getSelection();
        if (sel && sel.length > 0) {
          void clipboardWriteText(sel).catch(() => {});
          term?.clearSelection();
          e.preventDefault();
          return false;
        }
        // No selection → fall through so Ctrl+C reaches the shell as ETX.
        return true;
      }

      // Ctrl+V / Cmd+V → paste
      if (cmd && key === "v") {
        e.preventDefault();
        void pasteFromClipboard();
        return false;
      }

      // Shift+Insert → paste (Linux convention)
      if (e.shiftKey && e.key === "Insert") {
        e.preventDefault();
        void pasteFromClipboard();
        return false;
      }

      // Ctrl+Shift+C / Ctrl+Shift+V (xterm convention)
      if (cmd && e.shiftKey && key === "c") {
        e.preventDefault();
        void copySelection();
        return false;
      }
      if (cmd && e.shiftKey && key === "v") {
        e.preventDefault();
        void pasteFromClipboard();
        return false;
      }

      // Ctrl+A → select all (when no shell needs it) — skip; let shell handle.
      return true;
    });
  });

  function onContextMenu(e: MouseEvent) {
    e.preventDefault();
    if (!term) return;

    // Try to map the click to a buffer cell to detect a URL under the cursor.
    let linkUri: string | null = null;
    try {
      const rect = host.getBoundingClientRect();
      const xtermEl = host.querySelector(".xterm-rows") as HTMLElement | null;
      if (xtermEl) {
        const cellWidth = xtermEl.clientWidth / term.cols;
        const cellHeight = xtermEl.clientHeight / term.rows;
        const col = Math.floor((e.clientX - rect.left) / cellWidth) + 1;
        const row = Math.floor((e.clientY - rect.top) / cellHeight) + 1;
        linkUri = urlAtCell(col, row);
      }
    } catch {
      // ignored
    }

    menu = { x: e.clientX, y: e.clientY, linkUri };
  }

  function closeMenu() {
    menu = null;
  }

  async function menuCopy() {
    await copySelection();
    closeMenu();
  }
  async function menuPaste() {
    await pasteFromClipboard();
    closeMenu();
  }
  function menuSelectAll() {
    term?.selectAll();
    closeMenu();
  }
  function menuClear() {
    term?.clear();
    closeMenu();
  }
  function menuOpenLink() {
    if (menu?.linkUri) void openUrl(menu.linkUri).catch(() => {});
    closeMenu();
  }
  async function menuCopyLink() {
    if (menu?.linkUri) {
      try {
        await clipboardWriteText(menu.linkUri);
      } catch {
        // ignored
      }
    }
    closeMenu();
  }

  $effect(() => {
    if (visible && term && fit) {
      requestAnimationFrame(() => {
        try {
          fit?.fit();
          term?.focus();
        } catch {
          // ignored
        }
      });
    }
  });

  $effect(() => {
    const fam = $settings.terminalFontFamily;
    const size = $settings.terminalFontSize;
    if (!term) return;
    term.options.fontFamily = fam;
    term.options.fontSize = size;
    try {
      fit?.fit();
    } catch {
      // ignored
    }
  });

  onDestroy(() => {
    resizeObserver?.disconnect();
    if (resizeTimer) clearTimeout(resizeTimer);
    pty?.kill().catch(() => {});
    term?.dispose();
  });
</script>

<svelte:window onclick={closeMenu} onkeydown={(e) => e.key === "Escape" && closeMenu()} />

<div class="term" bind:this={host} oncontextmenu={onContextMenu} role="presentation"></div>

{#if menu}
  <div
    class="ctx"
    style="left: {menu.x}px; top: {menu.y}px"
    role="menu"
    tabindex="-1"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.stopPropagation()}
  >
    {#if menu.linkUri}
      <button class="ci" onclick={menuOpenLink}>Open link</button>
      <button class="ci" onclick={menuCopyLink}>Copy link address</button>
      <div class="sep"></div>
    {/if}
    <button class="ci" onclick={menuCopy}>Copy</button>
    <button class="ci" onclick={menuPaste}>Paste</button>
    <div class="sep"></div>
    <button class="ci" onclick={menuSelectAll}>Select all</button>
    <button class="ci" onclick={menuClear}>Clear</button>
  </div>
{/if}

<style>
  .term {
    height: 100%;
    width: 100%;
    background: var(--bg);
  }
  :global(.xterm) {
    height: 100% !important;
  }
  :global(.xterm-viewport) {
    background-color: transparent !important;
  }
  /* Make link hover obvious */
  :global(.xterm a) {
    cursor: pointer;
  }
  :global(.xterm .xterm-link) {
    cursor: pointer;
    text-decoration: underline;
  }

  .ctx {
    position: fixed;
    z-index: 1000;
    background: var(--bg-3);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 4px;
    min-width: 180px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5);
    font-size: 12px;
  }
  .ci {
    display: block;
    width: 100%;
    text-align: left;
    padding: 6px 10px;
    border-radius: 4px;
    color: var(--fg-dim);
    background: transparent;
    border: 0;
    cursor: pointer;
  }
  .ci:hover {
    background: var(--bg-hover);
    color: var(--fg);
  }
  .sep {
    height: 1px;
    background: var(--border);
    margin: 4px 2px;
  }
</style>
