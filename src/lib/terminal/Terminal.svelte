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
    } catch {
      // canvas/dom fallback
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

    term.onResize(({ cols, rows }) => {
      pty?.resize(cols, rows).catch(() => {});
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
