// AetherEcho — Tauri IPC Commands
// All commands exposed to the frontend via invoke().
// Every command validates input and returns typed results.

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_store::StoreExt;

use crate::{
    audio, groq_client, keyring_store, overlay, pipeline,
    state::{AppState, Settings},
    tray,
};

// ── Generic Response ──────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct CommandResult<T: Serialize> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T: Serialize> CommandResult<T> {
    pub fn ok(data: T) -> Self {
        Self { success: true, data: Some(data), error: None }
    }
    pub fn err(msg: impl ToString) -> Self {
        Self { success: false, data: None, error: Some(msg.to_string()) }
    }
}

// ── API Key Commands ──────────────────────────────────────────────────────────

#[tauri::command]
pub async fn save_api_key(key: String) -> CommandResult<()> {
    if key.trim().is_empty() {
        return CommandResult::err("API key cannot be empty");
    }
    match keyring_store::save_api_key(key.trim()) {
        Ok(_) => CommandResult::ok(()),
        Err(e) => CommandResult::err(format!("Failed to save API key: {e}")),
    }
}

#[tauri::command]
pub async fn get_api_key() -> CommandResult<Option<String>> {
    match keyring_store::load_api_key() {
        Ok(key) => CommandResult::ok(key),
        Err(e) => CommandResult::err(format!("Failed to load API key: {e}")),
    }
}

#[tauri::command]
pub async fn has_api_key() -> bool {
    keyring_store::has_api_key()
}

#[tauri::command]
pub async fn delete_api_key() -> CommandResult<()> {
    match keyring_store::delete_api_key() {
        Ok(_) => CommandResult::ok(()),
        Err(e) => CommandResult::err(format!("Failed to delete API key: {e}")),
    }
}

#[tauri::command]
pub async fn test_api_key(key: String) -> CommandResult<bool> {
    if key.trim().is_empty() {
        return CommandResult::err("API key cannot be empty");
    }
    match groq_client::GroqClient::test_connection(key.trim()).await {
        Ok(valid) => CommandResult::ok(valid),
        Err(e) => CommandResult::err(format!("Connection test failed: {e}")),
    }
}

// ── Listening Commands ────────────────────────────────────────────────────────

