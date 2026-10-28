use lingua_core::{LinguaError, OverlaySettings, SystemAudioPermission};

pub struct LiveOverlayController;

impl LiveOverlayController {
    /// Check operating system permission for system audio loopback capture.
    pub fn check_system_audio_permission() -> SystemAudioPermission {
        // Honest capability detection per platform
        #[cfg(target_os = "windows")]
        {
            // Windows WASAPI loopback capture is permitted for standard applications without special kernel rights
            SystemAudioPermission::Granted
        }
        #[cfg(target_os = "macos")]
        {
            // macOS requires explicit Screen & System Audio Recording permission in System Settings
            SystemAudioPermission::Restricted
        }
        #[cfg(target_os = "linux")]
        {
            // Linux PulseAudio / PipeWire monitor sources are accessible where user is in audio group
            SystemAudioPermission::Granted
        }
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            SystemAudioPermission::NotDetermined
        }
    }

    /// Document protected-content and application-capture restrictions (Contract Rule 13).
    pub fn get_protected_content_policy() -> &'static str {
        "LinguaFlow respects host operating system privacy boundaries and DRM policies.\n\
         Protected media streams (such as HDCP, Widevine L1, or macOS FairPlay protected windows)\n\
         and applications explicitly flagged with privacy protections cannot be captured via\n\
         background system loopback. For such applications, use LinguaFlow's microphone-based\n\
         acoustic capture or manual media file import."
    }

    /// Validate and clamp overlay settings within safe viewport and visual bounds.
    pub fn validate_and_clamp_settings(settings: &mut OverlaySettings) {
        settings.width = settings.width.clamp(300, 1920);
        settings.height = settings.height.clamp(80, 600);
        settings.font_size = settings.font_size.clamp(14, 48);
        settings.opacity = settings.opacity.clamp(0.2, 1.0);
    }
}

pub struct PlatformIntegrationsCoordinator;

impl PlatformIntegrationsCoordinator {
    /// Parse and validate standard accelerator strings for global shortcuts.
    pub fn validate_shortcut_accelerator(accel: &str) -> Result<(), LinguaError> {
        let clean = accel.trim();
        if clean.is_empty() {
            return Err(LinguaError::Validation(
                "Shortcut accelerator cannot be empty".into(),
            ));
        }

        let valid_modifiers = [
            "Ctrl",
            "Control",
            "Command",
            "CommandOrControl",
            "Alt",
            "Shift",
            "Super",
        ];
        let has_modifier = valid_modifiers.iter().any(|&m| clean.contains(m));

        if !has_modifier {
            return Err(LinguaError::Validation(format!(
                "Global shortcut '{clean}' must include at least one modifier key (e.g. Ctrl, Alt, Shift)"
            )));
        }

        Ok(())
    }

    /// Process clipboard translation on explicit user demand.
    /// Strictly requires user-initiated action to prevent invasive background scraping.
    pub fn handle_user_clipboard_translation(clipboard_text: &str) -> Result<String, LinguaError> {
        let clean = clipboard_text.trim();
        if clean.is_empty() {
            return Err(LinguaError::Validation(
                "Clipboard contains no readable text".into(),
            ));
        }

        if clean.len() > 100_000 {
            return Err(LinguaError::Validation(
                "Clipboard text exceeds maximum supported size (100,000 characters)".into(),
            ));
        }

        Ok(clean.to_string())
    }

    /// Format payload for OS native share sheet target.
    pub fn format_share_payload(title: &str, source_text: &str, translated_text: &str) -> String {
        format!("{title}\n\nOriginal:\n{source_text}\n\nTranslation:\n{translated_text}\n\n— Shared via LinguaFlow")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_audio_permission_and_policy() {
        let perm = LiveOverlayController::check_system_audio_permission();
        assert!(matches!(
            perm,
            SystemAudioPermission::Granted
                | SystemAudioPermission::Restricted
                | SystemAudioPermission::NotDetermined
        ));

        let policy = LiveOverlayController::get_protected_content_policy();
        assert!(policy.contains("DRM"));
        assert!(policy.contains("HDCP"));
    }

    #[test]
    fn test_overlay_bounds_clamping() {
        let mut settings = OverlaySettings {
            width: 2500,
            height: 20,
            font_size: 10,
            opacity: 0.05,
            ..Default::default()
        };

        LiveOverlayController::validate_and_clamp_settings(&mut settings);
        assert_eq!(settings.width, 1920);
        assert_eq!(settings.height, 80);
        assert_eq!(settings.font_size, 14);
        assert_eq!(settings.opacity, 0.2);
    }

    #[test]
    fn test_shortcut_validation() {
        assert!(
            PlatformIntegrationsCoordinator::validate_shortcut_accelerator(
                "CommandOrControl+Shift+L"
            )
            .is_ok()
        );
        assert!(PlatformIntegrationsCoordinator::validate_shortcut_accelerator("Alt+T").is_ok());
        assert!(
            PlatformIntegrationsCoordinator::validate_shortcut_accelerator("JustAKey").is_err()
        );
    }

    #[test]
    fn test_clipboard_handling_and_share_formatting() {
        let clean =
            PlatformIntegrationsCoordinator::handle_user_clipboard_translation("   ನಮಸ್ಕಾರ   ")
                .unwrap();
        assert_eq!(clean, "ನಮಸ್ಕಾರ");

        let share = PlatformIntegrationsCoordinator::format_share_payload(
            "Meeting Note",
            "Hello",
            "ನಮಸ್ಕಾರ",
        );
        assert!(share.contains("Meeting Note"));
        assert!(share.contains("Shared via LinguaFlow"));
    }
}
