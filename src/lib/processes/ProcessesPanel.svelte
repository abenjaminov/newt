<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onDestroy, onMount } from "svelte";
  import { listPtys, type PtyEntry } from "../terminal/pty-client";

  type Props = {
    visible: boolean;
  };

  let { visible }: Props = $props();

  type ProcInfo = {
    pid: number;
    parent_pid: number | null;
    name: string;
    cmd: string[];
    cpu: number;
    memory: number;
    run_time_secs: number;
  };

  type ProcTree = {
    roots: ProcInfo[];
    all: ProcInfo[];
  };

  let ptys = $state<PtyEntry[]>([]);
  let tree = $state<ProcTree>({ roots: [], all: [] });
  let error = $state<string | null>(null);
  let busy = $state(false);
  let timer: number | undefined;

  async function refresh() {
    if (busy) return;
    busy = true;
    try {
      ptys = await listPtys();
      const roots = ptys.map((p) => p.os_pid).filter((x): x is number => x !== null);
      tree = await invoke<ProcTree>("list_descendants", { roots });
      error = null;
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  function startPolling() {
    void refresh();
    timer = window.setInterval(() => void refresh(), 1500);
  }

  function stopPolling() {
    if (timer !== undefined) {
      clearInterval(timer);
      timer = undefined;
    }
  }

  $effect(() => {
    if (visible) startPolling();
    else stopPolling();
  });

  onMount(() => {
    if (visible) startPolling();
  });
  onDestroy(stopPolling);

  // Group process tree by root pty.
  type Group = { pty: PtyEntry; rootPid: number; children: Map<number, ProcInfo[]> };
  const groups = $derived.by((): Group[] => {
    const byParent = new Map<number, ProcInfo[]>();
    for (const p of tree.all) {
      if (p.parent_pid === null) continue;
      const list = byParent.get(p.parent_pid) ?? [];
      list.push(p);
      byParent.set(p.parent_pid, list);
    }
    return ptys
      .filter((p) => p.os_pid !== null)
      .map((p) => ({
        pty: p,
        rootPid: p.os_pid as number,
        children: byParent,
      }));
  });

  function findProc(pid: number): ProcInfo | undefined {
    return tree.all.find((p) => p.pid === pid);
  }

  function descendants(rootPid: number, byParent: Map<number, ProcInfo[]>): { proc: ProcInfo; depth: number }[] {
    const out: { proc: ProcInfo; depth: number }[] = [];
    const root = findProc(rootPid);
    if (!root) return out;
    const stack: { proc: ProcInfo; depth: number }[] = [{ proc: root, depth: 0 }];
    while (stack.length) {
      const node = stack.pop()!;
      out.push(node);
      const kids = byParent.get(node.proc.pid) ?? [];
      for (const k of [...kids].reverse()) {
        stack.push({ proc: k, depth: node.depth + 1 });
      }
    }
    return out;
  }

  async function kill(pid: number) {
    if (!window.confirm(`Kill PID ${pid}?`)) return;
    try {
      await invoke("kill_process", { pid });
    } catch (e) {
      error = String(e);
    }
    void refresh();
  }

  function fmtMem(bytes: number): string {
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(0)} KB`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
  }

  function fmtRun(s: number): string {
    if (s < 60) return `${s}s`;
    if (s < 3600) return `${Math.floor(s / 60)}m ${s % 60}s`;
    return `${Math.floor(s / 3600)}h ${Math.floor((s % 3600) / 60)}m`;
  }

  function fmtCmd(cmd: string[]): string {
    if (cmd.length === 0) return "";
    return cmd.slice(1).join(" ");
  }
</script>

<div class="proc-panel">
  {#if error}
    <div class="error">{error}</div>
  {/if}

  {#if ptys.length === 0}
    <div class="muted">No terminals are open. Start one to see its process tree here.</div>
  {:else}
    {#each groups as g (g.pty.uid)}
      <div class="group">
        <div class="group-head">
          <span class="label">{g.pty.label}</span>
          <span class="pid">root pid {g.rootPid}</span>
        </div>
        {#each descendants(g.rootPid, g.children) as { proc, depth } (proc.pid)}
          <div class="row" style:padding-left="{depth * 14 + 12}px" title={proc.cmd.join(" ")}>
            <span class="name">{proc.name}</span>
            <span class="cmd-args">{fmtCmd(proc.cmd)}</span>
            <span class="meta">
              <span class="cpu" class:hot={proc.cpu > 30}>{proc.cpu.toFixed(0)}%</span>
              <span class="mem">{fmtMem(proc.memory)}</span>
              <span class="rt">{fmtRun(proc.run_time_secs)}</span>
              <span class="pid">{proc.pid}</span>
              <button class="kill" title="Kill" onclick={() => kill(proc.pid)}>×</button>
            </span>
          </div>
        {/each}
      </div>
    {/each}
  {/if}
</div>

<style>
  .proc-panel {
    height: 100%;
    overflow: auto;
    padding: 4px 0 12px;
    font-size: 11px;
    user-select: none;
  }
  .muted {
    padding: 12px;
    color: var(--fg-faint);
    font-size: 12px;
  }
  .error {
    padding: 8px 12px;
    color: var(--red);
    font-size: 12px;
  }
  .group {
    margin-bottom: 4px;
  }
  .group-head {
    padding: 6px 12px 4px;
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    color: var(--fg-faint);
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    font-weight: 600;
  }
  .group-head .label {
    color: var(--fg-dim);
  }
  .row {
    display: grid;
    grid-template-columns: minmax(0, auto) minmax(0, 1fr) auto;
    gap: 6px;
    align-items: center;
    height: 22px;
    padding-right: 8px;
    color: var(--fg-dim);
    overflow: hidden;
  }
  .row:hover {
    background: var(--bg-hover);
    color: var(--fg);
  }
  .name {
    font-family: var(--font-mono);
    color: var(--fg);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .cmd-args {
    font-family: var(--font-mono);
    color: var(--fg-faint);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 10px;
  }
  .meta {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--fg-faint);
    font-size: 10px;
  }
  .cpu.hot {
    color: var(--yellow);
  }
  .pid {
    font-family: var(--font-mono);
  }
  .kill {
    width: 16px;
    height: 16px;
    border-radius: 3px;
    color: var(--fg-faint);
    font-size: 13px;
    line-height: 1;
    opacity: 0;
  }
  .row:hover .kill {
    opacity: 1;
  }
  .kill:hover {
    background: var(--red);
    color: #0e0f12;
  }
</style>