#[tauri::command]
pub async fn start_listening(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<CommandResult<()>, String> {
    let api_key = match keyring_store::load_api_key() {
        Ok(Some(k)) => k,
        Ok(None) => return Ok(CommandResult::err("No API key configured — please add your Groq API key in Settings")),
        Err(e) => return Ok(CommandResult::err(format!("Failed to load API key: {e}"))),
    };

    // Check if model exists
    let (model_path, settings) = {
        let lock = state.0.lock().unwrap();
        let app_data = app
            .path()
            .app_data_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        let model_name = lock.settings.whisper_model.clone();
        let path = crate::transcriber::model_path(&app_data, &model_name);
        (path, lock.settings.clone())
    };

    if !std::path::Path::new(&model_path).exists() {
        return Ok(CommandResult::err(
            "Whisper model not found. Please download a model in Settings → Model."
        ));
    }

    // Check already listening
    {
        let lock = state.0.lock().unwrap();
        if lock.is_listening {
            return Ok(CommandResult::err("Already listening"));
        }
    }

    match pipeline::start_pipeline(app.clone(), state.inner().clone(), api_key, model_path, settings).await {
        Ok(_) => {
            tray::update_tray_listening_state(&app, true);
            Ok(CommandResult::ok(()))
        }
        Err(e) => Ok(CommandResult::err(format!("Failed to start pipeline: {e}"))),
    }
}

#[tauri::command]
pub async fn stop_listening(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<CommandResult<()>, String> {
    pipeline::stop_pipeline(state.inner().clone());
    tray::update_tray_listening_state(&app, false);
    Ok(CommandResult::ok(()))
}

#[tauri::command]
pub async fn get_status(state: State<'_, AppState>) -> Result<CommandResult<serde_json::Value>, String> {
    let lock = state.0.lock().unwrap();
    let status_json = serde_json::json!({
        "is_listening": lock.is_listening,
        "status": lock.status,
        "transcript": lock.current_transcript,
        "answer": lock.current_answer,
        "overlay_visible": lock.overlay_visible,
    });
    Ok(CommandResult::ok(status_json))
}

// ── Overlay Commands ──────────────────────────────────────────────────────────

#[tauri::command]
pub async fn toggle_overlay(
    app: AppHandle,
    state: State<'_, AppState>,
    visible: Option<bool>,
) -> Result<CommandResult<bool>, String> {
    let new_visible = {
        let mut lock = state.0.lock().unwrap();
        let next = visible.unwrap_or(!lock.overlay_visible);
        lock.overlay_visible = next;
        next
    };

    if let Err(e) = overlay::set_overlay_visible(&app, new_visible) {
        return Ok(CommandResult::err(format!("Overlay error: {e}")));
    }

    tray::update_tray_overlay_state(&app, new_visible);
    let _ = app.emit("overlay-visibility-changed", new_visible);
    Ok(CommandResult::ok(new_visible))
}

#[tauri::command]
pub async fn set_overlay_opacity(
    app: AppHandle,
    opacity: f64,
) -> CommandResult<()> {
    match overlay::set_overlay_opacity(&app, opacity) {
        Ok(_) => CommandResult::ok(()),
        Err(e) => CommandResult::err(format!("Failed to set opacity: {e}")),
    }
}

#[tauri::command]
pub async fn set_overlay_click_through(
    app: AppHandle,
    click_through: bool,
) -> CommandResult<()> {
    match overlay::set_overlay_click_through(&app, click_through) {
        Ok(_) => CommandResult::ok(()),
        Err(e) => CommandResult::err(format!("Failed to set click-through: {e}")),
    }
}

// ── Settings Commands ─────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<CommandResult<Settings>, String> {
    let lock = state.0.lock().unwrap();
    Ok(CommandResult::ok(lock.settings.clone()))
}

#[tauri::command]
pub async fn save_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    settings: Settings,
) -> Result<CommandResult<()>, String> {
    // Persist to tauri-plugin-store
    let store = match app.store("settings.json") {
        Ok(s) => s,
        Err(e) => return Ok(CommandResult::err(format!("Store error: {e}"))),
    };

    let val = match serde_json::to_value(&settings) {
        Ok(v) => v,
        Err(e) => return Ok(CommandResult::err(format!("Serialize error: {e}"))),
    };

    store.set("settings", val);
    if let Err(e) = store.save() {
        return Ok(CommandResult::err(format!("Failed to persist settings: {e}")));
    }

    // Apply opacity immediately
    let opacity = settings.overlay_opacity as f64;
    let _ = overlay::set_overlay_opacity(&app, opacity);
    let _ = overlay::set_overlay_click_through(&app, settings.overlay_click_through);

    // Update in-memory state and re-register hotkeys
    {
        let mut lock = state.0.lock().unwrap();
        lock.settings = settings;
    }

    crate::hotkeys::reregister_hotkeys(&app, state.inner().clone());

    Ok(CommandResult::ok(()))
}

// ── Audio Device Commands ─────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
}

#[tauri::command]
pub async fn get_audio_devices() -> CommandResult<Vec<AudioDevice>> {
    let devices = audio::list_audio_output_devices()
        .into_iter()
        .map(|(id, name)| AudioDevice { id, name })
        .collect();
    CommandResult::ok(devices)
}

// ── Model Commands ────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_available_models() -> CommandResult<Vec<groq_client::ModelInfo>> {
    CommandResult::ok(groq_client::available_models())
}

#[tauri::command]
pub async fn check_model_exists(
    app: AppHandle,
    model_name: String,
) -> CommandResult<bool> {
    let app_data = app
        .path()
        .app_data_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    let exists = crate::transcriber::model_exists(&app_data, &model_name);
    CommandResult::ok(exists)
}

#[tauri::command]
pub async fn download_model(
    app: AppHandle,
    model_name: String,
) -> CommandResult<String> {
    let app_data = app
        .path()
        .app_data_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    let app_clone = app.clone();
    let result = crate::transcriber::download_model(&app_data, &model_name, move |pct| {
        let _ = app_clone.emit("model-download-progress", pct);
    })
    .await;

    match result {
        Ok(path) => CommandResult::ok(path),
        Err(e) => CommandResult::err(format!("Download failed: {e}")),
    }
}

// ── Window Commands ───────────────────────────────────────────────────────────

#[tauri::command]
pub async fn show_main_window(app: AppHandle) -> CommandResult<()> {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.show();
        let _ = win.set_focus();
    }
    CommandResult::ok(())
}

