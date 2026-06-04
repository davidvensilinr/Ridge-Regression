// AetherEcho — Frontend API Wrappers
// Type-safe wrappers around Tauri's invoke() calls.

import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

// ── Types ─────────────────────────────────────────────────────────────────────

export interface CommandResult<T> {
  success: boolean;
  data?: T;
  error?: string;
}

export interface Settings {
  model: string;
  overlay_opacity: number;
  hotkey_toggle_overlay: string;
  hotkey_toggle_listening: string;
  hotkey_toggle_click_through: string;
  hotkey_screenshot_solve: string;
  vad_sensitivity: number;
  vad_silence_timeout: number;
  auto_start_listening: boolean;
  whisper_model: string;
  system_prompt: string;
  overlay_click_through: boolean;
  theme: string;
}

export interface AudioDevice {
  id: string;
  name: string;
}

export interface ModelInfo {
  id: string;
  name: string;
  context_window: number;
}

export interface AppStatus {
  is_listening: boolean;
  status: 'idle' | 'listening' | 'transcribing' | 'thinking' | 'error';
  transcript: string;
  answer: string;
  overlay_visible: boolean;
  error?: string;
}

// ── API Key ───────────────────────────────────────────────────────────────────

export async function saveApiKey(key: string): Promise<CommandResult<void>> {
  return invoke('save_api_key', { key });
}

export async function getApiKey(): Promise<CommandResult<string | null>> {
  return invoke('get_api_key');
}

export async function hasApiKey(): Promise<boolean> {
  return invoke('has_api_key');
}

export async function deleteApiKey(): Promise<CommandResult<void>> {
  return invoke('delete_api_key');
}

export async function testApiKey(key: string): Promise<CommandResult<boolean>> {
  return invoke('test_api_key', { key });
}

// ── Listening ─────────────────────────────────────────────────────────────────

export async function startListening(): Promise<CommandResult<void>> {
  return invoke('start_listening');
}

export async function stopListening(): Promise<CommandResult<void>> {
  return invoke('stop_listening');
}

export async function getStatus(): Promise<CommandResult<AppStatus>> {
  return invoke('get_status');
}

// ── Overlay ───────────────────────────────────────────────────────────────────

export async function toggleOverlay(visible?: boolean): Promise<CommandResult<boolean>> {
  return invoke('toggle_overlay', { visible: visible ?? null });
}

export async function setOverlayOpacity(opacity: number): Promise<CommandResult<void>> {
  return invoke('set_overlay_opacity', { opacity });
}

export async function setOverlayClickThrough(clickThrough: boolean): Promise<CommandResult<void>> {
  return invoke('set_overlay_click_through', { clickThrough });
}

// ── Settings ──────────────────────────────────────────────────────────────────

export async function getSettings(): Promise<CommandResult<Settings>> {
  return invoke('get_settings');
}

export async function saveSettings(settings: Settings): Promise<CommandResult<void>> {
  return invoke('save_settings', { settings });
}

// ── Audio & Models ────────────────────────────────────────────────────────────

export async function getAudioDevices(): Promise<CommandResult<AudioDevice[]>> {
  return invoke('get_audio_devices');
}

export async function getAvailableModels(): Promise<CommandResult<ModelInfo[]>> {
  return invoke('get_available_models');
}

export async function checkModelExists(modelName: string): Promise<CommandResult<boolean>> {
  return invoke('check_model_exists', { modelName });
}

export async function downloadModel(modelName: string): Promise<CommandResult<string>> {
  return invoke('download_model', { modelName });
}

export async function showMainWindow(): Promise<void> {
  return invoke('show_main_window');
}

export async function solveScreenCapture(): Promise<CommandResult<string>> {
  return invoke('solve_screen_capture');
}

export async function solveScreenRegion(x: number, y: number, w: number, h: number): Promise<CommandResult<string>> {
  return invoke('solve_screen_region', { x, y, w, h });
}

// ── Event Listeners ───────────────────────────────────────────────────────────

export interface StatusChangedEvent {
  is_listening?: boolean;
  status: string;
  transcript?: string;
  error?: string;
}

export interface AnswerReadyEvent {
  transcript: string;
  answer: string;
}

export function onStatusChanged(cb: (e: StatusChangedEvent) => void): Promise<UnlistenFn> {
  return listen<StatusChangedEvent>('status-changed', (event) => cb(event.payload));
}

export function onTranscriptUpdated(cb: (text: string) => void): Promise<UnlistenFn> {
  return listen<string>('transcript-updated', (event) => cb(event.payload));
}

export function onAnswerReady(cb: (e: AnswerReadyEvent) => void): Promise<UnlistenFn> {
  return listen<AnswerReadyEvent>('answer-ready', (event) => cb(event.payload));
}

export function onPipelineError(cb: (msg: string) => void): Promise<UnlistenFn> {
  return listen<string>('pipeline-error', (event) => cb(event.payload));
}

export function onModelDownloadProgress(cb: (pct: number) => void): Promise<UnlistenFn> {
  return listen<number>('model-download-progress', (event) => cb(event.payload));
}

export function onOverlayVisibilityChanged(cb: (visible: boolean) => void): Promise<UnlistenFn> {
  return listen<boolean>('overlay-visibility-changed', (event) => cb(event.payload));
}

export function onHotkeyToggleListening(cb: () => void): Promise<UnlistenFn> {
  return listen('hotkey-toggle-listening', () => cb());
}

export function onTrayToggleListening(cb: () => void): Promise<UnlistenFn> {
  return listen('tray-toggle-listening', () => cb());
}

export function onTrayToggleOverlay(cb: () => void): Promise<UnlistenFn> {
  return listen('tray-toggle-overlay', () => cb());
}

export function onNavigateToSettings(cb: () => void): Promise<UnlistenFn> {
  return listen('navigate-to-settings', () => cb());
}
