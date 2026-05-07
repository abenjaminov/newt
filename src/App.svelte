<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onDestroy, onMount } from "svelte";
  import ActivityBar from "./lib/activity-bar/ActivityBar.svelte";
  import Welcome from "./lib/welcome/Welcome.svelte";
  import FileTree from "./lib/filetree/FileTree.svelte";
  import Editor from "./lib/editor/Editor.svelte";
  import EditorTabs from "./lib/editor/EditorTabs.svelte";
  import Breadcrumbs from "./lib/editor/Breadcrumbs.svelte";
  import ImageView from "./lib/editor/ImageView.svelte";
  import { treeStore, type TreeNode } from "./lib/filetree/tree-store.svelte";
  import MarkdownPreview from "./lib/markdown/MarkdownPreview.svelte";
  import Splitter from "./lib/common/Splitter.svelte";
  import TerminalPanel from "./lib/terminal/TerminalPanel.svelte";
  import GitPanel from "./lib/git/GitPanel.svelte";
  import SearchPanel from "./lib/search/SearchPanel.svelte";
  import DiffView from "./lib/git/DiffView.svelte";
  import { refreshIgnored } from "./lib/git/git-store";
  import ProcessesPanel from "./lib/processes/ProcessesPanel.svelte";
  import WorktreeCreateModal from "./lib/worktree/WorktreeCreateModal.svelte";
  import Footer from "./lib/footer/Footer.svelte";
  import SettingsView from "./lib/settings/SettingsView.svelte";
  import AppMenu from "./lib/menu/AppMenu.svelte";
  import WindowControls from "./lib/menu/WindowControls.svelte";
  import NamePrompt from "./lib/common/NamePrompt.svelte";
  import { loadSettings, settings, updateSettings } from "./lib/settings/settings-store";
  import CommandPalette from "./lib/palette/CommandPalette.svelte";
  import OutlinePalette from "./lib/palette/OutlinePalette.svelte";
  import {
    commandRegistry,
    paletteOpen,
    refreshFileIndex,
    type Command,
  } from "./lib/palette/palette-store";
  import {
    eventToChord,
    findCommandForChord,
    isEditableTarget,
    keymap,
    loadKeymap,
  } from "./lib/keybindings/keybinding-store";
  import { get } from "svelte/store";
  import { pickAndOpenFolder } from "./lib/workspace/open-workspace";
  import {
    refreshWorktrees,
    activeWorktreePath,
    pathExists,
    joinPath,
    relativeTo,
    type Worktree,
  } from "./lib/worktree/worktree-store";
  import { isMarkdown } from "./lib/editor/languages";
  import { tabs, activeTab, popRecentlyClosed } from "./lib/editor/tabs-store";
  import { openFileAtPath } from "./lib/editor/open-file";
  import { workspace, activePanel } from "./lib/workspace/workspace-store";

  let saveError = $state<string | null>(null);
  let unlistenFs: UnlistenFn | undefined;
  let showPreview = $state(true);
  let showCreateWorktree = $state(false);
  function openSettings() {
    tabs.openSettings();
  }
  let sidePanelCollapsed = $state(false);
  let terminalCollapsed = $state(false);
  let mdSplitFrac = $state(0.5);
  let editorHostEl = $state<HTMLDivElement | undefined>(undefined);
  let editorScrollPct = $state(0);
  let terminalH = $state(220);
  let sidePanelW = $state(260);
  let shellEl = $state<HTMLDivElement | undefined>(undefined);
  let searchPanelEl = $state<SearchPanel | undefined>(undefined);
  let showOutline = $state(false);

  function openSearchPanel() {
    activePanel.set("search");
    sidePanelCollapsed = false;
    queueMicrotask(() => searchPanelEl?.focusInput());
  }

  function startTerminalResize(e: PointerEvent) {
    if (e.button !== 0) return;
    e.preventDefault();
    const shellRect = shellEl?.getBoundingClientRect();
    const shellBottom = shellRect ? shellRect.bottom : window.innerHeight;
    const shellTop = shellRect ? shellRect.top : 0;
    const STATUS_BAR_H = 24;
    const startY = e.clientY;
    // Offset between cursor and visible boundary at click time (≤3px in
    // either direction inside the 6px hitbox). We preserve this offset for
    // the first move (no click-jump), then linearly decay it to 0 over the
    // first 24px of movement so subsequent dragging tracks 1:1.
    const startBoundaryY = shellBottom - terminalH - STATUS_BAR_H;
    const initialOffset = e.clientY - startBoundaryY;
    const target = e.currentTarget as HTMLElement;
    target.setPointerCapture(e.pointerId);
    const move = (ev: PointerEvent) => {
      const moveDist = Math.abs(ev.clientY - startY);
      const offsetWeight = Math.max(0, 1 - moveDist / 24);
      const currentOffset = initialOffset * offsetWeight;
      const proposed = shellBottom - (ev.clientY - currentOffset) - STATUS_BAR_H;
      const maxH = (shellBottom - shellTop) * 0.85;
      terminalH = Math.max(60, Math.min(maxH, proposed));
    };
    const up = (ev: PointerEvent) => {
      target.removeEventListener("pointermove", move);
      target.removeEventListener("pointerup", up);
      try {
        target.releasePointerCapture(ev.pointerId);
      } catch {
        // ignored
      }
      document.body.style.cursor = "";
      document.body.style.userSelect = "";
    };
    target.addEventListener("pointermove", move);
    target.addEventListener("pointerup", up);
    document.body.style.cursor = "row-resize";
    document.body.style.userSelect = "none";
  }

  function startSidePanelResize(e: PointerEvent) {
    if (e.button !== 0) return;
    e.preventDefault();
    const activityW =
      parseFloat(
        getComputedStyle(document.documentElement).getPropertyValue("--activity-bar-w"),
      ) || 44;
    const startX = e.clientX;
    const startBoundaryX = activityW + sidePanelW;
    const initialOffset = e.clientX - startBoundaryX;
    const target = e.currentTarget as HTMLElement;
    target.setPointerCapture(e.pointerId);
    const move = (ev: PointerEvent) => {
      const moveDist = Math.abs(ev.clientX - startX);
      const offsetWeight = Math.max(0, 1 - moveDist / 24);
      const currentOffset = initialOffset * offsetWeight;
      const proposed = ev.clientX - currentOffset - activityW;
      sidePanelW = Math.max(160, Math.min(window.innerWidth * 0.6, proposed));
    };
    const up = (ev: PointerEvent) => {
      target.removeEventListener("pointermove", move);
      target.removeEventListener("pointerup", up);
      try {
        target.releasePointerCapture(ev.pointerId);
      } catch {
        // ignored
      }
      document.body.style.cursor = "";
      document.body.style.userSelect = "";
    };
    target.addEventListener("pointermove", move);
    target.addEventListener("pointerup", up);
    document.body.style.cursor = "col-resize";
    document.body.style.userSelect = "none";
  }

  function resetTerminalSize() {
    terminalH = 220;
  }
  function resetSidePanelSize() {
    sidePanelW = 260;
  }

  let namePrompt = $state<{
    title: string;
    confirmLabel: string;
    onConfirm: (name: string) => Promise<void>;
  } | null>(null);
  let createError = $state<string | null>(null);

  function newAtSelection(isDir: boolean) {
    if (!$workspace) return;
    namePrompt = {
      title: isDir ? "New folder" : "New file",
      confirmLabel: "Create",
      onConfirm: async (name) => {
        await createInWorkspace(isDir, name);
      },
    };
  }

  async function createInWorkspace(isDir: boolean, name: string) {
    if (!$workspace) return;
    const sep =
      $workspace.rootPath.includes("\\") && !$workspace.rootPath.includes("/")
        ? "\\"
        : "/";

    let parentPath = $workspace.rootPath;
    const sel = treeStore.selected;
    if (sel) {
      const node = findInTree(treeStore.root, sel);
      if (node) {
        if (node.is_dir) parentPath = node.path;
        else
          parentPath =
            sel
              .replace(/[\\/]+$/, "")
              .split(/[\\/]/)
              .slice(0, -1)
              .join(sep) || $workspace.rootPath;
      }
    }
    const newPath = `${parentPath.replace(/[\\/]+$/, "")}${sep}${name}`;
    try {
      if (isDir) await invoke("create_dir", { path: newPath });
      else await invoke("create_file", { path: newPath });
      const pNode = findInTree(treeStore.root, parentPath);
      if (pNode && pNode.is_dir && !pNode.loaded) {
        const entries = await invoke<
          { name: string; path: string; is_dir: boolean }[]
        >("read_dir", {
          path: parentPath,
          showHidden: $settings.fileTreeShowHidden,
          respectGitignore: !$settings.fileTreeShowHidden,
        });
        pNode.children = entries.map((e) => ({
          ...e,
          expanded: false,
          loaded: false,
        }));
        pNode.loaded = true;
        pNode.expanded = true;
      } else {
        treeStore.insertEntry(parentPath, { name, path: newPath, is_dir: isDir });
        const pn = findInTree(treeStore.root, parentPath);
        if (pn) pn.expanded = true;
      }
      treeStore.selected = newPath;
      if (!isDir) {
        const content = await invoke<string>("read_file", { path: newPath });
        tabs.open({ path: newPath, kind: "text", content });
      }
      namePrompt = null;
    } catch (e) {
      createError = String(e);
    }
  }

  function nameValidator(name: string): string | null {
    if (!name) return null;
    if (name.includes("/") || name.includes("\\"))
      return "Name can't contain / or \\";
    if (name === "." || name === "..") return "Reserved name";
    return null;
  }

  function findInTree(
    n: TreeNode | null,
    path: string,
  ): TreeNode | null {
    if (!n) return null;
    if (n.path === path) return n;
    if (!n.children) return null;
    for (const c of n.children) {
      const r = findInTree(c, path);
      if (r) return r;
    }
    return null;
  }

  // Maximize window on first load + load persisted settings & keymap.
  let didMaximize = false;
  onMount(async () => {
    void loadSettings();
    void loadKeymap();
    if (!didMaximize) {
      didMaximize = true;
      try {
        const win = getCurrentWindow();
        if (!(await win.isMaximized())) {
          await win.maximize();
        }
      } catch {
        // running in browser preview without Tauri
      }
    }
    window.addEventListener("keydown", handleKey);
    window.addEventListener("keyup", handleKeyUp);
    window.addEventListener("blur", onWindowBlur);
  });
  onDestroy(() => {
    window.removeEventListener("keydown", handleKey);
    window.removeEventListener("keyup", handleKeyUp);
    window.removeEventListener("blur", onWindowBlur);
  });

  let lastShift = 0;
  function handleKey(e: KeyboardEvent) {
    // Ctrl+Shift+P always opens the palette (independent of keymap).
    if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key.toLowerCase() === "p") {
      e.preventDefault();
      paletteOpen.set(true);
      return;
    }

    // While the palette is open, let it own the keyboard.
    if (get(paletteOpen)) return;

    const chord = eventToChord(e);
    if (!chord) return;

    // For text-input targets, only fire chords with a modifier — never
    // hijack plain typing.
    if (isEditableTarget(e.target) && !(e.ctrlKey || e.metaKey || e.altKey)) {
      return;
    }

    const cid = findCommandForChord(chord, get(keymap));
    if (!cid) return;
    const cmd = get(commandRegistry).find((c) => c.id === cid);
    if (!cmd) return;
    e.preventDefault();
    void cmd.run();
  }

  function handleKeyUp(e: KeyboardEvent) {
    if (e.key !== "Shift") return;
    const target = e.target as HTMLElement | null;
    const inField =
      target?.tagName === "INPUT" ||
      target?.tagName === "TEXTAREA" ||
      target?.isContentEditable;
    if (inField) return;
    const now = Date.now();
    if (now - lastShift > 30 && now - lastShift < 350) {
      paletteOpen.set(true);
      lastShift = 0;
    } else {
      lastShift = now;
    }
  }

  function toggleSidePanel(target: typeof $activePanel) {
    if ($activePanel === target && !sidePanelCollapsed) {
      sidePanelCollapsed = true;
    } else {
      sidePanelCollapsed = false;
      activePanel.set(target);
      if (target === "search") {
        queueMicrotask(() => searchPanelEl?.focusInput());
      }
    }
  }

  // Per-worktree tab snapshots: { [worktreePath]: { tabs, activePath } }
  const perTreeTabs = new Map<string, ReturnType<typeof tabs.snapshot>>();
  let lastWorktree: string | null = null;

  // Refresh worktrees + file index + ignored-paths cache when workspace changes.
  $effect(() => {
    const ws = $workspace;
    if (ws) {
      void refreshWorktrees(ws.rootPath);
      void refreshFileIndex(ws.rootPath);
      void refreshIgnored(ws.rootPath);
    } else {
      void refreshWorktrees(null);
      void refreshFileIndex(null);
      void refreshIgnored(null);
    }
  });

  async function pickFolder() {
    await pickAndOpenFolder();
  }

  // Build command list. Re-evaluated whenever the user opens the palette
  // (the registry is published via a writable store).
  $effect(() => {
    const cmds: Command[] = [];

    cmds.push(
      {
        id: "settings.open",
        title: "Open Settings",
        group: "Settings",
        hint: "Ctrl+,",
        run: openSettings,
      },
      {
        id: "settings.zoomIn",
        title: "Zoom In",
        group: "Settings",
        run: () =>
          updateSettings({ uiZoom: Math.min(2, +($settings.uiZoom + 0.1).toFixed(2)) }),
      },
      {
        id: "settings.zoomOut",
        title: "Zoom Out",
        group: "Settings",
        run: () =>
          updateSettings({ uiZoom: Math.max(0.5, +($settings.uiZoom - 0.1).toFixed(2)) }),
      },
      {
        id: "settings.zoomReset",
        title: "Reset Zoom to 100%",
        group: "Settings",
        run: () => updateSettings({ uiZoom: 1 }),
      },
      {
        id: "settings.toggleWrap",
        title: $settings.editorLineWrap ? "Disable Line Wrap" : "Enable Line Wrap",
        group: "Settings",
        run: () => updateSettings({ editorLineWrap: !$settings.editorLineWrap }),
      },
      {
        id: "settings.editorBigger",
        title: "Increase Editor Font Size",
        group: "Settings",
        run: () =>
          updateSettings({ editorFontSize: Math.min(28, $settings.editorFontSize + 1) }),
      },
      {
        id: "settings.editorSmaller",
        title: "Decrease Editor Font Size",
        group: "Settings",
        run: () =>
          updateSettings({ editorFontSize: Math.max(9, $settings.editorFontSize - 1) }),
      },
    );

    cmds.push(
      {
        id: "view.files",
        title: "Show File Tree",
        group: "View",
        run: () => {
          activePanel.set("files");
          sidePanelCollapsed = false;
        },
      },
      {
        id: "view.search",
        title: "Search in Files",
        group: "View",
        hint: "Ctrl+Shift+F",
        run: openSearchPanel,
      },
      {
        id: "view.outline",
        title: "Go to Symbol in File…",
        group: "View",
        hint: "Ctrl+Shift+O",
        run: () => (showOutline = true),
      },
      {
        id: "view.git",
        title: "Show Git Changes",
        group: "View",
        run: () => {
          activePanel.set("git");
          sidePanelCollapsed = false;
        },
      },
      {
        id: "view.processes",
        title: "Show Processes",
        group: "View",
        run: () => {
          activePanel.set("processes");
          sidePanelCollapsed = false;
        },
      },
      {
        id: "view.toggleSidebar",
        title: "Toggle Sidebar",
        group: "View",
        run: () => (sidePanelCollapsed = !sidePanelCollapsed),
      },
      {
        id: "view.toggleTerminal",
        title: "Toggle Terminal",
        group: "View",
        run: () => (terminalCollapsed = !terminalCollapsed),
      },
    );

    cmds.push({
      id: "file.reopenClosed",
      title: "Reopen Closed Tab",
      group: "File",
      hint: "Ctrl+Shift+T",
      run: async () => {
        const p = popRecentlyClosed();
        if (p) await openFileAtPath(p);
      },
    });
    if ($activeTab) {
      cmds.push(
        {
          id: "file.save",
          title: "Save File",
          group: "File",
          hint: "Ctrl+S",
          run: () => saveActive(),
        },
        {
          id: "file.close",
          title: "Close Tab",
          group: "File",
          run: () => tabs.close($activeTab!.path),
        },
      );
      if ($activeTab.kind === "svg") {
        cmds.push({
          id: "file.svgToggle",
          title: $activeTab.viewMode === "image" ? "View SVG Source" : "View SVG Image",
          group: "File",
          run: () =>
            tabs.setViewMode(
              $activeTab!.path,
              $activeTab!.viewMode === "image" ? "text" : "image",
            ),
        });
      }
    }

    cmds.push(
      {
        id: "action.openFolder",
        title: "Open Folder…",
        group: "Action",
        hint: "Ctrl+O",
        run: () => pickFolder(),
      },
    );
    if ($workspace) {
      cmds.push({
        id: "action.closeFolder",
        title: "Close Folder",
        group: "Action",
        run: closeWorkspace,
      });
    }

    if ($workspace) {
      cmds.push({
        id: "worktree.create",
        title: "New Worktree…",
        group: "Worktree",
        run: () => (showCreateWorktree = true),
      });
    }

    commandRegistry.set(cmds);
  });

  // Track active worktree changes for save/restore of per-tree tab state.
  $effect(() => {
    const wt = $activeWorktreePath;
    if (wt === lastWorktree) return;
    if (lastWorktree !== null) {
      perTreeTabs.set(lastWorktree, tabs.snapshot());
    }
    lastWorktree = wt;
    if (wt && perTreeTabs.has(wt)) {
      tabs.restore(perTreeTabs.get(wt)!);
    }
  });

  async function handleWorktreeSwitch(target: Worktree) {
    if (!$workspace) return;
    const oldRoot = $workspace.rootPath;

    // Snapshot current tabs (in case they include unsaved changes — we keep them
    // re-pointed; save warnings are user's responsibility for now).
    const snap = tabs.snapshot();

    // Update workspace root to the new worktree path. Side effects: file tree
    // reloads, git panel refreshes, terminal cwd updates for new shells.
    workspace.set({
      rootPath: target.path,
      rootName: target.path.split(/[\\/]/).filter(Boolean).pop() ?? target.path,
    });

    // For each currently-open tab, decide: re-point, mark foreign, or leave.
    for (const t of snap.tabs) {
      const rel = relativeTo(oldRoot, t.path);
      if (rel === null) {
        // Tab is outside the old worktree; leave it alone.
        continue;
      }
      const candidate = joinPath(target.path, rel);
      let exists = false;
      try {
        exists = await pathExists(candidate);
      } catch {
        exists = false;
      }
      if (exists && !t.dirty) {
        try {
          if (t.kind === "image") {
            const info = await invoke<{ data_url: string; size: number }>(
              "read_image",
              { path: candidate },
            );
            tabs.replace(t.path, {
              path: candidate,
              kind: "image",
              content: "",
              imageDataUrl: info.data_url,
              imageSize: info.size,
              foreignWorktree: null,
            });
          } else if (t.kind === "svg") {
            const [content, info] = await Promise.all([
              invoke<string>("read_file", { path: candidate }),
              invoke<{ data_url: string; size: number }>("read_image", {
                path: candidate,
              }),
            ]);
            tabs.replace(t.path, {
              path: candidate,
              kind: "svg",
              content,
              imageDataUrl: info.data_url,
              imageSize: info.size,
              foreignWorktree: null,
            });
          } else {
            const next = await invoke<string>("read_file", { path: candidate });
            tabs.replace(t.path, {
              path: candidate,
              kind: "text",
              content: next,
              foreignWorktree: null,
            });
          }
        } catch {
          tabs.markForeign(t.path, target.branch ?? null);
        }
      } else if (exists && t.dirty) {
        // Don't clobber unsaved edits — leave on old path with foreign chip
        // (regardless of policy; closing dirty tabs would lose work).
        tabs.markForeign(t.path, target.branch ?? null);
      } else {
        // File doesn't exist in new worktree — apply foreign-tabs policy.
        const policy = $settings.worktreeForeignTabs;
        if (policy === "close") {
          tabs.close(t.path);
        } else if (policy === "keepActive") {
          // leave silently — no chip, no close
        } else {
          tabs.markForeign(t.path, target.branch ?? null);
        }
      }
    }
  }

  const isMd = $derived(
    $activeTab && $activeTab.kind !== "diff" && $activeTab.kind !== "settings"
      ? isMarkdown($activeTab.name)
      : false,
  );
  const showingDiff = $derived($activeTab?.kind === "diff");
  const showingSettings = $derived($activeTab?.kind === "settings");
  const showingImage = $derived(
    $activeTab !== null &&
      ($activeTab.kind === "image" ||
        ($activeTab.kind === "svg" && $activeTab.viewMode === "image")),
  );

  function closeWorkspace() {
    workspace.set(null);
    tabs.closeAll();
    void invoke("stop_watch").catch(() => {});
  }

  // Start/stop file watcher when workspace changes.
  $effect(() => {
    const ws = $workspace;
    if (!ws) return;
    void invoke("start_watch", { path: ws.rootPath }).catch(() => {});
  });

  // Listen for fs:changed events; reload tabs whose disk content changed.
  (async () => {
    unlistenFs = await listen<string[]>("fs:changed", async (e) => {
      const changed = new Set(e.payload);
      for (const t of $tabs.tabs) {
        if (!changed.has(t.path)) continue;
        if (t.dirty) continue; // don't clobber unsaved edits
        try {
          if (t.kind === "image") {
            const info = await invoke<{ data_url: string; size: number }>(
              "read_image",
              { path: t.path },
            );
            tabs.replace(t.path, {
              path: t.path,
              kind: "image",
              content: "",
              imageDataUrl: info.data_url,
              imageSize: info.size,
              foreignWorktree: t.foreignWorktree,
            });
          } else if (t.kind === "svg") {
            const [content, info] = await Promise.all([
              invoke<string>("read_file", { path: t.path }),
              invoke<{ data_url: string; size: number }>("read_image", {
                path: t.path,
              }),
            ]);
            tabs.replace(t.path, {
              path: t.path,
              kind: "svg",
              content,
              imageDataUrl: info.data_url,
              imageSize: info.size,
              foreignWorktree: t.foreignWorktree,
            });
          } else {
            const next = await invoke<string>("read_file", { path: t.path });
            if (next !== t.content) {
              tabs.edit(t.path, next);
              tabs.markSaved(t.path);
            }
          }
        } catch {
          // file may have been deleted; ignore
        }
      }
    });
  })();

  onDestroy(() => unlistenFs?.());

  function extOf(path: string): string {
    const base = path.split(/[\\/]/).pop() ?? "";
    const dot = base.lastIndexOf(".");
    return dot > 0 ? base.slice(dot + 1).toLowerCase() : "";
  }

  async function maybeFormat(path: string): Promise<boolean> {
    if (!$settings.formatOnSave) return false;
    const ext = extOf(path);
    const command = $settings.formatters[ext];
    if (!command || !command.trim()) return false;
    try {
      const res = await invoke<{ ok: boolean }>("run_formatter", { command, file: path });
      return res.ok;
    } catch {
      return false;
    }
  }

  async function reloadFromDisk(path: string) {
    const t = get(tabs).tabs.find((tt) => tt.path === path);
    if (!t || t.kind !== "text") return;
    try {
      const next = await invoke<string>("read_file", { path });
      if (next !== t.content) {
        tabs.edit(path, next);
      }
      tabs.markSaved(path);
    } catch {
      // ignored
    }
  }

  async function saveActive() {
    const t = $activeTab;
    if (!t || !t.dirty) return;
    try {
      await invoke("write_file", { path: t.path, content: t.content });
      tabs.markSaved(t.path);
      saveError = null;
      if (await maybeFormat(t.path)) {
        await reloadFromDisk(t.path);
      }
    } catch (e) {
      saveError = String(e);
    }
  }

  async function saveByPath(path: string) {
    const t = get(tabs).tabs.find((tt) => tt.path === path);
    if (!t || !t.dirty || t.kind !== "text" && t.kind !== "svg") return;
    try {
      await invoke("write_file", { path, content: t.content });
      tabs.markSaved(path);
      if (await maybeFormat(path)) {
        await reloadFromDisk(path);
      }
    } catch {
      // ignored — manual save will surface error
    }
  }

  // Debounced auto-save (mode === "afterDelay").
  let autoSaveTimer: ReturnType<typeof setTimeout> | undefined;
  function scheduleAutoSave() {
    if ($settings.editorAutoSave !== "afterDelay") return;
    if (!$activeTab) return;
    const path = $activeTab.path;
    if (autoSaveTimer) clearTimeout(autoSaveTimer);
    autoSaveTimer = setTimeout(
      () => void saveByPath(path),
      $settings.editorAutoSaveDelayMs,
    );
  }

  // On focus change between tabs: save the previous active tab.
  let prevActivePath: string | null = null;
  $effect(() => {
    const cur = $tabs.activePath;
    if (cur === prevActivePath) return;
    if (
      $settings.editorAutoSave === "onFocusChange" &&
      prevActivePath !== null
    ) {
      const prevPath = prevActivePath;
      void saveByPath(prevPath);
    }
    prevActivePath = cur;
  });

  // Save dirty tabs when window blurs (also "onFocusChange" mode).
  function onWindowBlur() {
    if ($settings.editorAutoSave !== "onFocusChange") return;
    for (const t of get(tabs).tabs) {
      if (t.dirty) void saveByPath(t.path);
    }
  }