#[tauri::command]
pub async fn solve_screen_capture(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<CommandResult<String>, String> {
    match run_screen_capture_and_solve(&app, state.inner().clone()).await {
        Ok(ans) => Ok(CommandResult::ok(ans)),
        Err(e) => Ok(CommandResult::err(e)),
    }
}

pub async fn run_screen_capture_and_solve(
    app: &AppHandle,
    state: AppState,
) -> Result<String, String> {
    log::info!("📸 [Screen Solve] Starting screen capture and solve workflow...");

    // Capture Screen (Main dashboard and overlay are NOT hidden because overlay is excluded and main dashboard can remain untouched)
    log::info!("📸 [Screen Solve] Executing screen capture...");
    let jpeg_bytes = match tokio::task::spawn_blocking(|| {
        use screenshots::Screen;
        let screens = Screen::all().map_err(|e| anyhow::anyhow!("Failed to list screens: {e}"))?;
        let screen = screens.first().ok_or_else(|| anyhow::anyhow!("No screens found"))?;
        let image = screen.capture().map_err(|e| anyhow::anyhow!("Failed to capture screen: {e}"))?;
        let mut buffer = std::io::Cursor::new(Vec::new());
        image.write_to(&mut buffer, screenshots::image::ImageOutputFormat::Jpeg(80))
            .map_err(|e| anyhow::anyhow!("Failed to encode to JPEG: {e}"))?;
        let jpeg_bytes = buffer.into_inner();
        Ok::<Vec<u8>, anyhow::Error>(jpeg_bytes)
    }).await {
        Ok(Ok(bytes)) => {
            log::info!("📸 [Screen Solve] Screen captured successfully ({} bytes JPEG)", bytes.len());
            bytes
        }
        Ok(Err(e)) => {
            log::error!("📸 [Screen Solve] Capture error: {e}");
            return Err(format!("Screen capture error: {e}"));
        }
        Err(e) => {
            log::error!("📸 [Screen Solve] Task join error: {e}");
            return Err(format!("Task execution failed: {e}"));
        }
    };

    // Base64 Encode JPEG
    use base64::{prelude::BASE64_STANDARD, Engine};
    let base64_image = BASE64_STANDARD.encode(&jpeg_bytes);

    // 5. Get API key
    let api_key = match keyring_store::load_api_key() {
        Ok(Some(k)) => k,
        _ => {
            log::error!("📸 [Screen Solve] Groq API key is not configured");
            return Err("Groq API key not configured — add it in Settings".to_string());
        }
    };

    let groq = match groq_client::GroqClient::new(api_key) {
        Ok(c) => c,
        Err(e) => {
            log::error!("📸 [Screen Solve] Failed to initialize Groq client: {e}");
            return Err(format!("Failed to create Groq client: {e}"));
        }
    };

    // 6. Emit status updates
    let _ = app.emit("status-changed", serde_json::json!({
        "status": "thinking"
    }));

    // 7. Call Vision API
    log::info!("📸 [Screen Solve] Sending image to Groq Vision API...");
    match groq.solve_vision_screenshot(&base64_image).await {
        Ok(answer) => {
            log::info!("📸 [Screen Solve] Groq Vision solved successfully!");
            // Update app inner state
            {
                if let Ok(mut lock) = state.0.lock() {
                    lock.current_transcript = "[Visual Solve]".to_string();
                    lock.current_answer = answer.clone();
                }
            }

            // Emit to frontend (so history gets updated too)
            let _ = app.emit("answer-ready", serde_json::json!({
                "transcript": "[Visual Solve]",
                "answer": &answer
            }));

            let _ = app.emit("status-changed", serde_json::json!({
                "status": "listening"
            }));

            Ok(answer)
        }
        Err(e) => {
            let msg = e.to_string();
            log::error!("📸 [Screen Solve] Groq Vision API error: {msg}");
            let _ = app.emit("status-changed", serde_json::json!({
                "status": "listening"
            }));
            let _ = app.emit("pipeline-error", &msg);
            Err(msg)
        }
    }
}

#[tauri::command]
pub async fn solve_screen_region(
    app: AppHandle,
    state: State<'_, AppState>,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
) -> Result<CommandResult<String>, String> {
    match run_screen_region_capture_and_solve(&app, state.inner().clone(), x, y, w, h).await {
        Ok(ans) => Ok(CommandResult::ok(ans)),
        Err(e) => Ok(CommandResult::err(e)),
    }
}

pub async fn run_screen_region_capture_and_solve(
    app: &AppHandle,
    state: AppState,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
) -> Result<String, String> {
    log::info!("📸 [Region Solve] Starting capture at logical x={}, y={}, w={}, h={}", x, y, w, h);

    if w <= 5 || h <= 5 {
        return Err("Selection region too small".to_string());
    }

    // 1. Compute physical coordinates (Main dashboard and overlay are NOT hidden because overlay is excluded and main dashboard can remain untouched)
    let overlay_win = app.get_webview_window("overlay");
    let (monitor_x, monitor_y, scale_factor) = if let Some(ref win) = overlay_win {
        let monitor = win.current_monitor().ok().flatten();
        let (mx, my) = if let Some(ref m) = monitor {
            let pos = m.position();
            (pos.x, pos.y)
        } else {
            (0, 0)
        };
        let sf = win.scale_factor().unwrap_or(1.0);
        (mx, my, sf)
    } else {
        (0, 0, 1.0)
    };

    let phys_x = monitor_x + (x as f64 * scale_factor) as i32;
    let phys_y = monitor_y + (y as f64 * scale_factor) as i32;
    let phys_w = (w as f64 * scale_factor) as u32;
    let phys_h = (h as f64 * scale_factor) as u32;

    log::info!(
        "📸 [Region Solve] Physical bounds: origin=({}, {}), pos=({}, {}), size=({}x{}), scale={}",
        monitor_x, monitor_y, phys_x, phys_y, phys_w, phys_h, scale_factor
    );

    // 2. Capture region
    let jpeg_bytes = match tokio::task::spawn_blocking(move || {
        use screenshots::Screen;
        let screen = Screen::from_point(phys_x, phys_y)
            .or_else(|_| {
                Screen::all()
                    .map_err(|e| anyhow::anyhow!("Failed to list screens: {e}"))?
                    .into_iter()
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("No screens found"))
            })?;

        // Translate to screen-local coordinate space
        let local_x = phys_x - screen.display_info.x;
        let local_y = phys_y - screen.display_info.y;

        log::info!(
            "📸 [Region Solve] screen.display_info origin=({}, {}), local bounds=({}, {})",
            screen.display_info.x, screen.display_info.y, local_x, local_y
        );

        let image = screen.capture_area(local_x, local_y, phys_w, phys_h)
            .map_err(|e| anyhow::anyhow!("Failed to capture screen area: {e}"))?;

        let mut buffer = std::io::Cursor::new(Vec::new());
        image.write_to(&mut buffer, screenshots::image::ImageOutputFormat::Jpeg(80))
            .map_err(|e| anyhow::anyhow!("Failed to encode to JPEG: {e}"))?;
        
        Ok::<Vec<u8>, anyhow::Error>(buffer.into_inner())
    }).await {
        Ok(Ok(bytes)) => bytes,
        Ok(Err(e)) => {
            log::error!("📸 [Region Solve] Capture error: {e}");
            return Err(format!("Screen capture error: {e}"));
        }
        Err(e) => {
            log::error!("📸 [Region Solve] Task join error: {e}");
            return Err(format!("Task execution failed: {e}"));
        }
    };

    // 3. Base64 Encode JPEG
    use base64::{prelude::BASE64_STANDARD, Engine};
    let base64_image = BASE64_STANDARD.encode(&jpeg_bytes);

    // 6. Get API key
    let api_key = match keyring_store::load_api_key() {
        Ok(Some(k)) => k,
        _ => return Err("Groq API key not configured — add it in Settings".to_string()),
    };

    let groq = groq_client::GroqClient::new(api_key)
        .map_err(|e| format!("Failed to create Groq client: {e}"))?;

    // 7. Emit status updates
    let _ = app.emit("status-changed", serde_json::json!({
        "status": "thinking"
    }));

    // 8. Call Vision API
    log::info!("📸 [Region Solve] Sending region image to Groq Vision API...");
    match groq.solve_vision_screenshot(&base64_image).await {
        Ok(answer) => {
            log::info!("📸 [Region Solve] Groq Vision solved region successfully!");
            // Update app inner state
            {
                if let Ok(mut lock) = state.0.lock() {
                    lock.current_transcript = "[Visual Solve]".to_string();
                    lock.current_answer = answer.clone();
                }
            }

            // Emit to frontend
            let _ = app.emit("answer-ready", serde_json::json!({
                "transcript": "[Visual Solve]",
                "answer": &answer
            }));

            let _ = app.emit("status-changed", serde_json::json!({
                "status": "listening"
            }));

            Ok(answer)
        }
        Err(e) => {
            let msg = e.to_string();
            log::error!("📸 [Region Solve] Groq Vision API error: {msg}");
            let _ = app.emit("status-changed", serde_json::json!({
                "status": "listening"
            }));
            let _ = app.emit("pipeline-error", &msg);
            Err(msg)
        }
    }
}
