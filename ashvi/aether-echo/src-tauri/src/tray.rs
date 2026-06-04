// AetherEcho — System Tray
// Creates the system tray icon with a context menu for quick controls.

use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    AppHandle, Emitter, Manager, Result,
};

pub fn setup_tray(app: &AppHandle) -> Result<()> {
    TrayIconBuilder::with_id("main-tray")
        .tooltip("AetherEcho — AI Interview Assistant")
        .icon(app.default_window_icon().unwrap().clone())
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| {
            match event.id.as_ref() {
                "start_stop" => {
                    let _ = app.emit("tray-toggle-listening", ());
                }
                "toggle_overlay" => {
                    let _ = app.emit("tray-toggle-overlay", ());
                }
                "settings" => {
                    if let Some(win) = app.get_webview_window("main") {
                        let _ = win.show();
                        let _ = win.set_focus();
                        let _ = app.emit("navigate-to-settings", ());
                    }
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .build(app)?;

    update_tray_menu(app);
    log::info!("✅ System tray initialized");
    Ok(())
}

fn rebuild_tray_menu(app: &AppHandle, is_listening: bool, overlay_visible: bool) -> Result<()> {
    let separator = PredefinedMenuItem::separator(app)?;

    let start_stop_label = if is_listening {
        "⏹ Stop Listening"
    } else {
        "▶ Start Listening"
    };
    let toggle_overlay_label = if overlay_visible {
        "🙈 Hide Overlay"
    } else {
        "👁 Show Overlay"
    };

    let start_stop = MenuItem::with_id(app, "start_stop", start_stop_label, true, None::<&str>)?;
    let toggle_overlay = MenuItem::with_id(app, "toggle_overlay", toggle_overlay_label, true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "⚙ Settings", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "✕ Quit AetherEcho", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[
        &start_stop,
        &toggle_overlay,
        &separator,
        &settings,
        &separator,
        &quit,
    ])?;

    if let Some(tray) = app.tray_by_id("main-tray") {
        let _ = tray.set_menu(Some(menu));
    }
    Ok(())
}

pub fn update_tray_menu(app: &AppHandle) {
    if let Some(state) = app.try_state::<crate::state::AppState>() {
        let (is_listening, overlay_visible) = {
            if let Ok(lock) = state.0.lock() {
                (lock.is_listening, lock.overlay_visible)
            } else {
                (false, false)
            }
        };
        let _ = rebuild_tray_menu(app, is_listening, overlay_visible);
    }
}

pub fn update_tray_listening_state(app: &AppHandle, _is_listening: bool) {
    update_tray_menu(app);
}

pub fn update_tray_overlay_state(app: &AppHandle, _visible: bool) {
    update_tray_menu(app);
}
