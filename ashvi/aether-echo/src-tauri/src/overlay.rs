// AetherEcho — Overlay Window Manager
// Manages the transparent always-on-top overlay window.
// On Windows, calls SetWindowDisplayAffinity with WDA_EXCLUDEFROMCAPTURE
// so the overlay is invisible in screen recordings and screen shares.

use anyhow::Result;
use tauri::{AppHandle, Manager};

/// Apply all overlay-specific window attributes after the window is created.
/// Call this once during app setup.
pub fn setup_overlay_window(app: &AppHandle) -> Result<()> {
    let Some(overlay) = app.get_webview_window("overlay") else {
        log::warn!("Overlay window not found — skipping setup");
        return Ok(());
    };

    // Prevent the overlay from appearing in screen captures
    #[cfg(target_os = "windows")]
    apply_capture_exclusion(&overlay)?;

    // Make it always on top
    overlay.set_always_on_top(true)?;

    // Start with overlay hidden
    overlay.hide()?;

    log::info!("✅ Overlay window configured");
    Ok(())
}

/// Apply WDA_EXCLUDEFROMCAPTURE on Windows so the overlay is invisible
/// in OBS, Zoom, Teams, Google Meet screen shares, etc.
#[cfg(target_os = "windows")]
fn apply_capture_exclusion(window: &tauri::WebviewWindow) -> Result<()> {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{
        SetWindowDisplayAffinity, WDA_EXCLUDEFROMCAPTURE,
    };

    // Get the HWND via the window's inner size (forces initialization),
    // then extract it from the raw handle
    let hwnd_raw = window.hwnd()
        .map_err(|e| anyhow::anyhow!("Failed to get HWND: {e}"))?;
    let hwnd = HWND(hwnd_raw.0 as *mut std::ffi::c_void);

    unsafe {
        SetWindowDisplayAffinity(hwnd, WDA_EXCLUDEFROMCAPTURE)
            .map_err(|e| anyhow::anyhow!("SetWindowDisplayAffinity failed: {e}"))?;
    }

    log::info!("✅ WDA_EXCLUDEFROMCAPTURE applied — overlay is screen-capture invisible");
    Ok(())
}

/// Show or hide the overlay window.
pub fn set_overlay_visible(app: &AppHandle, visible: bool) -> Result<()> {
    let Some(overlay) = app.get_webview_window("overlay") else {
        return Err(anyhow::anyhow!("Overlay window not found"));
    };

    if visible {
        overlay.show()?;
        overlay.set_always_on_top(true)?;
    } else {
        overlay.hide()?;
    }

    log::info!("Overlay visibility set to: {visible}");
    Ok(())
}

/// Enable or disable click-through mode on the overlay.
/// In click-through mode (focusable=false), mouse clicks pass through the overlay.
pub fn set_overlay_click_through(app: &AppHandle, click_through: bool) -> Result<()> {
    let Some(overlay) = app.get_webview_window("overlay") else {
        return Err(anyhow::anyhow!("Overlay window not found"));
    };

    // When click_through = true, set focusable = false so clicks pass through
    overlay.set_ignore_cursor_events(click_through)?;

    log::info!("Overlay click-through: {click_through}");
    Ok(())
}

/// Set the overlay window opacity (0.0 – 1.0).
pub fn set_overlay_opacity(app: &AppHandle, opacity: f64) -> Result<()> {
    let Some(overlay) = app.get_webview_window("overlay") else {
        return Err(anyhow::anyhow!("Overlay window not found"));
    };

    #[cfg(target_os = "windows")]
    {
        use windows::Win32::Foundation::HWND;
        use windows::Win32::UI::WindowsAndMessaging::{
            GetWindowLongPtrW, SetWindowLongPtrW, SetLayeredWindowAttributes,
            GWL_EXSTYLE, WS_EX_LAYERED, LWA_ALPHA,
        };

        let hwnd_raw = overlay.hwnd()
            .map_err(|e| anyhow::anyhow!("Failed to get HWND: {e}"))?;
        let hwnd = HWND(hwnd_raw.0 as *mut std::ffi::c_void);

        unsafe {
            let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
            if (ex_style & (WS_EX_LAYERED.0 as isize)) == 0 {
                let _ = SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex_style | (WS_EX_LAYERED.0 as isize));
            }
            let alpha = (opacity.clamp(0.05, 1.0) * 255.0) as u8;
            SetLayeredWindowAttributes(hwnd, windows::Win32::Foundation::COLORREF(0), alpha, LWA_ALPHA)
                .map_err(|e| anyhow::anyhow!("SetLayeredWindowAttributes failed: {e}"))?;
        }
    }

    log::info!("Overlay opacity set to: {opacity}");
    Ok(())
}
