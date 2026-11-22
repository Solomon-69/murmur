//! Repairs stale macOS privacy entries after an update.
//!
//! macOS remembers Accessibility and Microphone grants by the app's code
//! signature. When the signature changes (every ad-hoc build did, and the
//! move to the stable Murmur certificate does once), the old entry stays in
//! System Settings with its switch ON but no longer matches the app, so the
//! permission silently doesn't work. Users had to remove the entry with the
//! minus button and add it again.
//!
//! At startup, if a permission that was granted before now reads as missing,
//! the app's own entry is reset with `tccutil reset <service> com.pais.handy`
//! (no admin password needed for an app's own bundle id) and access is asked
//! for again, so the next Allow creates a fresh, matching entry. Each reset
//! runs at most once per app version, so it can never loop.

use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const STATE_FILE: &str = "permission_repair.json";

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct RepairState {
    pub accessibility_was_granted: bool,
    pub microphone_was_granted: bool,
    /// App version in which each reset last ran.
    pub accessibility_reset_version: String,
    pub microphone_reset_version: String,
}

/// Reset only when the permission worked before, doesn't now, and no reset
/// has run yet for this app version.
pub fn should_reset(was_granted: bool, granted_now: bool, last_reset: &str, version: &str) -> bool {
    was_granted && !granted_now && last_reset != version
}

fn state_path(dir: &Path) -> PathBuf {
    dir.join(STATE_FILE)
}

pub fn load_state(dir: &Path) -> RepairState {
    std::fs::read_to_string(state_path(dir))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_state(dir: &Path, state: &RepairState) {
    match serde_json::to_string_pretty(state) {
        Ok(json) => {
            if let Err(e) = std::fs::write(state_path(dir), json) {
                warn!("Couldn't save permission repair state: {}", e);
            }
        }
        Err(e) => warn!("Couldn't encode permission repair state: {}", e),
    }
}

/// What to do this launch, given the current readings. Updates `state`
/// (grant flags and reset versions) and returns which resets to run.
pub fn plan(
    state: &mut RepairState,
    version: &str,
    onboarding_completed: bool,
    accessibility_now: bool,
    microphone_now: bool,
    microphone_denied: bool,
) -> (bool, bool) {
    if accessibility_now {
        state.accessibility_was_granted = true;
    }
    if microphone_now {
        state.microphone_was_granted = true;
    }
    let reset_ax = should_reset(
        state.accessibility_was_granted || onboarding_completed,
        accessibility_now,
        &state.accessibility_reset_version,
        version,
    );
    // A microphone entry that doesn't match shows as denied; "not asked yet"
    // already gets the normal prompt, so only a denial is reset.
    let reset_mic = microphone_denied
        && should_reset(
            state.microphone_was_granted || onboarding_completed,
            microphone_now,
            &state.microphone_reset_version,
            version,
        );
    if reset_ax {
        state.accessibility_reset_version = version.to_string();
    }
    if reset_mic {
        state.microphone_reset_version = version.to_string();
    }
    (reset_ax, reset_mic)
}

#[cfg(target_os = "macos")]
mod mac {
    use objc2::{class, msg_send};
    use objc2_foundation::NSString;

    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXIsProcessTrusted() -> bool;
    }

    pub fn accessibility_granted() -> bool {
        unsafe { AXIsProcessTrusted() }
    }

    /// AVAuthorizationStatus: 0 not determined, 1 restricted, 2 denied, 3 authorized.
    pub fn microphone_status() -> isize {
        unsafe {
            let media = NSString::from_str("soun");
            msg_send![class!(AVCaptureDevice), authorizationStatusForMediaType: &*media]
        }
    }

    pub fn tcc_reset(service: &str, bundle_id: &str) -> bool {
        match std::process::Command::new("/usr/bin/tccutil")
            .args(["reset", service, bundle_id])
            .output()
        {
            Ok(out) if out.status.success() => true,
            Ok(out) => {
                log::warn!(
                    "tccutil reset {} failed: {}",
                    service,
                    String::from_utf8_lossy(&out.stderr).trim()
                );
                false
            }
            Err(e) => {
                log::warn!("Couldn't run tccutil: {}", e);
                false
            }
        }
    }
}

/// Run at startup (macOS only). Never blocks the caller for long: the
/// prompts are asynchronous.
#[cfg(target_os = "macos")]
pub fn repair_stale_permissions(app: &tauri::AppHandle) {
    use tauri::Manager;
    let Ok(dir) = crate::portable::app_data_dir(app) else {
        return;
    };
    let version = app.package_info().version.to_string();
    let bundle_id = app.config().identifier.clone();
    let onboarding = crate::settings::get_settings(app).onboarding_completed;
    let ax_now = mac::accessibility_granted();
    let mic_status = mac::microphone_status();
    let mut state = load_state(&dir);
    let before = state.clone();
    let (reset_ax, reset_mic) = plan(
        &mut state,
        &version,
        onboarding,
        ax_now,
        mic_status == 3,
        mic_status == 2,
    );
    if state != before {
        save_state(&dir, &state);
    }
    if reset_ax {
        info!("Accessibility entry looks stale after an update; resetting it");
        if mac::tcc_reset("Accessibility", &bundle_id) {
            tauri::async_runtime::spawn(
                tauri_plugin_macos_permissions::request_accessibility_permission(),
            );
        }
    }
    if reset_mic {
        info!("Microphone entry looks stale after an update; resetting it");
        if mac::tcc_reset("Microphone", &bundle_id) {
            tauri::async_runtime::spawn(async {
                let _ = tauri_plugin_macos_permissions::request_microphone_permission().await;
            });
        }
    }
}

#[cfg(not(target_os = "macos"))]
pub fn repair_stale_permissions(_app: &tauri::AppHandle) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resets_once_per_version() {
        let mut s = RepairState {
            accessibility_was_granted: true,
            microphone_was_granted: true,
            ..Default::default()
        };
        assert_eq!(plan(&mut s, "0.12.6", true, false, false, true), (true, true));
        // Same version, still broken: no second reset, so no loop.
        assert_eq!(plan(&mut s, "0.12.6", true, false, false, true), (false, false));
        // Next version: one more try.
        assert_eq!(plan(&mut s, "0.12.7", true, false, false, true), (true, true));
    }

    #[test]
    fn no_reset_when_granted_or_never_granted() {
        let mut s = RepairState::default();
        assert_eq!(plan(&mut s, "1", true, true, true, false), (false, false));
        assert!(s.accessibility_was_granted && s.microphone_was_granted);
        let mut fresh = RepairState::default();
        // New user, onboarding not done: nothing to repair.
        assert_eq!(plan(&mut fresh, "1", false, false, false, false), (false, false));
        // Microphone not asked yet (not denied): normal prompt, no reset.
        let mut s2 = RepairState {
            microphone_was_granted: true,
            ..Default::default()
        };
        assert_eq!(plan(&mut s2, "1", true, true, false, false), (false, false));
    }

    #[test]
    fn state_round_trips() {
        let dir = tempfile::TempDir::new().unwrap();
        assert_eq!(load_state(dir.path()), RepairState::default());
        let s = RepairState {
            accessibility_was_granted: true,
            accessibility_reset_version: "0.12.6".into(),
            ..Default::default()
        };
        save_state(dir.path(), &s);
        assert_eq!(load_state(dir.path()), s);
    }
}
