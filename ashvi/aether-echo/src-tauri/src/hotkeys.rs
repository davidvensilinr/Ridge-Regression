// AetherEcho — Global Hotkeys
// Registers system-wide keyboard shortcuts that work even when the app is in background.

use anyhow::Result;
use tauri::{AppHandle, Emitter};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState, Shortcut};
use tauri_plugin_store::StoreExt;

use crate::state::AppState;

pub fn register_hotkeys(app: &AppHandle, state: AppState) -> Result<()> {
    let settings = {
        let lock = state.0.lock().unwrap();
        lock.settings.clone()
    };

    // Unregister all existing shortcuts first (safe to call even if none registered)
    let _ = app.global_shortcut().unregister_all();

    let toggle_overlay_key = settings.hotkey_toggle_overlay.clone();
    let toggle_listen_key = settings.hotkey_toggle_listening.clone();
    let toggle_click_through_key = settings.hotkey_toggle_click_through.clone();
    let screenshot_solve_key = settings.hotkey_screenshot_solve.clone();

    // Register overlay toggle
    if let Ok(shortcut) = toggle_overlay_key.parse::<Shortcut>() {
        let app_handle = app.clone();
        let state_clone = state.clone();
        let res = app.global_shortcut()
            .on_shortcut(shortcut.clone(), move |_app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    let visible = {
                        let mut lock = state_clone.0.lock().unwrap();
                        lock.overlay_visible = !lock.overlay_visible;
                        lock.overlay_visible
                    };
                    let _ = crate::overlay::set_overlay_visible(&app_handle, visible);
                    let _ = crate::tray::update_tray_overlay_state(&app_handle, visible);
                    let _ = app_handle.emit("overlay-visibility-changed", visible);
                }
            })
            .and_then(|_| app.global_shortcut().register(shortcut));
        if let Err(e) = res {
            log::warn!("Failed to register overlay toggle hotkey '{}': {e}", toggle_overlay_key);
        }
    }

    // Register listen toggle
    if let Ok(shortcut) = toggle_listen_key.parse::<Shortcut>() {
        let app_handle = app.clone();
        let res = app.global_shortcut()
            .on_shortcut(shortcut.clone(), move |_app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    let _ = app_handle.emit("hotkey-toggle-listening", ());
                }
            })
            .and_then(|_| app.global_shortcut().register(shortcut));
        if let Err(e) = res {
            log::warn!("Failed to register listening toggle hotkey '{}': {e}", toggle_listen_key);
        }
    }

    // Register click-through toggle
    if let Ok(shortcut) = toggle_click_through_key.parse::<Shortcut>() {
        let app_handle = app.clone();
        let state_clone = state.clone();
        let res = app.global_shortcut()
            .on_shortcut(shortcut.clone(), move |_app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    let next_click_through = {
                        let mut lock = state_clone.0.lock().unwrap();
                        lock.settings.overlay_click_through = !lock.settings.overlay_click_through;

                        // Persist to store
                        if let Ok(store) = app_handle.store("settings.json") {
                            if let Ok(val) = serde_json::to_value(&lock.settings) {
                                store.set("settings", val);
                                let _ = store.save();
                            }
                        }

                        lock.settings.overlay_click_through
                    };

                    let _ = crate::overlay::set_overlay_click_through(&app_handle, next_click_through);
                    let _ = app_handle.emit("click-through-toggled", next_click_through);
                }
            })
            .and_then(|_| app.global_shortcut().register(shortcut));
        if let Err(e) = res {
            log::warn!("Failed to register click-through toggle hotkey '{}': {e}", toggle_click_through_key);
        }
    }

    // Register screenshot solve
    if let Ok(shortcut) = screenshot_solve_key.parse::<Shortcut>() {
        let app_handle = app.clone();
        let state_clone = state.clone();
        let res = app.global_shortcut()
            .on_shortcut(shortcut.clone(), move |_app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    let app_clone = app_handle.clone();
                    let state_clone2 = state_clone.clone();
                    tauri::async_runtime::spawn(async move {
                        let _ = crate::commands::run_screen_capture_and_solve(&app_clone, state_clone2).await;
                    });
                }
            })
            .and_then(|_| app.global_shortcut().register(shortcut));
        if let Err(e) = res {
            log::warn!("Failed to register screenshot solve hotkey '{}': {e}", screenshot_solve_key);
        }
    }

    log::info!(
        "✅ Hotkeys registered: overlay={}, listen={}, click_through={}, screenshot_solve={}",
        settings.hotkey_toggle_overlay,
        settings.hotkey_toggle_listening,
        settings.hotkey_toggle_click_through,
        settings.hotkey_screenshot_solve
    );

    Ok(())
}

/// Re-register hotkeys after settings change.
pub fn reregister_hotkeys(app: &AppHandle, state: AppState) {
    if let Err(e) = register_hotkeys(app, state) {
        log::warn!("Failed to register hotkeys: {e}");
    }
}
