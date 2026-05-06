<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onDestroy, onMount } from "svelte";
  import logoUrl from "../../../src-tauri/icons/128x128@2x.png";
  import {
    getRecents,
    removeRecent,
    type RecentFolder,
  } from "../recent/recent";
  import {
    openFolder,
    pickAndOpenFolder,
  } from "../workspace/open-workspace";

  let recents = $state<RecentFolder[]>([]);
  let logos = $state<Record<string, string | null>>({});

  async function refresh() {
    recents = await getRecents();
    // Kick off logo lookups in parallel; update map as each resolves.
    for (const r of recents) {
      if (r.path in logos) continue;
      logos = { ...logos, [r.path]: null };
      void invoke<string | null>("find_logo", { path: r.path })
        .then((url) => {
          logos = { ...logos, [r.path]: url };
        })
        .catch(() => {});
    }
  }

  async function pickFolder() {
    await pickAndOpenFolder();
  }

  async function openPath(path: string) {
    await openFolder(path);
  }

  async function forget(path: string, e: MouseEvent) {
    e.stopPropagation();
    await removeRecent(path);
    await refresh();
  }

  function relTime(ts: number): string {
    const diffSec = Math.max(1, Math.floor((Date.now() - ts) / 1000));
    if (diffSec < 60) return "just now";
    const min = Math.floor(diffSec / 60);
    if (min < 60) return `${min}m ago`;
    const hr = Math.floor(min / 60);
    if (hr < 24) return `${hr}h ago`;
    const d = Math.floor(hr / 24);
    if (d < 30) return `${d}d ago`;
    const mo = Math.floor(d / 30);
    if (mo < 12) return `${mo}mo ago`;
    return `${Math.floor(mo / 12)}y ago`;
  }

  function shortPath(p: string): string {
    // Collapse $HOME / drive prefixes for readability.
    const max = 56;
    if (p.length <= max) return p;
    const parts = p.split(/[\\/]/);
    if (parts.length <= 4) return p;
    const sep = p.includes("\\") && !p.includes("/") ? "\\" : "/";
    return [parts[0], "…", parts[parts.length - 2], parts[parts.length - 1]].join(sep);
  }

  function handleKey(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "o") {
      e.preventDefault();
      void pickFolder();
    }
  }

  onMount(() => {
    void refresh();
    window.addEventListener("keydown", handleKey);
  });
  onDestroy(() => window.removeEventListener("keydown", handleKey));
</script>

