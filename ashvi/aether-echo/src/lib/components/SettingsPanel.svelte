<script lang="ts">
  import { onMount } from 'svelte';
  import * as api from '../api';
  import type { Settings, ModelInfo } from '../api';

  interface Props {
    onBack?: () => void;
  }
  let { onBack }: Props = $props();

  let settings = $state<Settings>({
    model: 'llama-3.3-70b-versatile',
    overlay_opacity: 0.88,
    hotkey_toggle_overlay: 'CommandOrControl+Alt+Shift+O',
    hotkey_toggle_listening: 'CommandOrControl+Alt+Shift+L',
    hotkey_toggle_click_through: 'CommandOrControl+Alt+Shift+K',
    hotkey_screenshot_solve: 'CommandOrControl+Alt+Shift+S',
    vad_sensitivity: 0.015,
    vad_silence_timeout: 1000,
    auto_start_listening: false,
    whisper_model: 'tiny',
    system_prompt: '',
    overlay_click_through: true,
    theme: 'dark',
  });

  let models = $state<ModelInfo[]>([]);
  let saving = $state(false);
  let saveOk = $state(false);
  let saveErr = $state('');
  let changingKey = $state(false);
  let newKey = $state('');
  let keyVisible = $state(false);
  let testingKey = $state(false);
  let keyTestResult = $state<null | 'valid' | 'invalid'>(null);
  let checkingModel = $state(false);
  let modelOk = $state(false);
  let downloadingModel = $state(false);
  let downloadPct = $state(0);
  let downloadErr = $state('');
  let activeTab = $state<'general' | 'model' | 'overlay' | 'hotkeys' | 'api'>('general');

  let unlisten: (() => void) | undefined;
  onMount(() => {
    const init = async () => {
      const [settingsRes, modelsRes] = await Promise.all([
        api.getSettings(),
        api.getAvailableModels(),
      ]);

      if (settingsRes.success && settingsRes.data) settings = settingsRes.data;
      if (modelsRes.success && modelsRes.data) models = modelsRes.data;

      checkModelStatus();

      const fn = await api.onModelDownloadProgress((pct) => {
        downloadPct = Math.round(pct * 100);
      });
      unlisten = fn;
    };

    init();

    return () => {
      if (unlisten) unlisten();
    };
  });

  async function checkModelStatus() {
    checkingModel = true;
    const res = await api.checkModelExists(settings.whisper_model);
    modelOk = res.data ?? false;
    checkingModel = false;
  }

  async function saveAll() {
    saving = true;
    saveErr = '';
    const res = await api.saveSettings(settings);
    saving = false;
    if (res.success) {
      saveOk = true;
      setTimeout(() => saveOk = false, 2000);
    } else {
      saveErr = res.error || 'Save failed';
    }
  }

  async function testNewKey() {
    if (!newKey.trim()) return;
    testingKey = true;
    keyTestResult = null;
    const res = await api.testApiKey(newKey.trim());
    testingKey = false;
    if (res.success) keyTestResult = res.data ? 'valid' : 'invalid';
  }

  async function saveNewKey() {
    if (!newKey.trim()) return;
    await api.saveApiKey(newKey.trim());
    changingKey = false;
    newKey = '';
    keyTestResult = null;
  }

  async function downloadModel() {
    downloadingModel = true;
    downloadErr = '';
    downloadPct = 0;
    const res = await api.downloadModel(settings.whisper_model);
    downloadingModel = false;
    if (res.success) modelOk = true;
    else downloadErr = res.error || 'Download failed';
  }

  function opacityLabel(v: number): string {
    return `${Math.round(v * 100)}%`;
  }
</script>

