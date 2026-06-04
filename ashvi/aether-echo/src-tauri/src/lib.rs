// AetherEcho — Tauri Application Entry Point (lib.rs)
// Sets up the Tauri app with all plugins, state, tray, hotkeys, and overlay.

use tauri::Manager;
use tauri_plugin_store::StoreExt;

mod audio;
mod commands;
mod groq_client;
mod hotkeys;
mod keyring_store;
mod overlay;
mod pipeline;
mod state;
mod tray;
mod transcriber;
mod vad;

use state::{AppState, Settings};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = AppState::new();

    tauri::Builder::default()
        // ── Plugins ───────────────────────────────────────────────────────────
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        // ── Logging ───────────────────────────────────────────────────────────
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .build(),
        )
        // ── Shared State ──────────────────────────────────────────────────────
        .manage(app_state.clone())
        // ── Setup ─────────────────────────────────────────────────────────────
        .setup(move |app| {
            let handle = app.handle().clone();

            // ── Load persisted settings ───────────────────────────────────────
            let store = app.store("settings.json")?;
            if let Some(val) = store.get("settings") {
                if let Ok(settings) = serde_json::from_value::<Settings>(val.clone()) {
                    let mut lock = app_state.0.lock().unwrap();
                    lock.settings = settings;
                    log::info!("Settings loaded from store");
                }
            }

            // ── Set app data dir in state ─────────────────────────────────────
            let app_data = handle
                .path()
                .app_data_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();
            {
                let mut lock = app_state.0.lock().unwrap();
                let model_name = lock.settings.whisper_model.clone();
                let path = crate::transcriber::model_path(&app_data, &model_name);
                lock.model_path = Some(path);
            }

            // ── Set up overlay window ─────────────────────────────────────────
            if let Err(e) = overlay::setup_overlay_window(&handle) {
                log::warn!("Overlay setup warning: {e}");
            }

            // ── Apply initial overlay opacity ─────────────────────────────────
            let opacity = {
                let lock = app_state.0.lock().unwrap();
                lock.settings.overlay_opacity as f64
            };
            let _ = overlay::set_overlay_opacity(&handle, opacity);

            // ── Apply initial click-through ───────────────────────────────────
            let click_through = {
                let lock = app_state.0.lock().unwrap();
                lock.settings.overlay_click_through
            };
            let _ = overlay::set_overlay_click_through(&handle, click_through);

            // ── System tray ───────────────────────────────────────────────────
            tray::setup_tray(&handle)?;

            // ── Global hotkeys ────────────────────────────────────────────────
            hotkeys::register_hotkeys(&handle, app_state.clone())
                .unwrap_or_else(|e| log::warn!("Hotkey registration failed: {e}"));

            // ── Auto-start listening if configured ────────────────────────────
            let auto_start = {
                let lock = app_state.0.lock().unwrap();
                lock.settings.auto_start_listening
            };
            if auto_start && keyring_store::has_api_key() {
                let handle2 = handle.clone();
                let state2 = app_state.clone();
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    let api_key = match keyring_store::load_api_key() {
                        Ok(Some(k)) => k,
                        _ => return,
                    };
                    let app_data2 = handle2
                        .path()
                        .app_data_dir()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_default();
                    let (model_path, settings) = {
                        let lock = state2.0.lock().unwrap();
                        let model_name = &lock.settings.whisper_model;
                        (
                            crate::transcriber::model_path(&app_data2, model_name),
                            lock.settings.clone(),
                        )
                    };
                    if std::path::Path::new(&model_path).exists() {
                        let _ = pipeline::start_pipeline(
                            handle2.clone(),
                            state2,
                            api_key,
                            model_path,
                            settings,
                        )
                        .await;
                        tray::update_tray_listening_state(&handle2, true);
                    }
                });
            }

            log::info!("✅ AetherEcho setup complete");
            Ok(())
        })
        // ── Window events ──────────────────────────────────────────────────────
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // Hide main window instead of quitting (live in tray)
                if window.label() == "main" {
                    window.hide().ok();
                    api.prevent_close();
                }
            }
        })
        // ── IPC Commands ───────────────────────────────────────────────────────
        .invoke_handler(tauri::generate_handler![
            commands::save_api_key,
            commands::get_api_key,
            commands::has_api_key,
            commands::delete_api_key,
            commands::test_api_key,
            commands::start_listening,
            commands::stop_listening,
            commands::get_status,
            commands::toggle_overlay,
            commands::set_overlay_opacity,
            commands::set_overlay_click_through,
            commands::get_settings,
            commands::save_settings,
            commands::get_audio_devices,
            commands::get_available_models,
            commands::check_model_exists,
            commands::download_model,
            commands::show_main_window,
            commands::solve_screen_capture,
            commands::solve_screen_region,
        ])
        .run(tauri::generate_context!())
        .expect("error while running AetherEcho");
}