<div class="welcome">
  <div class="glow glow-a"></div>
  <div class="glow glow-b"></div>

  <div class="content">
    <div class="brand">
      <div class="logo" aria-hidden="true">
        <img src={logoUrl} alt="Newt" />
      </div>
      <h1>Newt</h1>
      <p class="tagline">A small, snappy IDE.</p>
    </div>

    <div class="primary-action">
      <button class="primary" onclick={pickFolder}>
        <span>Open folder</span>
        <span class="kbd-group">
          <kbd>Ctrl</kbd><span class="plus">+</span><kbd>O</kbd>
        </span>
      </button>
    </div>

    {#if recents.length > 0}
      <section class="recents-section">
        <div class="section-head">Recent</div>
        <ul class="recents">
          {#each recents as r (r.path)}
            <li class="recent">
              <button class="recent-main" onclick={() => openPath(r.path)}>
                <span class="folder-icon" aria-hidden="true">
                  {#if logos[r.path]}
                    <img src={logos[r.path]} alt="" />
                  {:else}
                    <span class="folder-glyph">▣</span>
                  {/if}
                </span>
                <span class="recent-info">
                  <span class="recent-name">{r.name}</span>
                  <span class="recent-path">{shortPath(r.path)}</span>
                </span>
                <span class="recent-time">{relTime(r.lastOpened)}</span>
              </button>
              <button
                class="forget"
                title="Remove from recents"
                onclick={(e) => forget(r.path, e)}
              >×</button>
            </li>
          {/each}
        </ul>
      </section>
    {/if}

    <div class="hints">
      <span><kbd>Ctrl</kbd>+<kbd>O</kbd> open folder</span>
      <span class="dot">·</span>
      <span><kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>W</kbd> worktrees</span>
    </div>
  </div>
</div>

<style>
  .welcome {
    position: relative;
    flex: 1;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg);
    overflow: hidden;
  }
  .glow {
    position: absolute;
    border-radius: 50%;
    filter: blur(120px);
    opacity: 0.18;
    pointer-events: none;
  }
  .glow-a {
    width: 600px;
    height: 600px;
    top: -200px;
    left: 30%;
    background: radial-gradient(circle, var(--accent), transparent 70%);
  }
  .glow-b {
    width: 480px;
    height: 480px;
    bottom: -160px;
    right: 25%;
    background: radial-gradient(circle, var(--accent-2), transparent 70%);
    opacity: 0.12;
  }
  .content {
    width: min(600px, 90%);
    position: relative;
    z-index: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 32px 24px;
  }

  .brand {
    text-align: center;
    margin-bottom: 28px;
  }
  .logo {
    width: 80px;
    height: 80px;
    border-radius: 18px;
    overflow: hidden;
    margin: 0 auto 18px;
    box-shadow: 0 8px 24px rgba(232, 145, 90, 0.25);
  }
  .logo img {
    width: 100%;
    height: 100%;
    object-fit: contain;
    display: block;
  }
  h1 {
    margin: 0 0 4px 0;
    font-weight: 600;
    font-size: 26px;
    letter-spacing: -0.6px;
    color: var(--fg);
  }
  .tagline {
    color: var(--fg-faint);
    margin: 0;
    font-size: 13px;
  }

  .primary-action {
    margin-bottom: 36px;
  }
  .primary {
    display: inline-flex;
    align-items: center;
    gap: 12px;
    background: var(--accent);
    color: #0a0c10;
    padding: 9px 16px 9px 18px;
    border-radius: 8px;
    font-weight: 500;
    font-size: 13px;
    box-shadow:
      0 4px 16px rgba(232, 145, 90, 0.25),
      inset 0 1px 0 rgba(255, 255, 255, 0.2);
    transition: transform 80ms ease, filter 80ms ease;
  }
  .primary:hover {
    filter: brightness(1.08);
    transform: translateY(-1px);
  }
  .primary:active {
    transform: translateY(0);
  }
  .kbd-group {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    opacity: 0.75;
  }
  .kbd-group .plus {
    color: rgba(10, 12, 16, 0.55);
    font-size: 10px;
  }
  .primary kbd {
    background: rgba(10, 12, 16, 0.18);
    color: rgba(10, 12, 16, 0.85);
    border-radius: 4px;
    padding: 1px 6px;
    font-size: 10px;
    font-family: var(--font-mono);
    font-weight: 500;
    line-height: 1.4;
  }

  .recents-section {
    width: 100%;
    margin-bottom: 28px;
  }
  .section-head {
    font-size: 10px;
    font-weight: 600;
    color: var(--fg-faint);
    text-transform: uppercase;
    letter-spacing: 0.8px;
    margin: 0 0 10px 6px;
  }
  .recents {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .recent {
    display: grid;
    grid-template-columns: 1fr auto;
    align-items: center;
    background: rgba(255, 255, 255, 0.025);
    border: 1px solid transparent;
    border-radius: 8px;
    transition: background 100ms ease, border-color 100ms ease, transform 100ms ease;
  }
  .recent:hover {
    background: rgba(255, 255, 255, 0.05);
    border-color: var(--border);
  }
  .recent-main {
    display: grid;
    grid-template-columns: auto 1fr auto;
    align-items: center;
    gap: 12px;
    width: 100%;
    text-align: left;
    padding: 10px 14px;
  }
  .folder-icon {
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    border-radius: 6px;
    background: rgba(255, 255, 255, 0.04);
    overflow: hidden;
  }
  .folder-icon img {
    width: 100%;
    height: 100%;
    object-fit: contain;
    display: block;
  }
  .folder-glyph {
    color: var(--accent);
    font-size: 16px;
    line-height: 1;
    opacity: 0.75;
  }
  .recent-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }
  .recent-name {
    color: var(--fg);
    font-weight: 500;
    font-size: 13px;
  }
  .recent-path {
    color: var(--fg-faint);
    font-family: var(--font-mono);
    font-size: 11px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .recent-time {
    color: var(--fg-faint);
    font-size: 11px;
    flex-shrink: 0;
  }
  .forget {
    width: 26px;
    height: 26px;
    margin-right: 10px;
    border-radius: 5px;
    color: var(--fg-faint);
    font-size: 16px;
    line-height: 1;
    opacity: 0;
  }
  .recent:hover .forget {
    opacity: 1;
  }
  .forget:hover {
    background: var(--bg-hover);
    color: var(--fg);
  }

  .hints {
    display: flex;
    align-items: center;
    gap: 12px;
    color: var(--fg-faint);
    font-size: 11px;
    opacity: 0.75;
  }
  .hints kbd {
    background: var(--bg-2);
    border: 1px solid var(--border);
    border-radius: 3px;
    padding: 1px 5px;
    font-size: 10px;
    font-family: var(--font-mono);
    color: var(--fg-dim);
  }
  .dot {
    color: var(--fg-faint);
  }
</style>
