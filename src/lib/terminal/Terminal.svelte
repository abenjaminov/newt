<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { Terminal } from "@xterm/xterm";
  import { FitAddon } from "@xterm/addon-fit";
  import { WebLinksAddon } from "@xterm/addon-web-links";
  import { SearchAddon } from "@xterm/addon-search";
  import { Unicode11Addon } from "@xterm/addon-unicode11";
  import { WebglAddon } from "@xterm/addon-webgl";
  import "@xterm/xterm/css/xterm.css";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { spawnPty, type PtyHandle, type ShellInfo } from "./pty-client";
  import { terminals } from "./terminal-store";
  import { xtermTheme } from "./terminal-theme";
  import { settings } from "../settings/settings-store";
  import { get } from "svelte/store";

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

  onMount(async () => {
    const s = get(settings);
    term = new Terminal({
      theme: xtermTheme,
      fontFamily: s.terminalFontFamily,
      fontSize: s.terminalFontSize,
      lineHeight: 1.2,
      cursorBlink: true,
      cursorStyle: "bar",
      allowProposedApi: true,
      scrollback: 5000,
      drawBoldTextInBrightColors: false,
    });

    fit = new FitAddon();
    search = new SearchAddon();
    const unicode = new Unicode11Addon();
    const links = new WebLinksAddon((_e, uri) => {
      void openUrl(uri).catch(() => {});
    });

    term.loadAddon(fit);
    term.loadAddon(search);
    term.loadAddon(unicode);
    term.loadAddon(links);
    term.unicode.activeVersion = "11";

    term.open(host);

    // WebGL renderer is a big perf win; fall back gracefully.
    try {
      const webgl = new WebglAddon();
      webgl.onContextLoss(() => webgl.dispose());
      term.loadAddon(webgl);
    } catch (e) {
      console.warn("xterm WebGL renderer unavailable, using canvas/DOM fallback:", e);
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

    // Debounce SIGWINCH: every dragged row would otherwise call pty.resize,
    // each one nudging bash/readline to redraw the prompt. The result is
    // stacked prompts in the buffer during a drag.
    term.onResize(({ cols, rows }) => {
      if (resizeTimer) clearTimeout(resizeTimer);
      resizeTimer = setTimeout(() => {
        pty?.resize(cols, rows).catch(() => {});
      }, 120);
    });

    // Watch for container size changes.
    resizeObserver = new ResizeObserver(() => {
      try {
        fit?.fit();
      } catch {
        // ignored — happens when the host is detached
      }
    });
    resizeObserver.observe(host);

    // Find / Cmd-F support.
    term.attachCustomKeyEventHandler((e) => {
      if (e.type !== "keydown") return true;
      const cmd = e.metaKey || e.ctrlKey;
      if (cmd && e.key.toLowerCase() === "f") {
        e.preventDefault();
        const q = window.prompt("Find in terminal:");
        if (q) search?.findNext(q);
        return false;
      }
      return true;
    });
  });

  $effect(() => {
    if (visible && term && fit) {
      // Defer to next frame so layout has settled.
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

  // Live-update terminal font when settings change.
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

<div class="term" bind:this={host}></div>

<style>
  .term {
    height: 100%;
    width: 100%;
    background: var(--bg);
    padding: 6px 8px 0 8px;
  }
  :global(.xterm) {
    height: 100% !important;
  }
  :global(.xterm-viewport) {
    background-color: transparent !important;
  }
</style>