</script>

{#if !$workspace}
  <div class="welcome-shell">
    <header class="top-bar welcome-bar">
      <div class="left"></div>
      <div class="title-center"></div>
      <div class="right">
        <WindowControls />
      </div>
    </header>
    <Welcome />
  </div>
{:else}
  <div
    class="shell"
    bind:this={shellEl}
    style:--side-col-w="{sidePanelCollapsed ? '0px' : sidePanelW + 'px'}"
    style:--bottom-row-h="{terminalCollapsed ? '28px' : terminalH + 'px'}"
  >
    <header class="top-bar">
      <div class="left">
        <AppMenu
          onOpenSettings={openSettings}
          onCreateWorktree={() => (showCreateWorktree = true)}
          onCloseWorkspace={closeWorkspace}
          onSaveActive={saveActive}
          onToggleSidebar={() => (sidePanelCollapsed = !sidePanelCollapsed)}
          onToggleTerminal={() => (terminalCollapsed = !terminalCollapsed)}
        />
      </div>
      <div class="title-center">
        <span class="project">{$workspace.rootName}</span>
      </div>
      <div class="right">
        <WindowControls />
      </div>
    </header>

    <div class="body" class:side-collapsed={sidePanelCollapsed}>
      <ActivityBar
        active={$activePanel}
        collapsed={sidePanelCollapsed}
        onSelect={toggleSidePanel}
        onOpenSettings={openSettings}
      />

      <aside class="side-panel" class:hidden={sidePanelCollapsed}>
        <div class="panel-header">
          <span>{$activePanel}</span>
          <div class="header-actions">
            {#if $activePanel === "files" && $workspace}
              <button
                class="header-btn"
                title="New File"
                onclick={() => newAtSelection(false)}
              >
                <span class="hb-glyph">+📄</span>
              </button>
              <button
                class="header-btn"
                title="New Folder"
                onclick={() => newAtSelection(true)}
              >
                <span class="hb-glyph">+📁</span>
              </button>
              <button
                class="header-btn"
                class:on={$settings.fileTreeShowHidden}
                title={$settings.fileTreeShowHidden
                  ? "Hide ignored files (.gitignore)"
                  : "Show ignored files"}
                onclick={() =>
                  updateSettings({
                    fileTreeShowHidden: !$settings.fileTreeShowHidden,
                  })}
              >
                <span class="hb-glyph">{$settings.fileTreeShowHidden ? "👁" : "⦰"}</span>
              </button>
            {/if}
            <button
              class="collapse-btn"
              title="Collapse panel"
              onclick={() => (sidePanelCollapsed = true)}
            >
              ◀
            </button>
          </div>
        </div>
        <div class="panel-body">
          <div class="panel-slot" class:active={$activePanel === "files"}>
            <FileTree />
          </div>
          <div class="panel-slot" class:active={$activePanel === "search"}>
            <SearchPanel bind:this={searchPanelEl} />
          </div>
          <div class="panel-slot" class:active={$activePanel === "git"}>
            <GitPanel />
          </div>
          <div class="panel-slot" class:active={$activePanel === "processes"}>
            <ProcessesPanel visible={$activePanel === "processes" && !sidePanelCollapsed} />
          </div>
        </div>
        <!-- side-panel resize handle disabled while we sort out drag UX -->
        <!-- <div
          class="side-resize"
          onpointerdown={startSidePanelResize}
          ondblclick={resetSidePanelSize}
          role="separator"
          aria-orientation="vertical"
          title="Drag to resize · double-click to reset"
        ></div> -->
      </aside>

      <main class="editor-area">
        <div class="tab-row">
          <EditorTabs />
          {#if $activeTab?.kind === "svg"}
            <button
              class="view-toggle"
              title={$activeTab.viewMode === "image" ? "View source" : "View image"}
              onclick={() =>
                tabs.setViewMode(
                  $activeTab!.path,
                  $activeTab!.viewMode === "image" ? "text" : "image",
                )}
            >
              {$activeTab.viewMode === "image" ? "Source" : "Image"}
            </button>
          {/if}
          {#if isMd}
            <button
              class="view-toggle"
              class:active={showPreview}
              title={showPreview ? "Hide markdown preview" : "Show markdown preview"}
              onclick={() => (showPreview = !showPreview)}
            >
              Preview
            </button>
          {/if}
        </div>
        {#if $activeTab && !showingDiff && !showingSettings && $activeTab.kind !== "diff"}
          <Breadcrumbs
            path={$activeTab.path}
            rootPath={$workspace?.rootPath ?? null}
          />
        {/if}
        <div
          class="editor-host"
          class:split={isMd && showPreview && !showingImage}
          style:--md-split-pct="{mdSplitFrac * 100}%"
          bind:this={editorHostEl}
        >
          {#if showingSettings}
            <SettingsView />
          {:else if showingDiff && $activeTab?.diffMeta}
            {#key $activeTab.path}
              <DiffView
                repo={$activeTab.diffMeta.repo}
                path={$activeTab.diffMeta.relPath}
                isUntracked={$activeTab.diffMeta.isUntracked}
              />
            {/key}
          {:else if $activeTab && showingImage && $activeTab.imageDataUrl}
            {#key $activeTab.path}
              <ImageView
                src={$activeTab.imageDataUrl}
                name={$activeTab.name}
                size={$activeTab.imageSize}
              />
            {/key}
          {:else if $activeTab}
            {#key $activeTab.path}
              <div class="editor-pane">
                <Editor
                  path={$activeTab.path}
                  initialContent={$activeTab.content}
                  onChange={(next) => {
                    tabs.edit($activeTab!.path, next);
                    scheduleAutoSave();
                  }}
                  onSave={saveActive}
                  onScroll={(pct) => (editorScrollPct = pct)}
                />
              </div>
              {#if isMd && showPreview}
                <Splitter
                  orientation="vertical"
                  container={editorHostEl}
                  onChange={(f) => (mdSplitFrac = f)}
                />
                <MarkdownPreview
                  source={$activeTab.content}
                  scrollPct={editorScrollPct}
                />
              {/if}
            {/key}
          {:else}
            <div class="empty">
              <div>
                <div class="empty-title">No file open</div>
                <div class="empty-sub">Click a file in the tree to start.</div>
                <div class="empty-hint">{$workspace.rootPath}</div>
              </div>
            </div>
          {/if}
        </div>
        {#if saveError}
          <div class="error-bar">{saveError}</div>
        {/if}
      </main>
    </div>

    <footer class="terminal-area" class:collapsed={terminalCollapsed}>
      <!-- terminal resize handle disabled while we sort out drag UX -->
      <!-- {#if !terminalCollapsed}
        <div
          class="terminal-resize"
          onpointerdown={startTerminalResize}
          ondblclick={resetTerminalSize}
          role="separator"
          aria-orientation="horizontal"
          title="Drag to resize · double-click to reset"
        ></div>
      {/if} -->
      <TerminalPanel
        collapsed={terminalCollapsed}
        onToggleCollapse={() => (terminalCollapsed = !terminalCollapsed)}
      />
    </footer>

    <Footer
      onWorktreeSwitch={handleWorktreeSwitch}
      onWorktreeCreate={() => (showCreateWorktree = true)}
    />

    {#if showCreateWorktree && $workspace}
      <WorktreeCreateModal
        repo={$workspace.rootPath}
        onClose={() => (showCreateWorktree = false)}
        onCreated={() => (showCreateWorktree = false)}
      />
    {/if}


    {#if $paletteOpen}
      <CommandPalette />
    {/if}

    {#if showOutline}
      <OutlinePalette onClose={() => (showOutline = false)} />
    {/if}

    {#if namePrompt}
      <NamePrompt
        title={namePrompt.title}
        confirmLabel={namePrompt.confirmLabel}
        placeholder="name"
        validate={nameValidator}
        onConfirm={namePrompt.onConfirm}
        onCancel={() => {
          namePrompt = null;
          createError = null;
        }}
      />
    {/if}
    {#if createError}
      <div class="toast" onclick={() => (createError = null)} role="presentation">
        {createError}
      </div>
    {/if}
  </div>
{/if}

{#if !$workspace && $paletteOpen}
  <CommandPalette />
{/if}

<style>
  .shell {
    height: 100%;
    display: grid;
    grid-template-rows: var(--top-bar-h) 1fr var(--bottom-row-h, var(--terminal-h)) 24px;
  }
  .welcome-shell {
    height: 100%;
    display: grid;
    grid-template-rows: var(--top-bar-h) 1fr;
  }
  .welcome-bar {
    background: transparent;
    border-bottom: none;
  }
  .top-bar {
    background: var(--bg-2);
    border-bottom: 1px solid var(--border);
    display: grid;
    grid-template-columns: minmax(0, auto) 1fr minmax(0, auto);
    align-items: center;
    -webkit-app-region: drag;
  }
  .top-bar .left,
  .top-bar .right {
    display: flex;
    align-items: center;
    height: 100%;
  }
  .top-bar .left {
    padding-left: 4px;
  }
  .title-center {
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--fg-faint);
    font-size: 11px;
  }
  .top-bar :global(button) {
    -webkit-app-region: no-drag;
  }
  .project {
    color: var(--fg-dim);
    font-size: 12px;
  }

  .body {
    display: grid;
    grid-template-columns: var(--activity-bar-w) var(--side-col-w, var(--side-panel-w)) 1fr;
    overflow: hidden;
    min-height: 0;
  }
  .side-panel {
    position: relative;
    background: var(--bg-2);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    min-height: 0;
    overflow: hidden;
  }
  .side-panel.hidden {
    display: none;
  }
  .side-resize {
    position: absolute;
    top: 0;
    right: -3px;
    bottom: 0;
    width: 6px;
    cursor: col-resize;
    z-index: 5;
  }
  .side-resize::after {
    content: "";
    position: absolute;
    top: 0;
    right: 3px;
    bottom: 0;
    width: 1px;
    background: transparent;
    transition: background 100ms ease;
  }
  .side-resize:hover::after,
  .side-resize:active::after {
    background: var(--accent);
  }
  .panel-header {
    height: 28px;
    padding: 0 4px 0 12px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    text-transform: uppercase;
    font-size: 11px;
    font-weight: 600;
    color: var(--fg-faint);
    letter-spacing: 0.5px;
    border-bottom: 1px solid var(--border);
  }
  .header-actions {
    display: flex;
    align-items: center;
    gap: 2px;
  }
  .header-btn {
    height: 22px;
    padding: 0 5px;
    border-radius: 4px;
    color: var(--fg-faint);
    line-height: 1;
  }
  .header-btn:hover {
    background: var(--bg-hover);
    color: var(--fg);
  }
  .header-btn.on {
    color: var(--accent);
  }
  .hb-glyph {
    font-size: 11px;
    letter-spacing: -1px;
  }
  .collapse-btn {
    width: 22px;
    height: 22px;
    border-radius: 4px;
    color: var(--fg-faint);
    font-size: 9px;
    line-height: 1;
  }
  .collapse-btn:hover {
    background: var(--bg-hover);
    color: var(--fg);
  }
  .panel-body {
    flex: 1;
    overflow: hidden;
    position: relative;
    min-height: 0;
  }
  .panel-slot {
    position: absolute;
    inset: 0;
    display: none;
    flex-direction: column;
    min-height: 0;
  }
  .panel-slot.active {
    display: flex;
  }
  .editor-area {
    background: var(--bg);
    overflow: hidden;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }
  .editor-host {
    flex: 1;
    overflow: hidden;
    min-height: 0;
    display: grid;
    grid-template-columns: 1fr;
  }
  .editor-host.split {
    grid-template-columns: minmax(150px, var(--md-split-pct, 50%)) auto minmax(150px, 1fr);
  }
  .editor-host > :global(*) {
    min-width: 0;
    min-height: 0;
  }
  .editor-pane {
    height: 100%;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }
  .tab-row {
    display: flex;
    align-items: center;
    background: var(--bg-2);
    border-bottom: 1px solid var(--border);
  }
  .tab-row :global(.tabs) {
    flex: 1;
    border-bottom: none;
  }
  .view-toggle {
    color: var(--fg-faint);
    font-size: 11px;
    padding: 4px 10px;
    margin-right: 4px;
    border-radius: 4px;
    flex-shrink: 0;
  }
  .view-toggle:last-of-type {
    margin-right: 8px;
  }
  .view-toggle:hover {
    background: var(--bg-hover);
    color: var(--fg);
  }
  .view-toggle.active {
    color: var(--accent);
  }
  .terminal-area {
    position: relative;
    background: var(--bg-2);
    border-top: 1px solid var(--border);
    overflow: hidden;
  }
  .terminal-resize {
    position: absolute;
    left: 0;
    right: 0;
    top: -3px;
    height: 6px;
    cursor: row-resize;
    z-index: 5;
  }
  .terminal-resize::after {
    content: "";
    position: absolute;
    left: 0;
    right: 0;
    top: 3px;
    height: 1px;
    background: transparent;
    transition: background 100ms ease;
  }
  .terminal-resize:hover::after,
  .terminal-resize:active::after {
    background: var(--accent);
  }
  .empty {
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    text-align: center;
    color: var(--fg-faint);
  }
  .empty-title {
    color: var(--fg-dim);
    margin-bottom: 4px;
    font-size: 14px;
  }
  .empty-sub {
    font-size: 12px;
    margin-bottom: 12px;
  }
  .empty-hint {
    font-size: 11px;
    font-family: var(--font-mono);
    color: var(--fg-faint);
  }
  .error-bar {
    background: rgba(247, 118, 142, 0.15);
    color: var(--red);
    padding: 6px 12px;
    font-size: 12px;
    border-top: 1px solid rgba(247, 118, 142, 0.3);
  }
  .toast {
    position: fixed;
    bottom: 24px;
    left: 50%;
    transform: translateX(-50%);
    background: var(--bg-3);
    color: var(--red);
    border: 1px solid var(--border);
    padding: 8px 14px;
    border-radius: 6px;
    font-size: 12px;
    z-index: 500;
    cursor: pointer;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
    user-select: none;
    max-width: 80vw;
  }
</style>
