<script lang="ts">
  import { appStore, historyStore } from '../stores/app.svelte';
  import * as api from '../api';

  interface Props {
    onOpenSettings?: () => void;
    onToggleListening?: () => void;
  }
  let { onOpenSettings, onToggleListening }: Props = $props();

  let clearingHistory = $state(false);

  async function toggleListening() {
    onToggleListening?.();
  }

  async function toggleOverlay() {
    const res = await api.toggleOverlay();
    if (res.success && res.data !== undefined) {
      appStore.setOverlayVisible(res.data);
    }
  }

  function dismissError() {
    appStore.clearError();
  }

  function formatTime(ts: number): string {
    const d = new Date(ts);
    return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  }

  function truncate(text: string, maxLen = 80): string {
    return text.length > maxLen ? text.slice(0, maxLen) + '…' : text;
  }
</script>

<div class="main-screen">
  <!-- Top bar -->
  <header class="topbar">
    <div class="brand">
      <span class="logo-æ">Æ</span>
      <span class="brand-name">AetherEcho</span>
    </div>

    <div class="topbar-actions">
      <button
        class="btn btn-sm btn-secondary"
        class:active-overlay={appStore.overlayVisible}
        onclick={toggleOverlay}
        data-tooltip="Toggle overlay (Ctrl+Shift+A)"
      >
        {appStore.overlayVisible ? '🙈 Hide Overlay' : '👁 Overlay'}
      </button>

      <button
        class="btn btn-sm btn-secondary btn-icon"
        onclick={() => onOpenSettings?.()}
        data-tooltip="Settings"
      >⚙</button>
    </div>
  </header>

  <!-- Status banner -->
  {#if appStore.errorMsg}
    <div class="error-banner">
      <span>⚠ {appStore.errorMsg}</span>
      <button class="dismiss-btn" onclick={dismissError}>✕</button>
    </div>
  {/if}

  <!-- Main content -->
  <div class="content">
    <!-- Big listen button -->
    <div class="listen-card glass">
      <div class="status-row">
        <div class="status-dot {appStore.status}"></div>
        <span class="status-label">{appStore.statusLabel}</span>
      </div>

      <button
        class="listen-btn"
        class:listening={appStore.isListening}
        onclick={toggleListening}
        aria-label={appStore.isListening ? 'Stop listening' : 'Start listening'}
      >
        <div class="listen-rings">
          <div class="ring ring-1"></div>
          <div class="ring ring-2"></div>
          <div class="ring ring-3"></div>
        </div>
        <div class="listen-icon">
          {appStore.isListening ? '⏹' : '🎙'}
        </div>
      </button>

      <div class="listen-label">
        {appStore.isListening
          ? 'Capturing system audio — click to stop'
          : 'Click to start capturing system audio'}
      </div>

      <div class="hotkey-hint">
        <kbd class="kbd-sm">Ctrl+Alt+Shift+L</kbd> to toggle
      </div>
    </div>

    <!-- Live transcript + answer -->
    <div class="panels">
      <!-- Transcript panel -->
      <div class="panel glass">
        <div class="panel-header">
          <span class="section-label">Live Transcript</span>
          {#if appStore.status === 'transcribing' || appStore.status === 'listening'}
            <span class="badge badge-success text-xs">
              <span class="pulse-dot"></span> LIVE
            </span>
          {/if}
        </div>
        <div class="panel-body">
          {#if appStore.transcript}
            <p class="transcript-text">{appStore.transcript}</p>
          {:else}
            <p class="empty-text">Transcribed text will appear here…</p>
          {/if}
        </div>
      </div>

      <!-- AI Answer panel -->
      <div class="panel glass panel-answer">
        <div class="panel-header">
          <span class="section-label">AI Answer</span>
          {#if appStore.status === 'thinking'}
            <div class="thinking-dots">
              <span></span><span></span><span></span>
            </div>
          {/if}
        </div>
        <div class="panel-body">
          {#if appStore.answer}
            <p class="answer-text">{appStore.answer}</p>
          {:else}
            <p class="empty-text">AI answer will appear here after speech is detected…</p>
          {/if}
        </div>
      </div>
    </div>

    <!-- History -->
    {#if historyStore.items.length > 0}
      <div class="history-section">
        <div class="history-header">
          <span class="section-label">Recent History</span>
          <button class="btn btn-sm btn-secondary" onclick={() => historyStore.clear()}>
            Clear
          </button>
        </div>

        <div class="history-list">
          {#each historyStore.items as item (item.id)}
            <div class="history-item glass fade-in">
              <div class="history-meta">
                <span class="text-muted text-xs">{formatTime(item.timestamp)}</span>
              </div>
              <div class="history-q">
                <span class="text-xs text-secondary">Q: </span>
                <span class="text-sm">{truncate(item.transcript)}</span>
              </div>
              <div class="history-a">
                <span class="text-xs text-accent">A: </span>
                <span class="text-sm text-secondary">{truncate(item.answer)}</span>
              </div>
            </div>
          {/each}
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .main-screen {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: var(--bg-base);
    overflow: hidden;
  }

  /* Header */
  .topbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px var(--space-lg);
    background: var(--bg-surface);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
    -webkit-app-region: drag;
  }

  .topbar .btn, .topbar-actions {
    -webkit-app-region: no-drag;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .logo-æ {
    font-size: 24px;
    font-weight: 800;
    background: linear-gradient(135deg, var(--accent), #8b63ff);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
    line-height: 1;
  }

  .brand-name {
    font-size: var(--text-md);
    font-weight: 700;
    color: var(--text-primary);
    letter-spacing: -0.02em;
  }

  .topbar-actions {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
  }

  .active-overlay {
    border-color: var(--accent) !important;
    color: var(--accent-light) !important;
  }

  /* Error banner */
  .error-banner {
    background: var(--error-dim);
    border-bottom: 1px solid rgba(244,63,94,0.2);
    padding: 10px var(--space-lg);
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: var(--text-sm);
    color: var(--error);
    flex-shrink: 0;
  }

  .dismiss-btn {
    background: none;
    border: none;
    color: var(--error);
    cursor: pointer;
    opacity: 0.7;
    font-size: var(--text-base);
  }
  .dismiss-btn:hover { opacity: 1; }

  /* Content */
  .content {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-lg);
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
  }

  /* Listen card */
  .listen-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: var(--space-lg) var(--space-xl);
    gap: var(--space-md);
  }

  .status-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .status-label {
    font-size: var(--text-sm);
    font-weight: 600;
    color: var(--text-secondary);
    letter-spacing: 0.05em;
    text-transform: uppercase;
  }

  /* Listen button */
  .listen-btn {
    position: relative;
    width: 100px;
    height: 100px;
    border: none;
    background: none;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .listen-icon {
    font-size: 36px;
    position: relative;
    z-index: 2;
    transition: transform var(--duration-fast) var(--ease-spring);
  }

  .listen-btn:hover .listen-icon { transform: scale(1.1); }
  .listen-btn:active .listen-icon { transform: scale(0.95); }

  .listen-rings {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .ring {
    position: absolute;
    border-radius: 50%;
    border: 2px solid var(--accent);
    opacity: 0;
    transition: all var(--duration-slow) var(--ease-smooth);
  }

  .ring-1 { width: 72px; height: 72px; background: var(--accent-dim); }
  .ring-2 { width: 90px; height: 90px; }
  .ring-3 { width: 108px; height: 108px; }

  .listening .ring-1 { opacity: 0.9; animation: ring-pulse 2s infinite; }
  .listening .ring-2 { opacity: 0.4; animation: ring-pulse 2s 0.3s infinite; }
  .listening .ring-3 { opacity: 0.15; animation: ring-pulse 2s 0.6s infinite; }

  @keyframes ring-pulse {
    0%, 100% { transform: scale(0.95); opacity: 0.6; }
    50%       { transform: scale(1.05); opacity: 0.2; }
  }

  .listen-label {
    font-size: var(--text-sm);
    color: var(--text-secondary);
    text-align: center;
  }

  .hotkey-hint {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: var(--text-xs);
    color: var(--text-muted);
  }

  .kbd-sm {
    font-family: var(--font-mono);
    font-size: 10px;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    padding: 2px 6px;
    border-radius: 4px;
    color: var(--text-accent);
  }

  /* Panels */
  .panels {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-md);
  }

  @media (max-width: 600px) {
    .panels { grid-template-columns: 1fr; }
  }

  .panel {
    min-height: 140px;
    display: flex;
    flex-direction: column;
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-sm) var(--space-md);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .panel-body {
    padding: var(--space-md);
    flex: 1;
    overflow-y: auto;
    max-height: 160px;
  }

  .panel-answer { border: 1px solid var(--border-accent); }

  .transcript-text {
    font-size: var(--text-sm);
    color: var(--text-primary);
    line-height: 1.6;
    font-style: italic;
  }

  .answer-text {
    font-size: var(--text-sm);
    color: var(--text-primary);
    line-height: 1.7;
    white-space: pre-wrap;
  }

  .empty-text {
    color: var(--text-muted);
    font-size: var(--text-sm);
    font-style: italic;
  }

  .pulse-dot {
    display: inline-block;
    width: 6px; height: 6px;
    border-radius: 50%;
    background: var(--success);
    animation: pulse-green 1.5s infinite;
    margin-right: 2px;
  }

  /* Thinking dots */
  .thinking-dots {
    display: flex;
    gap: 4px;
    align-items: center;
  }

  .thinking-dots span {
    width: 5px; height: 5px;
    background: var(--accent);
    border-radius: 50%;
    animation: bounce 0.8s infinite;
  }

  .thinking-dots span:nth-child(2) { animation-delay: 0.15s; }
  .thinking-dots span:nth-child(3) { animation-delay: 0.3s; }

  @keyframes bounce {
    0%, 80%, 100% { transform: scale(0.5); }
    40%           { transform: scale(1); }
  }

  /* History */
  .history-section { display: flex; flex-direction: column; gap: var(--space-sm); }

  .history-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .history-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
    max-height: 280px;
    overflow-y: auto;
  }

  .history-item {
    padding: var(--space-sm) var(--space-md);
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .history-meta { margin-bottom: 2px; }
  .history-q, .history-a { display: flex; gap: 6px; align-items: flex-start; }
</style>
