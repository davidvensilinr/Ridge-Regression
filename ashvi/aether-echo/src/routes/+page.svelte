<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import * as api from '../lib/api';
  import { appStore, historyStore } from '../lib/stores/app.svelte';
  import SetupWizard from '../lib/components/SetupWizard.svelte';
  import MainScreen from '../lib/components/MainScreen.svelte';
  import SettingsPanel from '../lib/components/SettingsPanel.svelte';
  import '../app.css';

  let view = $state<'main' | 'settings'>('main');
  let loading = $state(true);

  onMount(async () => {
    // Check if API key is stored
    const hasKey = await api.hasApiKey();
    appStore.setHasApiKey(hasKey);
    appStore.setSetupComplete(hasKey);

    // Restore status from backend
    const statusRes = await api.getStatus();
    if (statusRes.success && statusRes.data) {
      const s = statusRes.data;
      appStore.setListening(s.is_listening);
      appStore.setStatus(s.status as any);
      appStore.setTranscript(s.transcript);
      appStore.setAnswer(s.answer);
      appStore.setOverlayVisible(s.overlay_visible);
    }

    // Subscribe to backend events
    await api.onStatusChanged((e) => {
      if (e.is_listening !== undefined) appStore.setListening(e.is_listening);
      if (e.status) appStore.setStatus(e.status as any);
      if (e.error) appStore.setError(e.error);
    });

    await api.onTranscriptUpdated((text) => {
      appStore.setTranscript(text);
    });

    await api.onAnswerReady((e) => {
      appStore.setAnswer(e.answer);
      historyStore.add(e.transcript, e.answer);
    });

    await api.onPipelineError((msg) => {
      appStore.setError(msg);
    });

    await api.onOverlayVisibilityChanged((visible) => {
      appStore.setOverlayVisible(visible);
    });

    await listen('clear-history', () => {
      historyStore.clear(false);
    });

    await api.onHotkeyToggleListening(() => {
      handleToggleListening();
    });

    await api.onTrayToggleListening(() => {
      handleToggleListening();
    });

    await api.onTrayToggleOverlay(() => {
      api.toggleOverlay();
    });

    await api.onNavigateToSettings(() => {
      view = 'settings';
    });

    loading = false;
  });

  async function handleToggleListening() {
    if (appStore.isListening) {
      const res = await api.stopListening();
      if (!res.success) appStore.setError(res.error || 'Stop failed');
    } else {
      const res = await api.startListening();
      if (!res.success) appStore.setError(res.error || 'Start failed');
    }
  }
</script>

{#if loading}
  <div class="splash">
    <div class="splash-logo">
      <span class="logo-ae">Æ</span>
      <span class="logo-text">AetherEcho</span>
    </div>
    <div class="splash-dots">
      <span></span><span></span><span></span>
    </div>
  </div>

{:else if !appStore.isSetupComplete}
  <SetupWizard onComplete={() => { appStore.setSetupComplete(true); appStore.setHasApiKey(true); }} />

{:else if view === 'settings'}
  <SettingsPanel onBack={() => view = 'main'} />

{:else}
  <MainScreen
    onOpenSettings={() => view = 'settings'}
    onToggleListening={handleToggleListening}
  />
{/if}

<style>
  .splash {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100vh;
    gap: 32px;
    background: var(--bg-base);
  }

  .splash-logo {
    display: flex;
    align-items: center;
    gap: 14px;
  }

  .logo-ae {
    font-size: 52px;
    font-weight: 800;
    background: linear-gradient(135deg, var(--accent), #8b63ff, var(--success));
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
    line-height: 1;
  }

  .logo-text {
    font-size: 28px;
    font-weight: 700;
    color: var(--text-primary);
    letter-spacing: -0.03em;
  }

  .splash-dots {
    display: flex;
    gap: 8px;
  }

  .splash-dots span {
    width: 8px; height: 8px;
    background: var(--accent);
    border-radius: 50%;
    animation: bounce 0.8s infinite;
  }

  .splash-dots span:nth-child(2) { animation-delay: 0.15s; }
  .splash-dots span:nth-child(3) { animation-delay: 0.3s; }

  @keyframes bounce {
    0%, 80%, 100% { transform: scale(0.6); opacity: 0.4; }
    40%           { transform: scale(1); opacity: 1; }
  }
</style>
