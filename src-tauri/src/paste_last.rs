//! Paste a past dictation again: the newest one (shortcut or command) or any
//! history entry (the "Paste again" button in History).

use crate::managers::history::{HistoryEntry, HistoryManager};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Manager};

/// Wait after the shortcut fires so its modifier keys are up before Cmd+V
/// (Option+Shift held during Cmd+V would turn it into "paste and match style").
const KEY_RELEASE_WAIT_MS: u64 = 200;
/// Wait after murmur hides itself so the previous app is in front again.
const FOCUS_RETURN_WAIT_MS: u64 = 350;

/// The text a history entry pastes: the cleaned version when there is one.
/// `None` for an entry with nothing in it.
pub fn paste_text(entry: &HistoryEntry) -> Option<String> {
    let text = entry
        .post_processed_text
        .as_deref()
        .filter(|t| !t.trim().is_empty())
        .unwrap_or(&entry.transcription_text);
    (!text.trim().is_empty()).then(|| text.to_string())
}

/// Text of the newest finished dictation.
pub fn latest_text(app: &AppHandle) -> Result<String, String> {
    let history = app.state::<Arc<HistoryManager>>();
    let entry = history
        .get_latest_completed_entry()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "No dictation to paste yet".to_string())?;
    paste_text(&entry).ok_or_else(|| "The last dictation is empty".to_string())
}

/// Pastes `text` on a background thread after `wait_ms` (paste sleeps
/// between steps and must not run on the main thread).
pub fn paste_later(app: &AppHandle, text: String, wait_ms: u64) {
    let app = app.clone();
    let spawned = std::thread::Builder::new()
        .name("paste-again".into())
        .spawn(move || {
            std::thread::sleep(Duration::from_millis(wait_ms));
            if let Err(e) = crate::utils::paste(text, app) {
                log::error!("Paste again failed: {e}");
            }
        });
    if let Err(e) = spawned {
        log::error!("Failed to start the paste-again thread: {e:?}");
    }
}

/// Shortcut path: paste the newest dictation into the app in front.
pub fn paste_latest_from_shortcut(app: &AppHandle) {
    match latest_text(app) {
        Ok(text) => paste_later(app, text, KEY_RELEASE_WAIT_MS),
        Err(e) => log::warn!("Paste last dictation: {e}"),
    }
}

/// Settings-window path: hide murmur so the previous app gets focus back,
/// then paste there.
pub fn paste_after_hiding(app: &AppHandle, text: String) {
    hide_murmur(app);
    paste_later(app, text, FOCUS_RETURN_WAIT_MS);
}

fn hide_murmur(app: &AppHandle) {
    #[cfg(target_os = "macos")]
    {
        // Hiding the whole app makes macOS activate the app used before it.
        if let Err(e) = app.hide() {
            log::warn!("Could not hide murmur before pasting: {e}");
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        if let Some(window) = app.get_webview_window("main") {
            if let Err(e) = window.minimize() {
                log::warn!("Could not minimise murmur before pasting: {e}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(raw: &str, processed: Option<&str>) -> HistoryEntry {
        HistoryEntry {
            id: 1,
            file_name: "a.wav".into(),
            timestamp: 0,
            saved: false,
            title: String::new(),
            transcription_text: raw.into(),
            post_processed_text: processed.map(str::to_string),
            post_process_prompt: None,
            post_process_requested: false,
            source_name: None,
            language: None,
            language_source: None,
        }
    }

    #[test]
    fn prefers_cleaned_text() {
        assert_eq!(paste_text(&entry("raw", Some("clean"))).as_deref(), Some("clean"));
    }

    #[test]
    fn blank_cleaned_text_falls_back_to_raw() {
        assert_eq!(paste_text(&entry("raw", Some("  "))).as_deref(), Some("raw"));
    }

    #[test]
    fn empty_entry_pastes_nothing() {
        assert_eq!(paste_text(&entry(" ", None)), None);
    }
}
