<script lang="ts">
  import { onMount } from 'svelte';
  import * as api from '../api';

  interface Props {
    onComplete?: () => void;
  }
  let { onComplete }: Props = $props();

  let step = $state(1);
  const totalSteps = 4;

  // Step 1 — API Key
  let apiKey = $state('');
  let apiKeyVisible = $state(false);
  let testingKey = $state(false);
  let keyTestResult = $state<null | 'valid' | 'invalid'>(null);
  let keyError = $state('');

  // Step 2 — Model
  let selectedModel = $state('tiny');
  let modelExists = $state(false);
  let downloading = $state(false);
  let downloadProgress = $state(0);
  let downloadError = $state('');

  // Step 3 — Hotkeys (just display defaults)
  // Step 4 — Done

  let unlisten: (() => void) | undefined;
  onMount(() => {
    api.onModelDownloadProgress((pct) => {
      downloadProgress = Math.round(pct * 100);
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      if (unlisten) unlisten();
    };
  });

  async function testKey() {
    if (!apiKey.trim()) { keyError = 'Enter your API key first'; return; }
    testingKey = true;
    keyTestResult = null;
    keyError = '';

    const res = await api.testApiKey(apiKey.trim());
    testingKey = false;

    if (!res.success) {
      keyError = res.error || 'Connection test failed';
      return;
    }
    keyTestResult = res.data ? 'valid' : 'invalid';
    if (!res.data) keyError = 'Invalid API key — please check it and try again';
  }

  async function saveAndContinue() {
    if (!apiKey.trim()) { keyError = 'API key is required'; return; }
    const res = await api.saveApiKey(apiKey.trim());
    if (!res.success) { keyError = res.error || 'Save failed'; return; }
    step = 2;
    checkModel();
  }

  async function checkModel() {
    const res = await api.checkModelExists(selectedModel);
    modelExists = res.data ?? false;
  }

  async function downloadSelectedModel() {
    downloading = true;
    downloadError = '';
    downloadProgress = 0;

    const res = await api.downloadModel(selectedModel);
    downloading = false;

    if (res.success) {
      modelExists = true;
    } else {
      downloadError = res.error || 'Download failed';
    }
  }

  function finish() {
    onComplete?.();
  }
</script>

<div class="wizard">
  <!-- Header -->
  <div class="wizard-header">
    <div class="logo-row">
      <span class="logo-æ">Æ</span>
      <span class="logo-name">AetherEcho</span>
      <span class="badge badge-accent">Setup</span>
    </div>
    <p class="subtitle">AI-powered interview assistant · Private · Secure · Stealthy</p>
  </div>

  <!-- Step indicator -->
  <div class="steps-row">
    {#each Array(totalSteps) as _, i}
      <div class="step-pip" class:active={i + 1 === step} class:done={i + 1 < step}></div>
    {/each}
  </div>

  <!-- Step content -->
  <div class="wizard-body fade-in">
    {#if step === 1}
      <!-- Step 1: Groq API Key -->
      <div class="step-content">
        <div class="step-icon">🔑</div>
        <h2>Connect to Groq</h2>
        <p class="step-desc">
          AetherEcho uses Groq's lightning-fast inference to generate answers.
          Your API key is stored securely in Windows Credential Manager — never on disk.
        </p>

        <a
          href="https://console.groq.com/keys"
          class="get-key-link"
          target="_blank"
          rel="noopener"
        >Get a free API key at console.groq.com →</a>

        <div class="input-group">
          <label class="section-label">Groq API Key</label>
          <div class="input-with-btn">
            <input
              type={apiKeyVisible ? 'text' : 'password'}
              class="input input-password"
              placeholder="gsk_••••••••••••••••••••••••"
              bind:value={apiKey}
              autocomplete="off"
              spellcheck="false"
            />
            <button class="btn btn-secondary btn-sm" onclick={() => apiKeyVisible = !apiKeyVisible}>
              {apiKeyVisible ? '🙈' : '👁'}
            </button>
          </div>
        </div>

        {#if keyError}
          <div class="alert alert-error">{keyError}</div>
        {/if}
        {#if keyTestResult === 'valid'}
          <div class="alert alert-success">✅ API key is valid! Connection successful.</div>
        {/if}

        <div class="step-actions">
          <button class="btn btn-secondary" onclick={testKey} disabled={testingKey || !apiKey.trim()}>
            {testingKey ? '⌛ Testing…' : '🧪 Test Connection'}
          </button>
          <button
            class="btn btn-primary btn-lg"
            onclick={saveAndContinue}
            disabled={!apiKey.trim()}
          >
            Save & Continue →
          </button>
        </div>
      </div>

    {:else if step === 2}
      <!-- Step 2: Whisper Model -->
      <div class="step-content">
        <div class="step-icon">🧠</div>
        <h2>Local Transcription Model</h2>
        <p class="step-desc">
          AetherEcho transcribes audio locally on your device using Whisper.
          No audio ever leaves your computer.
        </p>

        <div class="model-cards">
          {#each [
            { id: 'tiny', name: 'Tiny', size: '75 MB', speed: 'Very Fast', quality: 'Good', recommended: false },
            { id: 'base', name: 'Base', size: '145 MB', speed: 'Fast', quality: 'Better', recommended: true },
          ] as m}
            <button
              class="model-card glass"
              class:selected={selectedModel === m.id}
              onclick={() => { selectedModel = m.id; checkModel(); }}
            >
              <div class="model-name">{m.name}</div>
              <div class="model-size text-muted">{m.size}</div>
              <div class="model-stats">
                <span class="badge badge-accent">{m.speed}</span>
                <span class="badge badge-success">{m.quality}</span>
              </div>
              {#if m.recommended}
                <div class="model-rec">Recommended</div>
              {/if}
            </button>
          {/each}
        </div>

        {#if modelExists}
          <div class="alert alert-success">✅ Model already downloaded!</div>
        {:else if downloading}
          <div class="download-progress">
            <p class="text-sm text-secondary">Downloading {selectedModel} model… {downloadProgress}%</p>
            <div class="progress-bar">
              <div class="progress-fill" style="width: {downloadProgress}%"></div>
            </div>
          </div>
        {:else}
          <button class="btn btn-primary w-full" onclick={downloadSelectedModel}>
            ⬇ Download {selectedModel === 'tiny' ? 'Tiny (75 MB)' : 'Base (145 MB)'} Model
          </button>
        {/if}

        {#if downloadError}
          <div class="alert alert-error">{downloadError}</div>
        {/if}

        <div class="step-actions">
          <button class="btn btn-secondary" onclick={() => step = 1}>← Back</button>
          <button
            class="btn btn-primary btn-lg"
            onclick={() => step = 3}
            disabled={!modelExists}
          >
            Continue →
          </button>
        </div>
      </div>

    {:else if step === 3}
      <!-- Step 3: Hotkeys -->
      <div class="step-content">
        <div class="step-icon">⌨️</div>
        <h2>Keyboard Shortcuts</h2>
        <p class="step-desc">
          These global hotkeys work even when AetherEcho is hidden in the background.
          You can change them later in Settings.
        </p>

        <div class="hotkey-list glass">
          {#each [
            { keys: 'Ctrl+Alt+Shift+O', action: 'Show / Hide the overlay window' },
            { keys: 'Ctrl+Alt+Shift+L', action: 'Start / Stop listening' },
          ] as hk}
            <div class="hotkey-row">
              <kbd class="kbd">{hk.keys}</kbd>
              <span class="text-secondary">{hk.action}</span>
            </div>
          {/each}
        </div>

        <div class="step-actions">
          <button class="btn btn-secondary" onclick={() => step = 2}>← Back</button>
          <button class="btn btn-primary btn-lg" onclick={() => step = 4}>Got it! →</button>
        </div>
      </div>

    {:else if step === 4}
      <!-- Step 4: Done -->
      <div class="step-content step-done">
        <div class="done-glow">
          <div class="done-icon">✨</div>
        </div>
        <h2>You're all set!</h2>
        <p class="step-desc">
          AetherEcho is ready. During an interview, press <kbd class="kbd-inline">Ctrl+Alt+Shift+L</kbd>
          to start listening, and your AI answers will appear in the overlay.
          The overlay is <strong>invisible to screen recorders</strong>.
        </p>

        <div class="tips glass">
          <div class="tip">💡 The overlay won't appear in Zoom, Teams, or OBS recordings</div>
          <div class="tip">💡 Press Ctrl+Alt+Shift+O to show/hide the answer overlay</div>
          <div class="tip">💡 Right-click the tray icon for quick controls</div>
        </div>

        <button class="btn btn-primary btn-lg w-full" onclick={finish}>
          Launch AetherEcho 🚀
        </button>
      </div>
    {/if}
  </div>
</div>

<style>
  .wizard {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: var(--bg-base);
    overflow-y: auto;
  }

  .wizard-header {
    padding: var(--space-xl) var(--space-xl) var(--space-md);
    background: linear-gradient(180deg, rgba(108,99,255,0.08) 0%, transparent 100%);
    border-bottom: 1px solid var(--border);
    text-align: center;
  }

  .logo-row {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 12px;
    margin-bottom: var(--space-sm);
  }

  .logo-æ {
    font-size: 40px;
    font-weight: 800;
    background: linear-gradient(135deg, var(--accent), #8b63ff);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }

  .logo-name {
    font-size: 26px;
    font-weight: 700;
    color: var(--text-primary);
    letter-spacing: -0.02em;
  }

  .subtitle { color: var(--text-secondary); font-size: var(--text-sm); }

  .steps-row {
    display: flex;
    gap: 8px;
    padding: var(--space-md) var(--space-xl);
    justify-content: center;
  }

  .step-pip {
    width: 32px; height: 4px;
    border-radius: var(--radius-full);
    background: var(--bg-elevated);
    transition: background var(--duration-normal) var(--ease-smooth);
  }
  .step-pip.active { background: var(--accent); }
  .step-pip.done { background: var(--success); }

  .wizard-body {
    flex: 1;
    padding: var(--space-xl);
    display: flex;
    justify-content: center;
  }

  .step-content {
    width: 100%;
    max-width: 480px;
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
  }

  .step-icon {
    font-size: 40px;
    margin-bottom: var(--space-xs);
  }

  h2 {
    font-size: var(--text-xl);
    font-weight: 700;
    color: var(--text-primary);
    letter-spacing: -0.02em;
  }

  .step-desc {
    color: var(--text-secondary);
    font-size: var(--text-sm);
    line-height: 1.6;
  }

  .get-key-link {
    color: var(--accent-light);
    font-size: var(--text-sm);
    text-decoration: none;
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }
  .get-key-link:hover { text-decoration: underline; }

  .input-group { display: flex; flex-direction: column; gap: var(--space-xs); }

  .input-with-btn { display: flex; gap: var(--space-sm); }
  .input-with-btn .input { flex: 1; }

  .step-actions {
    display: flex;
    gap: var(--space-sm);
    justify-content: flex-end;
    margin-top: var(--space-sm);
  }

  .alert {
    padding: 10px 14px;
    border-radius: var(--radius-md);
    font-size: var(--text-sm);
  }
  .alert-error  { background: var(--error-dim); color: var(--error); border: 1px solid rgba(244,63,94,0.2); }
  .alert-success { background: var(--success-dim); color: var(--success); border: 1px solid rgba(34,211,165,0.2); }

  .model-cards { display: grid; grid-template-columns: 1fr 1fr; gap: var(--space-sm); }

  .model-card {
    padding: var(--space-md);
    display: flex;
    flex-direction: column;
    gap: 8px;
    cursor: pointer;
    position: relative;
    border: 2px solid transparent;
    transition: all var(--duration-normal) var(--ease-smooth);
    background: none;
    color: inherit;
    text-align: left;
  }
  .model-card.selected { border-color: var(--accent); box-shadow: var(--shadow-accent); }
  .model-card:hover { border-color: var(--border-accent); }

  .model-name { font-size: var(--text-md); font-weight: 700; color: var(--text-primary); }
  .model-size { font-size: var(--text-xs); }
  .model-stats { display: flex; gap: 6px; flex-wrap: wrap; }

  .model-rec {
    position: absolute; top: -10px; right: 12px;
    background: var(--accent);
    color: white;
    font-size: 10px;
    font-weight: 700;
    padding: 2px 8px;
    border-radius: var(--radius-full);
    letter-spacing: 0.05em;
  }

  .download-progress { display: flex; flex-direction: column; gap: var(--space-sm); }

  .hotkey-list {
    display: flex;
    flex-direction: column;
    gap: 0;
    overflow: hidden;
  }
  .hotkey-row {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    padding: var(--space-md);
    border-bottom: 1px solid var(--border);
    font-size: var(--text-sm);
  }
  .hotkey-row:last-child { border-bottom: none; }

  .kbd {
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-bottom: 3px solid var(--border);
    padding: 4px 10px;
    border-radius: var(--radius-sm);
    color: var(--text-accent);
    white-space: nowrap;
  }

  .kbd-inline {
    font-family: var(--font-mono);
    font-size: 0.85em;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    padding: 2px 6px;
    border-radius: 4px;
    color: var(--text-accent);
  }

  .step-done { align-items: center; text-align: center; }

  .done-glow {
    width: 80px; height: 80px;
    background: radial-gradient(circle, var(--accent-dim) 0%, transparent 70%);
    display: flex; align-items: center; justify-content: center;
    border-radius: 50%;
    margin-bottom: var(--space-sm);
  }

  .done-icon { font-size: 40px; }

  .tips {
    display: flex;
    flex-direction: column;
    gap: 0;
    overflow: hidden;
    width: 100%;
    text-align: left;
  }

  .tip {
    padding: 12px var(--space-md);
    font-size: var(--text-sm);
    color: var(--text-secondary);
    border-bottom: 1px solid var(--border);
    line-height: 1.5;
  }
  .tip:last-child { border-bottom: none; }
</style>
