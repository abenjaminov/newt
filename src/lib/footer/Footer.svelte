<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onDestroy, onMount } from "svelte";
  import WorktreeSwitcher from "../worktree/WorktreeSwitcher.svelte";
  import type { Worktree } from "../worktree/worktree-store";

  type Props = {
    onWorktreeSwitch: (target: Worktree) => void;
    onWorktreeCreate: () => void;
  };

  let { onWorktreeSwitch, onWorktreeCreate }: Props = $props();

  type SelfStats = {
    cpu: number; // 0..100, normalized across all cores
    memory: number;
    process_count: number;
    root_pid: number;
    num_cpus: number;
  };

  let stats = $state<SelfStats | null>(null);
  let smoothCpu = $state<number | null>(null);
  let smoothMem = $state<number | null>(null);
  let timer: ReturnType<typeof setInterval> | undefined;
  let busy = false;

  // Exponential moving average — alpha = how much weight the latest sample gets.
  // 0.25 → "show ~the average of the last ~4-5 samples"; new spikes ramp in
  // gradually instead of bouncing the displayed value 0 → 56% → 0 each tick.
  const ALPHA_CPU = 0.25;
  const ALPHA_MEM = 0.4; // memory is less noisy; can react a bit faster

  async function pollStats() {
    if (busy) return;
    busy = true;
    try {
      const fresh = await invoke<SelfStats>("self_stats");
      stats = fresh;
      smoothCpu =
        smoothCpu === null
          ? fresh.cpu
          : ALPHA_CPU * fresh.cpu + (1 - ALPHA_CPU) * smoothCpu;
      smoothMem =
        smoothMem === null
          ? fresh.memory
          : ALPHA_MEM * fresh.memory + (1 - ALPHA_MEM) * smoothMem;
    } catch {
      // ignore
    } finally {
      busy = false;
    }
  }

  onMount(() => {
    void pollStats();
    timer = setInterval(() => void pollStats(), 1500);
  });
  onDestroy(() => {
    if (timer) clearInterval(timer);
  });

  function fmtMem(bytes: number): string {
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(0)} KB`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(0)} MB`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
  }
</script>

<footer class="status-bar">
  <div class="left">
    <WorktreeSwitcher
      onSwitch={onWorktreeSwitch}
      onCreate={onWorktreeCreate}
      direction="up"
    />
  </div>

  <div class="right">
    {#if stats && smoothCpu !== null && smoothMem !== null}
      <span
        class="stat"
        title={`CPU across newt + descendants, normalized over ${stats.num_cpus} cores`}
      >
        <span class="dot cpu" class:hot={smoothCpu > 50}></span>
        <span class="val">{smoothCpu.toFixed(0)}%</span>
        <span class="lbl">CPU</span>
      </span>
      <span class="sep">·</span>
      <span class="stat" title="Resident memory (smoothed) of newt and its child processes">
        <span class="dot mem"></span>
        <span class="val">{fmtMem(smoothMem)}</span>
        <span class="lbl">RAM</span>
      </span>
      <span class="sep">·</span>
      <span class="stat procs" title="Number of processes (newt + descendants)">
        {stats.process_count} proc{stats.process_count === 1 ? "" : "s"}
      </span>
    {:else}
      <span class="stat muted">—</span>
    {/if}
  </div>
</footer>

<style>
  .status-bar {
    height: 24px;
    background: var(--bg-2);
    border-top: 1px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 4px 0 8px;
    font-size: 11px;
    color: var(--fg-faint);
    user-select: none;
    flex-shrink: 0;
  }
  .left,
  .right {
    display: flex;
    align-items: center;
    gap: 6px;
    height: 100%;
  }
  .stat {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 0 4px;
    height: 20px;
    border-radius: 3px;
  }
  .stat.muted {
    color: var(--fg-faint);
  }
  .val {
    font-family: var(--font-mono);
    color: var(--fg-dim);
  }
  .lbl {
    color: var(--fg-faint);
    font-size: 10px;
    letter-spacing: 0.3px;
  }
  .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
  }
  .dot.cpu {
    background: var(--green);
  }
  .dot.cpu.hot {
    background: var(--yellow);
  }
  .dot.mem {
    background: var(--accent-2);
  }
  .sep {
    color: var(--fg-faint);
    opacity: 0.5;
  }
  .procs {
    color: var(--fg-faint);
    font-family: var(--font-mono);
  }
</style>
