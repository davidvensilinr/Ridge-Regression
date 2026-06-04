// AetherEcho — Global App Stores (Svelte 5 Runes)

import { emit } from '@tauri-apps/api/event';
import { type Settings } from '../api';


// ── App Status State ──────────────────────────────────────────────────────────

export type AppStatusType = 'idle' | 'listening' | 'transcribing' | 'thinking' | 'error';

let _isListening = $state(false);
let _status = $state<AppStatusType>('idle');
let _transcript = $state('');
let _answer = $state('');
let _overlayVisible = $state(false);
let _errorMsg = $state('');
let _hasApiKey = $state(false);
let _isSetupComplete = $state(false);

export const appStore = {
  get isListening() { return _isListening; },
  get status() { return _status; },
  get transcript() { return _transcript; },
  get answer() { return _answer; },
  get overlayVisible() { return _overlayVisible; },
  get errorMsg() { return _errorMsg; },
  get hasApiKey() { return _hasApiKey; },
  get isSetupComplete() { return _isSetupComplete; },

  setListening(v: boolean) { _isListening = v; },
  setStatus(v: AppStatusType) { _status = v; },
  setTranscript(v: string) { _transcript = v; },
  setAnswer(v: string) { _answer = v; },
  setOverlayVisible(v: boolean) { _overlayVisible = v; },
  setError(v: string) { _errorMsg = v; _status = 'error'; },
  clearError() { _errorMsg = ''; if (_status === 'error') _status = 'idle'; },
  setHasApiKey(v: boolean) { _hasApiKey = v; },
  setSetupComplete(v: boolean) { _isSetupComplete = v; },

  get statusLabel(): string {
    switch (_status) {
      case 'idle':         return 'Idle';
      case 'listening':    return 'Listening…';
      case 'transcribing': return 'Transcribing…';
      case 'thinking':     return 'Thinking…';
      case 'error':        return 'Error';
      default:             return 'Unknown';
    }
  },

  get statusColor(): string {
    switch (_status) {
      case 'listening':    return 'var(--success)';
      case 'transcribing': return 'var(--warning)';
      case 'thinking':     return 'var(--accent)';
      case 'error':        return 'var(--error)';
      default:             return 'var(--text-muted)';
    }
  }
};

// ── Conversation History ──────────────────────────────────────────────────────

export interface ConversationItem {
  id: string;
  transcript: string;
  answer: string;
  timestamp: number;
}

let _history = $state<ConversationItem[]>([]);

export const historyStore = {
  get items() { return _history; },

  add(transcript: string, answer: string) {
    _history = [
      { id: crypto.randomUUID(), transcript, answer, timestamp: Date.now() },
      ..._history,
    ].slice(0, 50); // Keep last 50 items
  },

  clear(sync = true) {
    _history = [];
    if (sync) {
      emit('clear-history').catch(console.error);
    }
  }
};