<div class="settings-page">
  <!-- Header -->
  <header class="settings-header">
    <button class="back-btn" onclick={() => onBack?.()}>
      ← Back
    </button>
    <h1>Settings</h1>
    <button
      class="btn btn-primary btn-sm"
      onclick={saveAll}
      disabled={saving}
    >
      {saving ? '⌛ Saving…' : saveOk ? '✅ Saved!' : '💾 Save'}
    </button>
  </header>

  {#if saveErr}
    <div class="save-err">{saveErr}</div>
  {/if}

  <!-- Tabs -->
  <div class="tabs">
    {#each [
      { id: 'general', label: '⚙ General' },
      { id: 'overlay', label: '👁 Overlay' },
      { id: 'model',   label: '🧠 Model' },
      { id: 'hotkeys', label: '⌨ Hotkeys' },
      { id: 'api',     label: '🔑 API Key' },
    ] as tab}
      <button
        class="tab-btn"
        class:active={activeTab === tab.id}
        onclick={() => activeTab = tab.id as any}
      >{tab.label}</button>
    {/each}
  </div>

  <div class="settings-body">
    {#if activeTab === 'general'}
      <!-- General -->
      <div class="section">
        <div class="section-label">AI Model</div>
        <select class="input" bind:value={settings.model}>
          {#each models as m}
            <option value={m.id}>{m.name}</option>
          {/each}
        </select>
        <p class="hint">Model used for generating interview answers via Groq API</p>
      </div>

      <div class="section">
        <div class="section-label">System Prompt</div>
        <textarea
          class="input"
          rows="5"
          bind:value={settings.system_prompt}
          placeholder="You are an expert AI assistant helping in a technical interview…"
        ></textarea>
        <p class="hint">Instructions sent to the AI before your interview questions</p>
      </div>

      <div class="section">
        <div class="section-label">Voice Sensitivity</div>
        <div class="slider-row">
          <span class="text-muted text-xs">Low</span>
          <input
            type="range"
            class="slider flex-1"
            min="0.005"
            max="0.08"
            step="0.001"
            value={0.085 - settings.vad_sensitivity}
            oninput={(e) => settings.vad_sensitivity = 0.085 - parseFloat(e.currentTarget.value)}
          />
          <span class="text-muted text-xs">High</span>
          <span class="text-xs text-accent">{Math.round((0.085 - settings.vad_sensitivity) * 1250)}%</span>
        </div>
        <p class="hint">How sensitive the microphone detection is. Increase if misses words; decrease if too noisy.</p>
      </div>

      <div class="section">
        <div class="section-label">Silence Timeout (Pause Duration)</div>
        <div class="slider-row">
          <span class="text-muted text-xs">Short (300ms)</span>
          <input
            type="range"
            class="slider flex-1"
            min="300"
            max="3000"
            step="100"
            bind:value={settings.vad_silence_timeout}
          />
          <span class="text-muted text-xs">Long (3s)</span>
          <span class="text-xs text-accent">{settings.vad_silence_timeout} ms</span>
        </div>
        <p class="hint">How long of a pause to wait before sending speech for transcription. Increase if it cuts off mid-sentence (e.g. 800ms - 1500ms); decrease for faster responses (e.g. 400ms).</p>
      </div>

      <div class="section">
        <label class="toggle-wrap">
          <span class="toggle-label">Auto-start listening on launch</span>
          <label class="toggle">
            <input type="checkbox" bind:checked={settings.auto_start_listening} />
            <span class="toggle-track"></span>
            <span class="toggle-thumb"></span>
          </label>
        </label>
        <p class="hint">Automatically begin capturing audio when AetherEcho starts</p>
      </div>

    {:else if activeTab === 'overlay'}
      <!-- Overlay -->
      <div class="section">
        <div class="section-label">Overlay Opacity</div>
        <div class="slider-row">
          <span class="text-muted text-xs">10%</span>
          <input
            type="range"
            class="slider flex-1"
            min="0.1"
            max="0.98"
            step="0.01"
            bind:value={settings.overlay_opacity}
            oninput={() => api.setOverlayOpacity(settings.overlay_opacity)}
          />
          <span class="text-muted text-xs">100%</span>
          <span class="text-xs text-accent">{opacityLabel(settings.overlay_opacity)}</span>
        </div>
      </div>

      <div class="section">
        <label class="toggle-wrap">
          <span class="toggle-label">Click-through mode</span>
          <label class="toggle">
            <input
              type="checkbox"
              bind:checked={settings.overlay_click_through}
              onchange={() => api.setOverlayClickThrough(settings.overlay_click_through)}
            />
            <span class="toggle-track"></span>
            <span class="toggle-thumb"></span>
          </label>
        </label>
        <p class="hint">Mouse clicks pass through the overlay — you can interact with apps behind it</p>
      </div>

      <div class="section info-card glass">
        <div class="info-icon">🛡</div>
        <div>
          <div class="font-semibold text-sm">Screen capture protected</div>
          <p class="hint" style="margin-top: 4px">
            The overlay is excluded from Zoom, Teams, Google Meet, OBS, and Windows screen recordings
            using <code>WDA_EXCLUDEFROMCAPTURE</code>. Interviewers cannot see it.
          </p>
        </div>
      </div>

    {:else if activeTab === 'model'}
      <!-- Model -->
      <div class="section">
        <div class="section-label">Whisper Model</div>
        <div class="model-options">
          {#each [
            { id: 'tiny', name: 'Tiny', size: '75 MB', speed: 'Very Fast' },
            { id: 'base', name: 'Base', size: '145 MB', speed: 'Fast' },
            { id: 'small', name: 'Small', size: '461 MB', speed: 'Moderate' },
          ] as m}
            <label
              class="model-option glass"
              class:selected={settings.whisper_model === m.id}
            >
              <input
                type="radio"
                name="whisper-model"
                value={m.id}
                bind:group={settings.whisper_model}
                onchange={checkModelStatus}
              />
              <div class="model-option-info">
                <span class="font-semibold">{m.name}</span>
                <span class="text-muted text-xs">{m.size} · {m.speed}</span>
              </div>
            </label>
          {/each}
        </div>
      </div>

      <div class="section">
        {#if checkingModel}
          <p class="text-secondary text-sm">⌛ Checking model…</p>
        {:else if modelOk}
          <div class="alert-ok">✅ Model downloaded and ready</div>
        {:else if downloadingModel}
          <p class="text-sm text-secondary">Downloading… {downloadPct}%</p>
          <div class="progress-bar">
            <div class="progress-fill" style="width: {downloadPct}%"></div>
          </div>
        {:else}
          <button class="btn btn-primary w-full" onclick={downloadModel}>
            ⬇ Download {settings.whisper_model} model
          </button>
          {#if downloadErr}
            <p class="text-error text-sm">{downloadErr}</p>
          {/if}
        {/if}
      </div>

    {:else if activeTab === 'hotkeys'}
      <!-- Hotkeys -->
      <div class="section">
        <div class="section-label">Toggle Overlay</div>
        <input
          class="input text-mono"
          bind:value={settings.hotkey_toggle_overlay}
          placeholder="CommandOrControl+Alt+Shift+O"
        />
        <p class="hint">Global shortcut to show/hide the answer overlay</p>
      </div>

      <div class="section">
        <div class="section-label">Toggle Listening</div>
        <input
          class="input text-mono"
          bind:value={settings.hotkey_toggle_listening}
          placeholder="CommandOrControl+Alt+Shift+L"
        />
        <p class="hint">Global shortcut to start/stop audio capture</p>
      </div>

      <div class="section">
        <div class="section-label">Toggle Overlay Interaction / Click-Through</div>
        <input
          class="input text-mono"
          bind:value={settings.hotkey_toggle_click_through}
          placeholder="CommandOrControl+Alt+Shift+K"
        />
        <p class="hint">Global shortcut to toggle the overlay between click-through (locked) and interactive (scrollable/selectable)</p>
      </div>

      <div class="section">
        <div class="section-label">Screenshot Solve (Stealth Co-Pilot)</div>
        <input
          class="input text-mono"
          bind:value={settings.hotkey_screenshot_solve}
          placeholder="CommandOrControl+Alt+Shift+S"
        />
        <p class="hint">Global shortcut to hide app windows, capture the desktop behind, and solve the visible question via AI Vision</p>
      </div>

      <div class="info-card glass">
        <div class="info-icon">ℹ</div>
        <p class="hint">
          Use <code>CommandOrControl</code> for Ctrl on Windows.
          Changes take effect after saving and restarting.
        </p>
      </div>

    {:else if activeTab === 'api'}
      <!-- API Key -->
      {#if !changingKey}
        <div class="section">
          <div class="section-label">Groq API Key</div>
          <div class="api-key-display glass">
            <span class="text-mono text-sm">gsk_ •••••••••••••••••</span>
            <span class="badge badge-success">Stored Securely</span>
          </div>
          <p class="hint">
            Stored in Windows Credential Manager — never written to disk.
          </p>
          <div style="margin-top: var(--space-sm)">
            <button class="btn btn-secondary" onclick={() => changingKey = true}>
              🔄 Change API Key
            </button>
          </div>
        </div>
      {:else}
        <div class="section">
          <div class="section-label">New API Key</div>
          <div class="input-with-btn">
            <input
              type={keyVisible ? 'text' : 'password'}
              class="input input-password"
              bind:value={newKey}
              placeholder="gsk_••••••••••"
              autocomplete="off"
            />
            <button class="btn btn-secondary btn-sm" onclick={() => keyVisible = !keyVisible}>
              {keyVisible ? '🙈' : '👁'}
            </button>
          </div>

          {#if keyTestResult === 'valid'}
            <p class="text-success text-sm">✅ Key is valid!</p>
          {:else if keyTestResult === 'invalid'}
            <p class="text-error text-sm">❌ Invalid API key</p>
          {/if}

          <div class="flex gap-sm" style="margin-top: var(--space-sm)">
            <button class="btn btn-secondary" onclick={() => { changingKey = false; newKey = ''; }}>Cancel</button>
            <button class="btn btn-secondary" onclick={testNewKey} disabled={testingKey || !newKey.trim()}>
              {testingKey ? '⌛ Testing…' : '🧪 Test'}
            </button>
            <button
              class="btn btn-primary"
              onclick={saveNewKey}
              disabled={!newKey.trim() || keyTestResult === 'invalid'}
            >
              💾 Save Key
            </button>
          </div>
        </div>
      {/if}

      <div class="info-card glass" style="margin-top: var(--space-md)">
        <div class="info-icon">🔒</div>
        <div>
          <div class="font-semibold text-sm">Security Notice</div>
          <p class="hint" style="margin-top: 4px">
            Your API key is stored in Windows Credential Manager using the OS keyring.
            AetherEcho only communicates with <code>api.groq.com</code>. No telemetry or analytics.
          </p>
        </div>
      </div>
    {/if}
  </div>

  <!-- Save button footer -->
  <div class="settings-footer">
    <button
      class="btn btn-primary w-full btn-lg"
      onclick={saveAll}
      disabled={saving}
    >
      {saving ? '⌛ Saving…' : saveOk ? '✅ Saved!' : '💾 Save All Settings'}
    </button>
  </div>
</div>

<style>
  .settings-page {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: var(--bg-base);
    overflow: hidden;
  }

  .settings-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px var(--space-lg);
    border-bottom: 1px solid var(--border);
    background: var(--bg-surface);
    flex-shrink: 0;
    -webkit-app-region: drag;
  }

  .settings-header h1 {
    font-size: var(--text-md);
    font-weight: 700;
  }

  .back-btn {
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    font-size: var(--text-sm);
    font-family: var(--font-sans);
    -webkit-app-region: no-drag;
    transition: color var(--duration-fast);
  }
  .back-btn:hover { color: var(--text-primary); }

  .settings-header .btn { -webkit-app-region: no-drag; }

  .save-err {
    background: var(--error-dim);
    color: var(--error);
    padding: 8px var(--space-lg);
    font-size: var(--text-sm);
    border-bottom: 1px solid rgba(244,63,94,0.2);
  }

  .tabs {
    display: flex;
    gap: 2px;
    padding: var(--space-sm) var(--space-md);
    background: var(--bg-surface);
    border-bottom: 1px solid var(--border);
    overflow-x: auto;
    flex-shrink: 0;
  }

  .tab-btn {
    padding: 6px 14px;
    border: none;
    border-radius: var(--radius-md);
    background: none;
    color: var(--text-secondary);
    font-size: var(--text-xs);
    font-weight: 600;
    cursor: pointer;
    white-space: nowrap;
    transition: all var(--duration-fast) var(--ease-smooth);
    font-family: var(--font-sans);
  }

  .tab-btn:hover { background: var(--bg-elevated); color: var(--text-primary); }
  .tab-btn.active { background: var(--accent-dim); color: var(--accent-light); }

  .settings-body {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-lg);
    display: flex;
    flex-direction: column;
    gap: var(--space-lg);
  }

  .section {
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
  }

  .slider-row {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
  }

  .hint {
    font-size: var(--text-xs);
    color: var(--text-muted);
    line-height: 1.5;
  }

  .hint code {
    font-family: var(--font-mono);
    background: var(--bg-elevated);
    padding: 1px 5px;
    border-radius: 3px;
    font-size: 0.9em;
  }

  .toggle-wrap {
    display: flex;
    justify-content: space-between;
    align-items: center;
    cursor: pointer;
  }

  .toggle-label {
    font-size: var(--text-sm);
    font-weight: 500;
    color: var(--text-primary);
  }

  /* Model options */
  .model-options { display: flex; flex-direction: column; gap: var(--space-sm); }
  .model-option {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    padding: var(--space-sm) var(--space-md);
    cursor: pointer;
    border: 2px solid transparent;
    transition: border-color var(--duration-fast) var(--ease-smooth);
  }
  .model-option.selected { border-color: var(--accent); }
  .model-option input { accent-color: var(--accent); }
  .model-option-info { display: flex; flex-direction: column; gap: 2px; }

  .alert-ok {
    background: var(--success-dim);
    color: var(--success);
    padding: 10px 14px;
    border-radius: var(--radius-md);
    font-size: var(--text-sm);
    border: 1px solid rgba(34,211,165,0.2);
  }

  .api-key-display {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-sm) var(--space-md);
  }

  .input-with-btn { display: flex; gap: var(--space-sm); }
  .input-with-btn .input { flex: 1; }

  .info-card {
    display: flex;
    gap: var(--space-md);
    padding: var(--space-md);
    align-items: flex-start;
  }

  .info-icon { font-size: 20px; flex-shrink: 0; }

  .settings-footer {
    padding: var(--space-md) var(--space-lg);
    border-top: 1px solid var(--border);
    background: var(--bg-surface);
    flex-shrink: 0;
  }

  textarea.input {
    resize: vertical;
    font-size: var(--text-sm);
    line-height: 1.6;
    font-family: var(--font-sans);
  }
</style>
