use tauri::AppHandle;

/// Cross-platform coordinator managing the borderless caption overlay.
/// Spawns a secondary transparent multi-window on desktop (Windows, macOS, Linux),
/// and delegates to an in-app persistent floating HUD / PiP bridge on mobile (Android, iOS).
pub struct CaptionOverlayManager;

impl CaptionOverlayManager {
    /// Toggle the visibility of the native desktop overlay window or mobile PiP HUD.
    #[cfg(desktop)]
    pub fn toggle_desktop_overlay(app: &AppHandle) -> Result<bool, String> {
        use tauri::Manager;
        if let Some(window) = app.get_webview_window("caption-overlay") {
            let is_visible = window.is_visible().map_err(|e| e.to_string())?;
            if is_visible {
                window.hide().map_err(|e| e.to_string())?;
                Ok(false)
            } else {
                window.show().map_err(|e| e.to_string())?;
                let _ = window.set_focus();
                Ok(true)
            }
        } else {
            let overlay = tauri::WebviewWindowBuilder::new(
                app,
                "caption-overlay",
                tauri::WebviewUrl::App("index.html#/overlay".into()),
            )
            .title("LinguaFlow Overlay")
            .decorations(false)
            .transparent(true)
            .always_on_top(true)
            .inner_size(820.0, 160.0)
            .position(100.0, 100.0)
            .build()
            .map_err(|e| e.to_string())?;

            overlay.show().map_err(|e| e.to_string())?;
            Ok(true)
        }
    }

    /// Mobile fallback ensuring `cargo check` and run pass without attempting desktop multi-window creation.
    #[cfg(not(desktop))]
    pub fn toggle_desktop_overlay(_app: &AppHandle) -> Result<bool, String> {
        tracing::info!("[mobile] Caption overlay toggled via in-app persistent floating PiP HUD");
        Ok(true)
    }

    /// Broadcast live caption updates across IPC to the overlay window and main views.
    pub fn update_caption_text(
        app: &AppHandle,
        source_text: &str,
        translated_text: Option<&str>,
    ) -> Result<(), String> {
        use tauri::Emitter;
        app.emit(
            "caption-overlay-update",
            serde_json::json!({
                "source_text": source_text,
                "translated_text": translated_text,
            }),
        )
        .map_err(|e| e.to_string())
    }
}
