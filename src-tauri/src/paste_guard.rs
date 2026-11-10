//! Makes sure every dictation lands in the app it was meant for.
//!
//! Remembers which app was in front when recording started. Just before the
//! paste, if another app has come to the front, it brings the original one
//! back (waiting up to [`FOCUS_WAIT_MS`]). If that fails, it does not paste
//! into the wrong place: the words stay on the clipboard and the person is
//! told to press Cmd+V. Never pastes twice.

use std::sync::atomic::{AtomicI32, Ordering};
use std::time::{Duration, Instant};

/// Longest wait for the original app to come back to the front.
pub const FOCUS_WAIT_MS: u64 = 1500;
const POLL_MS: u64 = 20;
const NO_TARGET: i32 = 0;

static TARGET_PID: AtomicI32 = AtomicI32::new(NO_TARGET);

/// Called when recording starts: remember the app being dictated into.
pub fn remember_target() {
    let pid = crate::active_window::frontmost_pid().unwrap_or(NO_TARGET);
    // Never target murmur itself (its settings window can be in front).
    let own = std::process::id() as i32;
    TARGET_PID.store(if pid == own { NO_TARGET } else { pid }, Ordering::SeqCst);
}

/// What to do right before pasting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ready {
    /// The right app is in front (or there is nothing to check): paste.
    Paste,
    /// The right app could not be brought back: keep the words on the clipboard.
    CopyOnly,
}

/// Pure decision, so it can be tested: target vs what is in front now.
pub fn decide(target: Option<i32>, front: Option<i32>) -> Option<Ready> {
    match (target, front) {
        (None, _) | (_, None) => Some(Ready::Paste),
        (Some(t), Some(f)) if t == f => Some(Ready::Paste),
        _ => None, // needs the original app back first
    }
}

/// Checks (and if needed restores) the app in front before a paste.
pub fn ensure_target_front() -> Ready {
    let target = match TARGET_PID.load(Ordering::SeqCst) {
        NO_TARGET => None,
        pid => Some(pid),
    };
    if let Some(ready) = decide(target, crate::active_window::frontmost_pid()) {
        return ready;
    }
    let pid = target.unwrap_or(NO_TARGET);
    log::info!("paste guard: target app {pid} not in front, bringing it back");
    if !crate::active_window::activate_pid(pid) {
        return Ready::CopyOnly;
    }
    let deadline = Instant::now() + Duration::from_millis(FOCUS_WAIT_MS);
    while Instant::now() < deadline {
        if crate::active_window::frontmost_pid() == Some(pid) {
            // Give the window a moment to take keyboard focus.
            std::thread::sleep(Duration::from_millis(60));
            return Ready::Paste;
        }
        std::thread::sleep(Duration::from_millis(POLL_MS));
    }
    log::warn!("paste guard: target app did not come back; keeping text on the clipboard");
    Ready::CopyOnly
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pastes_when_target_is_in_front_or_unknown() {
        assert_eq!(decide(Some(42), Some(42)), Some(Ready::Paste));
        assert_eq!(decide(None, Some(7)), Some(Ready::Paste));
        assert_eq!(decide(Some(42), None), Some(Ready::Paste));
    }

    #[test]
    fn needs_refocus_when_another_app_took_over() {
        assert_eq!(decide(Some(42), Some(7)), None);
    }
}
