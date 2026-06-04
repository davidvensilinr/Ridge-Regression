<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { listen, emit } from '@tauri-apps/api/event';
  import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
  import * as api from '../../lib/api';
  import '../../app.css';

  interface HistoryItem {
    id: string;
    transcript: string;
    answer: string;
  }

  let transcript = $state('');
  let answer = $state('');
  let isListening = $state(false);
  let status = $state('idle');
  let collapsed = $state(false);
  let dragging = $state(false);
  let history = $state<HistoryItem[]>([]);
  let clickThrough = $state(true);
  let solvingScreen = $state(false);
  let errorMessage = $state('');

  let bodyEl = $state<HTMLDivElement | null>(null);

  // Region Selection State
  let isSelecting = $state(false);
  let selectionStart = $state({ x: 0, y: 0 });
  let selectionEnd = $state({ x: 0, y: 0 });
  let isDragging = $state(false);
  let cachedClickThrough = true;

  const win = getCurrentWebviewWindow();

  async function triggerSolve() {
    if (solvingScreen) return;
    solvingScreen = true;
    errorMessage = '';
    try {
      const res = await api.solveScreenCapture();
      if (!res.success) {
        errorMessage = res.error || 'Failed to capture or solve screen';
        setTimeout(() => { errorMessage = ''; }, 6000);
      }
    } catch (err: any) {
      console.error("Capture trigger error:", err);
      errorMessage = err?.message || err || 'An unexpected error occurred';
      setTimeout(() => { errorMessage = ''; }, 6000);
    } finally {
      solvingScreen = false;
    }
  }

  async function startSelection() {
    if (solvingScreen || isSelecting) return;
    isSelecting = true;
    isDragging = false;
    errorMessage = '';

    cachedClickThrough = clickThrough;
    await api.setOverlayClickThrough(false);
    await win.setFocus();
    await new Promise(r => setTimeout(r, 80));
    await win.setFullscreen(true);
  }

  async function cancelSelection() {
    if (!isSelecting) return;
    isSelecting = false;
    isDragging = false;

    await win.setFullscreen(false);
    await new Promise(r => setTimeout(r, 80));
    await api.setOverlayClickThrough(cachedClickThrough);
  }

  async function endSelection() {
    if (!isSelecting) return;
    isSelecting = false;
    isDragging = false;

    await win.setFullscreen(false);
    await new Promise(r => setTimeout(r, 80));
    await api.setOverlayClickThrough(cachedClickThrough);

    const x = Math.min(selectionStart.x, selectionEnd.x);
    const y = Math.min(selectionStart.y, selectionEnd.y);
    const w = Math.abs(selectionStart.x - selectionEnd.x);
    const h = Math.abs(selectionStart.y - selectionEnd.y);

    if (w > 5 && h > 5) {
      solvingScreen = true;
      try {
        const res = await api.solveScreenRegion(x, y, w, h);
        if (!res.success) {
          errorMessage = res.error || 'Failed to capture or solve screen region';
          setTimeout(() => { errorMessage = ''; }, 6000);
        }
      } catch (err: any) {
        console.error("Region capture trigger error:", err);
        errorMessage = err?.message || err || 'An unexpected error occurred';
        setTimeout(() => { errorMessage = ''; }, 6000);
      } finally {
        solvingScreen = false;
      }
    }
  }

  function handleMouseDown(e: MouseEvent) {
    if (!isSelecting) return;
    isDragging = true;
    selectionStart = { x: e.clientX, y: e.clientY };
    selectionEnd = { x: e.clientX, y: e.clientY };
  }

  function handleMouseMove(e: MouseEvent) {
    if (!isSelecting || !isDragging) return;
    selectionEnd = { x: e.clientX, y: e.clientY };
  }

  function handleMouseUp(e: MouseEvent) {
    if (!isSelecting || !isDragging) return;
    selectionEnd = { x: e.clientX, y: e.clientY };
    endSelection();
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Escape' && isSelecting) {
      cancelSelection();
    }
  }

  // Auto-scroll overlay content to the bottom on new updates
  $effect(() => {
    // depend on these variables to run when they change
    const _t = transcript;
    const _a = answer;
    const _s = status;
    const _h = history;
    const _e = errorMessage;

    tick().then(() => {
      if (bodyEl) {
        bodyEl.scrollTop = bodyEl.scrollHeight;
      }
    });
  });

  onMount(async () => {
    const win = getCurrentWebviewWindow();

    // Listen for status changes
    await listen<any>('status-changed', (e) => {
      const p = e.payload;
      if (p.is_listening !== undefined) isListening = p.is_listening;
      if (p.status) status = p.status;
    });

    await listen<string>('transcript-updated', (e) => {
      transcript = e.payload;
      answer = ''; // Clear stale answer for the new incoming question
    });

    await listen<any>('answer-ready', (e) => {
      // Add the completed Q&A to history
      history = [
        ...history,
        {
          id: crypto.randomUUID(),
          transcript: e.payload.transcript,
          answer: e.payload.answer,
        }
      ].slice(-50); // Keep last 50 items

      // Reset active display since it's now in history
      transcript = '';
      answer = '';
    });

    await listen<string>('pipeline-error', (e) => {
      errorMessage = e.payload;
      setTimeout(() => { errorMessage = ''; }, 6000);
    });

    await listen('clear-history', () => {
      history = [];
      transcript = '';
      answer = '';
    });

    await listen<boolean>('click-through-toggled', (e) => {
      clickThrough = e.payload;
    });

    // Fetch initial click-through state from saved settings
    api.getSettings().then((res) => {
      if (res.success && res.data) {
        clickThrough = res.data.overlay_click_through;
      }
    });

    // Allow dragging the overlay
    const header = document.getElementById('overlay-drag-handle');
    if (header) {
      header.addEventListener('mousedown', (e) => {
        // Prevent dragging if clicking a button or something inside a button
        const target = e.target as HTMLElement;
        if (target.closest('button') || target.closest('.lock-indicator')) {
          return;
        }
        win.startDragging();
      });
    }
  });

  function getStatusDot() {
    switch (status) {
      case 'listening':    return '🟢';
      case 'transcribing': return '🟡';
      case 'thinking':     return '🔵';
      default:             return '⚪';
    }
  }

  function getStatusText() {
    switch (status) {
      case 'listening':    return 'Listening…';
      case 'transcribing': return 'Transcribing…';
      case 'thinking':     return 'Thinking…';
      default:             return 'Idle';
    }
  }
