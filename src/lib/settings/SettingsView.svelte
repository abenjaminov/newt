<script lang="ts">
  import {
    DEFAULTS,
    FONT_SUGGESTIONS_MONO,
    FONT_SUGGESTIONS_UI,
    resetSettings,
    settings,
    updateSettings,
    type Settings,
  } from "./settings-store";
  import {
    keymap,
    setBinding,
    resetKeymap,
  } from "../keybindings/keybinding-store";
  import { commandRegistry } from "../palette/palette-store";
  import KeybindingRecorder from "../keybindings/KeybindingRecorder.svelte";

  const ZOOM_PRESETS = [0.75, 0.9, 1.0, 1.1, 1.25, 1.5];
  const SIZE_MIN = 9;
  const SIZE_MAX = 28;

  function set<K extends keyof Settings>(key: K, value: Settings[K]) {
    void updateSettings({ [key]: value } as Partial<Settings>);
  }

  function clampSize(v: number): number {
    if (Number.isNaN(v)) return 13;
    return Math.max(SIZE_MIN, Math.min(SIZE_MAX, Math.round(v)));
  }

  function zoomPercent(z: number): string {
    return `${Math.round(z * 100)}%`;
  }
</script>

<div class="settings-page">
  <header>
    <h2>Settings</h2>
    <button class="link" onclick={() => resetSettings()}>Reset all</button>
  </header>

  <div class="body">
    <section>
      <h3>General</h3>

      <div class="field">
        <label for="zoom">UI zoom</label>
        <div class="zoom-row">
          <button
            class="step"
            onclick={() =>
              set("uiZoom", Math.max(0.5, +(($settings.uiZoom - 0.1).toFixed(2))))}
          >−</button>
          <div class="zoom-display">{zoomPercent($settings.uiZoom)}</div>
          <button
            class="step"
            onclick={() =>
              set("uiZoom", Math.min(2, +(($settings.uiZoom + 0.1).toFixed(2))))}
          >+</button>
          <div class="presets">
            {#each ZOOM_PRESETS as z}
              <button
                class="preset"
                class:active={Math.abs(z - $settings.uiZoom) < 0.001}
                onclick={() => set("uiZoom", z)}
              >{zoomPercent(z)}</button>
            {/each}
          </div>
        </div>
        <p class="hint">Scales the entire window. Affects spacing, font sizes, and chrome.</p>
      </div>

      <div class="field">
        <label for="ui-font">Interface font</label>
        <input
          id="ui-font"
          type="text"
          list="ui-font-list"
          value={$settings.uiFontFamily}
          onchange={(e) => set("uiFontFamily", e.currentTarget.value)}
        />
        <datalist id="ui-font-list">
          {#each FONT_SUGGESTIONS_UI as f}<option value={f}></option>{/each}
        </datalist>
      </div>

      <div class="field">
        <label for="ui-size">Interface font size</label>
        <input
          id="ui-size"
          type="number"
          min={SIZE_MIN}
          max={SIZE_MAX}
          value={$settings.uiFontSize}
          onchange={(e) =>
            set("uiFontSize", clampSize(parseInt(e.currentTarget.value, 10)))}
        />
      </div>

      <div class="field">
        <span class="field-label">Tabs from a different worktree</span>
        <div class="radio-row">
          <label class="radio">
            <input
              type="radio"
              checked={$settings.worktreeForeignTabs === "mark"}
              onchange={() => set("worktreeForeignTabs", "mark")}
            />
            Mark with chip
          </label>
          <label class="radio">
            <input
              type="radio"
              checked={$settings.worktreeForeignTabs === "close"}
              onchange={() => set("worktreeForeignTabs", "close")}
            />
            Close on switch
          </label>
          <label class="radio">
            <input
              type="radio"
              checked={$settings.worktreeForeignTabs === "keepActive"}
              onchange={() => set("worktreeForeignTabs", "keepActive")}
            />
            Leave silently
          </label>
        </div>
      </div>
    </section>

    <section>
      <h3>Editor</h3>

      <div class="field">
        <label for="editor-font">Font family</label>
        <input
          id="editor-font"
          type="text"
          list="mono-font-list"
          value={$settings.editorFontFamily}
          onchange={(e) => set("editorFontFamily", e.currentTarget.value)}
        />
        <datalist id="mono-font-list">
          {#each FONT_SUGGESTIONS_MONO as f}<option value={f}></option>{/each}
        </datalist>
      </div>

      <div class="row two">
        <div class="field">
          <label for="editor-size">Font size</label>
          <input
            id="editor-size"
            type="number"
            min={SIZE_MIN}
            max={SIZE_MAX}
            value={$settings.editorFontSize}
            onchange={(e) =>
              set("editorFontSize", clampSize(parseInt(e.currentTarget.value, 10)))}
          />
        </div>

        <div class="field">
          <label for="tab-size">Tab size</label>
          <select
            id="tab-size"
            value={$settings.editorTabSize}
            onchange={(e) =>
              set("editorTabSize", parseInt(e.currentTarget.value, 10))}
          >
            {#each [2, 4, 8] as n}
              <option value={n} selected={$settings.editorTabSize === n}>{n}</option>
            {/each}
          </select>
        </div>
      </div>

      <div class="field">
        <label class="checkbox">
          <input
            type="checkbox"
            checked={$settings.editorLineWrap}
            onchange={(e) => set("editorLineWrap", e.currentTarget.checked)}
          />
          Wrap long lines
        </label>
      </div>

      <div class="field">
        <span class="field-label">Auto-save</span>
        <div class="radio-row">
          <label class="radio">
            <input
              type="radio"
              checked={$settings.editorAutoSave === "off"}
              onchange={() => set("editorAutoSave", "off")}
            />
            Off
          </label>
          <label class="radio">
            <input
              type="radio"
              checked={$settings.editorAutoSave === "afterDelay"}
              onchange={() => set("editorAutoSave", "afterDelay")}
            />
            After delay
          </label>
          <label class="radio">
            <input
              type="radio"
              checked={$settings.editorAutoSave === "onFocusChange"}
              onchange={() => set("editorAutoSave", "onFocusChange")}
            />
            On focus change
          </label>
        </div>
      </div>

      {#if $settings.editorAutoSave === "afterDelay"}
        <div class="field">
          <label for="autosave-delay">Auto-save delay (ms)</label>
          <input
            id="autosave-delay"
            type="number"
            min="50"
            max="5000"
            step="50"
            value={$settings.editorAutoSaveDelayMs}
            onchange={(e) => {
              const v = parseInt(e.currentTarget.value, 10);
              if (!Number.isNaN(v))
                set("editorAutoSaveDelayMs", Math.max(50, Math.min(5000, v)));
            }}
          />
        </div>
      {/if}

      <div class="field">
        <label class="checkbox">
          <input
            type="checkbox"
            checked={$settings.formatOnSave}
            onchange={(e) => set("formatOnSave", e.currentTarget.checked)}
          />
          Format on save (uses commands below)
        </label>
      </div>

      <div class="field">
        <span class="field-label">Formatters (one per extension; <code>{'{file}'}</code> is the path)</span>
        <textarea
          rows="4"
          spellcheck="false"
          placeholder={"ts: prettier --write {file}\nrs: rustfmt {file}\npy: black {file}"}
          value={Object.entries($settings.formatters)
            .map(([ext, cmd]) => `${ext}: ${cmd}`)
            .join("\n")}
          onchange={(e) => {
            const out: Record<string, string> = {};
            for (const line of e.currentTarget.value.split(/\r?\n/)) {
              const trimmed = line.trim();
              if (!trimmed) continue;
              const colon = trimmed.indexOf(":");
              if (colon <= 0) continue;
              const ext = trimmed.slice(0, colon).trim().toLowerCase().replace(/^\./, "");
              const cmd = trimmed.slice(colon + 1).trim();
              if (ext && cmd) out[ext] = cmd;
            }
            set("formatters", out);
          }}
        ></textarea>
      </div>
    </section>

    <section>
      <h3>Terminal</h3>

      <div class="field">
        <label for="term-font">Font family</label>
        <input
          id="term-font"
          type="text"
          list="mono-font-list"
          value={$settings.terminalFontFamily}
          onchange={(e) => set("terminalFontFamily", e.currentTarget.value)}
        />
      </div>

      <div class="field">
        <label for="term-size">Font size</label>
        <input
          id="term-size"
          type="number"
          min={SIZE_MIN}
          max={SIZE_MAX}
          value={$settings.terminalFontSize}
          onchange={(e) =>
            set("terminalFontSize", clampSize(parseInt(e.currentTarget.value, 10)))}
        />
      </div>
    </section>

    <section>
      <div class="section-head-row">
        <h3>Keyboard Shortcuts</h3>
        <button class="link" onclick={() => resetKeymap()}>Reset shortcuts</button>
      </div>
      {#if $commandRegistry.length === 0}
        <p class="muted">Open a folder to see workspace-specific commands here.</p>
      {/if}
      <div class="keybindings">
        {#each $commandRegistry as cmd (cmd.id)}
          <div class="kb-row">
            <div class="kb-label">
              <span class="kb-group">{cmd.group}</span>
              <span class="kb-title">{cmd.title}</span>
            </div>
            <KeybindingRecorder
              chord={$keymap[cmd.id] ?? null}
              onChange={(next) => setBinding(cmd.id, next)}
            />
          </div>
        {/each}
      </div>
    </section>

    <p class="footnote">
      Settings persist in <code>settings.json</code> and <code>keybindings.json</code> in the app config directory.
      <strong>Ctrl+Shift+P</strong> always opens the command palette.
      <strong>Shift+Shift</strong> opens it from a double-tap.
    </p>
  </div>
</div>

<style>
  .settings-page {
    height: 100%;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    background: var(--bg);
    color: var(--fg);
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 14px 24px 12px;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }
  h2 {
    margin: 0;
    font-size: 15px;
    font-weight: 600;
    color: var(--fg);
  }
  .link {
    color: var(--fg-faint);
    font-size: 11px;
    padding: 4px 8px;
    border-radius: 4px;
  }
  .link:hover {
    background: var(--bg-hover);
    color: var(--fg);
  }
  .body {
    flex: 1;
    overflow-y: auto;
    padding: 8px 24px 18px;
    max-width: 760px;
    width: 100%;
    align-self: center;
  }
  section {
    padding: 10px 0;
    border-bottom: 1px solid var(--border);
  }
  section:last-of-type {
    border-bottom: none;
  }
  h3 {
    margin: 8px 0 12px;
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.6px;
    color: var(--fg-faint);
    font-weight: 600;
  }
  .field {
    margin-bottom: 12px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .field > label:not(.checkbox):not(.radio),
  .field > .field-label {
    font-size: 11px;
    color: var(--fg-dim);
  }
  .field input[type="text"],
  .field input[type="number"],
  .field textarea,
  .field select {
    background: var(--bg-3);
    border: 1px solid var(--border);
    border-radius: 5px;
    padding: 6px 9px;
    color: var(--fg);
    font-size: 12px;
    font-family: var(--font-mono);
  }
  .field textarea {
    resize: vertical;
    line-height: 1.5;
  }
  .field input[type="number"] {
    width: 90px;
    font-family: var(--font-ui);
  }
  .field select {
    font-family: var(--font-ui);
  }
  .field input:focus,
  .field textarea:focus,
  .field select:focus {
    outline: none;
    border-color: var(--accent);
  }
  .row.two {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 14px;
  }
  .checkbox,
  .radio {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    cursor: pointer;
    font-size: 12px;
    color: var(--fg-dim);
  }
  .radio-row {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
  }
  .zoom-row {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }
  .step {
    width: 26px;
    height: 26px;
    border: 1px solid var(--border);
    border-radius: 5px;
    background: var(--bg-3);
    color: var(--fg-dim);
    font-size: 14px;
    line-height: 1;
  }
  .step:hover {
    background: var(--bg-hover);
    color: var(--fg);
  }
  .zoom-display {
    min-width: 50px;
    text-align: center;
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--fg);
  }
  .presets {
    display: flex;
    gap: 4px;
    margin-left: 8px;
  }
  .preset {
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 3px 7px;
    color: var(--fg-faint);
    font-size: 10px;
    background: transparent;
  }
  .preset:hover {
    background: var(--bg-hover);
    color: var(--fg);
  }
  .preset.active {
    border-color: var(--accent);
    color: var(--accent);
  }
  .hint {
    margin: 2px 0 0;
    color: var(--fg-faint);
    font-size: 11px;
  }
  .section-head-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .section-head-row h3 {
    margin: 8px 0 4px;
  }
  .keybindings {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-top: 4px;
  }
  .kb-row {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    gap: 12px;
    align-items: center;
    padding: 4px 6px;
    border-radius: 5px;
  }
  .kb-row:hover {
    background: rgba(255, 255, 255, 0.025);
  }
  .kb-label {
    display: flex;
    flex-direction: column;
    gap: 1px;
    min-width: 0;
  }
  .kb-group {
    font-size: 9px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--fg-faint);
  }
  .kb-title {
    font-size: 12px;
    color: var(--fg-dim);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .muted {
    color: var(--fg-faint);
    font-size: 12px;
    padding: 8px 0;
  }
  .footnote {
    color: var(--fg-faint);
    font-size: 11px;
    margin: 14px 0 0;
  }
  .footnote strong {
    color: var(--fg-dim);
    font-family: var(--font-mono);
    font-weight: 500;
  }
  .footnote code {
    background: var(--bg-3);
    padding: 1px 5px;
    border-radius: 3px;
    font-family: var(--font-mono);
    font-size: 10px;
  }
</style>