</script>

<svelte:window onkeydown={handleKeyDown} />

{#if !isSelecting}
  <div class="overlay-root" class:collapsed>
    <!-- Drag handle / status bar -->
    <div class="overlay-header" id="overlay-drag-handle">
      <div class="status-indicator">
        <span class="status-emoji">{getStatusDot()}</span>
        <span class="status-text">{getStatusText()}</span>
        <span
          class="lock-indicator"
          class:interactive={!clickThrough}
          title={clickThrough ? "Click-Through Mode (Locked) — Press Ctrl+Alt+Shift+K to unlock and scroll overlay" : "Interactive Mode (Scrollable) — Hover/scroll mouse. Press Ctrl+Alt+Shift+K to lock"}
        >
          {clickThrough ? '🔒' : '🔓'}
        </span>
      </div>

      <div class="header-actions">
        <button
          class="solve-btn"
          class:solving={solvingScreen}
          onclick={triggerSolve}
          onmousedown={(e) => e.stopPropagation()}
          title="Solve Screen Capture (Ctrl+Alt+Shift+S)"
          disabled={solvingScreen || isSelecting}
        >
          {solvingScreen ? '⏳' : '📸'}
        </button>
        <button
          class="select-btn"
          class:selecting={isSelecting}
          onclick={startSelection}
          onmousedown={(e) => e.stopPropagation()}
          title="Select Region to Solve (✂️)"
          disabled={solvingScreen || isSelecting}
        >
          ✂️
        </button>
        {#if history.length > 0}
          <button
            class="clear-btn"
            onclick={() => emit('clear-history')}
            onmousedown={(e) => e.stopPropagation()}
            title="Clear history"
          >
            🗑
          </button>
        {/if}
        <button
          class="collapse-btn"
          onclick={() => collapsed = !collapsed}
          onmousedown={(e) => e.stopPropagation()}
          aria-label={collapsed ? 'Expand' : 'Collapse'}
        >
          {collapsed ? '▼' : '▲'}
        </button>
      </div>
    </div>

    {#if !collapsed}
      <div class="overlay-body" bind:this={bodyEl}>
        {#if errorMessage}
          <div class="error-banner">
            ⚠️ {errorMessage}
          </div>
        {/if}
        <!-- Past Q&A History Items -->
        {#each history as item (item.id)}
          <div class="history-item-block">
            <div class="q-section">
              <div class="q-label">Q</div>
              <p class="q-text">{item.transcript}</p>
            </div>
            <div class="a-section">
              <div class="a-label">A</div>
              <p class="a-text">{item.answer}</p>
            </div>
          </div>
        {/each}

        <!-- Current / Active question being transcribed -->
        {#if transcript}
          <div class="q-section active-q">
            <div class="q-label">Q</div>
            <p class="q-text">{transcript}</p>
          </div>
        {/if}

        <!-- AI thinking indicator -->
        {#if status === 'thinking'}
          <div class="thinking">
            <div class="thinking-dots-overlay">
              <span></span><span></span><span></span>
            </div>
            <span class="thinking-label">
              {transcript === '[Visual Solve]' ? 'Analyzing & solving screen…' : 'Generating answer…'}
            </span>
          </div>
        {/if}

        <!-- Placeholder when nothing is present -->
        {#if history.length === 0 && !transcript && status !== 'thinking'}
          <p class="placeholder">
            {isListening
              ? 'Listening for speech…'
              : 'Start listening to see answers here'}
          </p>
        {/if}
      </div>
    {/if}
  </div>
{/if}

{#if isSelecting}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="selection-overlay"
    onmousedown={handleMouseDown}
    onmousemove={handleMouseMove}
    onmouseup={handleMouseUp}
  >
    <div class="selection-instructions">
      Click and drag to select screen region. Press <strong>Esc</strong> to cancel.
    </div>
    {#if isDragging}
      <div
        class="selection-rect"
        style="
          left: {Math.min(selectionStart.x, selectionEnd.x)}px;
          top: {Math.min(selectionStart.y, selectionEnd.y)}px;
          width: {Math.abs(selectionStart.x - selectionEnd.x)}px;
          height: {Math.abs(selectionStart.y - selectionEnd.y)}px;
        "
      ></div>
    {/if}
  </div>
{/if}

<style>
  :global(html, body, #app) {
    background: transparent !important;
    overflow: hidden;
    margin: 0;
    padding: 0;
    width: 100% !important;
    height: 100% !important;
  }

  .overlay-root {
    font-family: 'Inter', system-ui, sans-serif;
    background: rgba(10, 10, 20, 0.85);
    backdrop-filter: blur(24px) saturate(200%);
    -webkit-backdrop-filter: blur(24px) saturate(200%);
    border: 1px solid rgba(108, 99, 255, 0.3);
    border-radius: 14px;
    box-shadow: 0 8px 32px rgba(0,0,0,0.6), 0 0 0 1px rgba(255,255,255,0.04) inset;
    overflow: hidden;
    color: #f0f0ff;
    min-width: 320px;
    max-width: 100vw;
    width: 100%;
    display: flex;
    flex-direction: column;
    height: 100vh;
    box-sizing: border-box;
    transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
    -webkit-font-smoothing: antialiased;
  }

  .overlay-root.collapsed {
    border-radius: 10px;
    height: auto;
  }

  /* Header */
  .overlay-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 14px;
    background: rgba(26, 26, 40, 0.6);
    border-bottom: 1px solid rgba(255,255,255,0.06);
    cursor: move;
    user-select: none;
    flex-shrink: 0;
  }

  .status-indicator {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .status-emoji { font-size: 11px; line-height: 1; }

  .status-text {
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: #8888aa;
  }

  .collapse-btn {
    background: none;
    border: none;
    color: #8888aa;
    cursor: pointer;
    font-size: 10px;
    padding: 2px 6px;
    border-radius: 4px;
    transition: color 0.12s;
    user-select: none;
  }
  .collapse-btn:hover { color: #f0f0ff; }

  /* Body */
  .overlay-body {
    padding: 12px 14px;
    display: flex;
    flex-direction: column;
    gap: 10px;
    flex: 1;
    overflow-y: auto;
  }

  /* Q section */
  .q-section {
    display: flex;
    gap: 10px;
    align-items: flex-start;
  }

  .q-label {
    font-size: 10px;
    font-weight: 800;
    letter-spacing: 0.1em;
    color: #6c63ff;
    background: rgba(108,99,255,0.15);
    border: 1px solid rgba(108,99,255,0.3);
    border-radius: 4px;
    padding: 2px 6px;
    flex-shrink: 0;
    margin-top: 2px;
  }

  .q-text {
    font-size: 13px;
    color: #aaaacc;
    line-height: 1.5;
    font-style: italic;
    margin: 0;
  }

  /* A section */
  .a-section {
    display: flex;
    gap: 10px;
    align-items: flex-start;
    background: rgba(108,99,255,0.07);
    border: 1px solid rgba(108,99,255,0.15);
    border-radius: 8px;
    padding: 10px;
  }

  .a-label {
    font-size: 10px;
    font-weight: 800;
    letter-spacing: 0.1em;
    color: #22d3a5;
    background: rgba(34,211,165,0.15);
    border: 1px solid rgba(34,211,165,0.3);
    border-radius: 4px;
    padding: 2px 6px;
    flex-shrink: 0;
    margin-top: 2px;
  }

  .a-text {
    font-size: 13px;
    color: #e0e0f0;
    line-height: 1.65;
    white-space: pre-wrap;
    margin: 0;
  }

  /* Thinking */
  .thinking {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 4px;
  }

  .thinking-dots-overlay {
    display: flex;
    gap: 4px;
  }

  .thinking-dots-overlay span {
    width: 6px; height: 6px;
    background: #6c63ff;
    border-radius: 50%;
    animation: bounce-ov 0.8s infinite;
  }

  .thinking-dots-overlay span:nth-child(2) { animation-delay: 0.15s; }
  .thinking-dots-overlay span:nth-child(3) { animation-delay: 0.3s; }

  @keyframes bounce-ov {
    0%, 80%, 100% { transform: scale(0.5); opacity: 0.4; }
    40%           { transform: scale(1); opacity: 1; }
  }

  .thinking-label {
    font-size: 12px;
    color: #6c63ff;
    font-weight: 500;
  }

  /* Placeholder */
  .placeholder {
    font-size: 12px;
    color: #555570;
    text-align: center;
    padding: 8px 0;
    margin: 0;
    font-style: italic;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .clear-btn {
    background: none;
    border: none;
    color: #8888aa;
    cursor: pointer;
    font-size: 11px;
    padding: 2px 6px;
    border-radius: 4px;
    transition: color 0.12s;
    user-select: none;
  }
  .clear-btn:hover {
    color: #f43f5e;
  }

  .solve-btn {
    background: none;
    border: none;
    color: #8888aa;
    cursor: pointer;
    font-size: 11px;
    padding: 2px 6px;
    border-radius: 4px;
    transition: color 0.12s, transform 0.12s;
    user-select: none;
  }
  .solve-btn:hover {
    color: #38bdf8;
    transform: scale(1.1);
  }
  .solve-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .select-btn {
    background: none;
    border: none;
    color: #8888aa;
    cursor: pointer;
    font-size: 11px;
    padding: 2px 6px;
    border-radius: 4px;
    transition: color 0.12s, transform 0.12s;
    user-select: none;
  }
  .select-btn:hover {
    color: #c084fc;
    transform: scale(1.1);
  }
  .select-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .select-btn.selecting {
    color: #c084fc;
    animation: pulse-select 1.5s infinite ease-in-out;
  }
  @keyframes pulse-select {
    0%, 100% { opacity: 0.6; }
    50% { opacity: 1; }
  }

  .history-item-block {
    display: flex;
    flex-direction: column;
    gap: 10px;
    border-bottom: 1px dashed rgba(255, 255, 255, 0.05);
    padding-bottom: 12px;
    margin-bottom: 4px;
  }
  .history-item-block:last-of-type {
    border-bottom: none;
    padding-bottom: 0;
    margin-bottom: 0;
  }

  .lock-indicator {
    font-size: 11px;
    cursor: help;
    opacity: 0.4;
    transition: opacity 0.15s, transform 0.15s;
    margin-left: 2px;
    user-select: none;
  }
  .lock-indicator.interactive {
    opacity: 1;
    transform: scale(1.15);
    filter: drop-shadow(0 0 4px rgba(34, 211, 165, 0.4));
  }
  .error-banner {
    background: rgba(244, 63, 94, 0.2);
    border: 1px solid rgba(244, 63, 94, 0.4);
    color: #f43f5e;
    font-size: 12px;
    padding: 8px 12px;
    border-radius: 8px;
    text-align: center;
    margin-bottom: 8px;
    font-weight: 500;
    word-break: break-word;
  }



  .selection-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.1);
    z-index: 999999;
    cursor: crosshair;
    user-select: none;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: flex-start;
    padding-top: 10vh;
    pointer-events: auto;
  }

  .selection-instructions {
    background: rgba(15, 15, 25, 0.95);
    border: 1px solid rgba(108, 99, 255, 0.4);
    color: #e0e0f0;
    font-size: 13px;
    padding: 10px 20px;
    border-radius: 8px;
    box-shadow: 0 4px 20px rgba(0,0,0,0.6);
    pointer-events: none;
    font-weight: 500;
    letter-spacing: 0.02em;
  }

  .selection-rect {
    position: absolute;
    border: 2px solid #38bdf8;
    background: rgba(56, 189, 248, 0.12);
    box-shadow: 0 0 0 9999px rgba(0, 0, 0, 0.45);
    box-sizing: border-box;
    pointer-events: none;
  }
</style>
