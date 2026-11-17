//! Finds the frame of the window (and text box) the person is typing into, so
//! the recording bar can centre on it. macOS only; other platforms return
//! `None` and the bar falls back to the screen centre.
//!
//! Uses the Accessibility API, which murmur already needs for pasting. Every
//! call has a short messaging timeout so a hung app can never stall the bar.

use crate::overlay_placement::{pick_target, Rect};

/// The frame to centre the bar on, in global top-left-origin points.
pub fn focused_target_frame() -> Option<Rect> {
    #[cfg(target_os = "macos")]
    {
        let (window, focused) = mac::frontmost_frames()?;
        pick_target(window, focused)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = pick_target;
        None
    }
}

/// Bundle id of the app in front (e.g. "com.apple.mail"). macOS only.
pub fn frontmost_bundle_id() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        mac::frontmost_bundle_id()
    }
    #[cfg(not(target_os = "macos"))]
    {
        None
    }
}

/// The text selected in the focused field of the app in front, read through
/// Accessibility: no keystroke, no clipboard, no focus change. None when
/// nothing is selected, the app does not say, or access is not granted.
pub fn frontmost_selected_text() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        mac::frontmost_selected_text()
    }
    #[cfg(not(target_os = "macos"))]
    {
        None
    }
}

/// Process id of the app in front. macOS only.
pub fn frontmost_pid() -> Option<i32> {
    #[cfg(target_os = "macos")]
    {
        mac::frontmost_pid()
    }
    #[cfg(not(target_os = "macos"))]
    {
        None
    }
}

/// Brings the app with this process id back to the front. macOS only.
pub fn activate_pid(pid: i32) -> bool {
    #[cfg(target_os = "macos")]
    {
        mac::activate_pid(pid)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = pid;
        false
    }
}

#[cfg(target_os = "macos")]
mod mac {
    use super::Rect;
    use objc2::runtime::AnyObject;
    use objc2::{class, msg_send};
    use objc2_foundation::NSString;
    use std::ffi::c_void;

    type CFTypeRef = *const c_void;
    type AXUIElementRef = *const c_void;
    type AXError = i32;

    const AX_SUCCESS: AXError = 0;
    const AX_VALUE_CGPOINT: u32 = 1;
    const AX_VALUE_CGSIZE: u32 = 2;
    /// Seconds an Accessibility request may take before we give up.
    const AX_TIMEOUT_SECS: f32 = 0.15;

    #[repr(C)]
    #[derive(Default, Clone, Copy)]
    struct CGPoint {
        x: f64,
        y: f64,
    }

    #[repr(C)]
    #[derive(Default, Clone, Copy)]
    struct CGSize {
        width: f64,
        height: f64,
    }

    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXIsProcessTrusted() -> bool;
        fn AXUIElementCreateApplication(pid: i32) -> AXUIElementRef;
        fn AXUIElementSetMessagingTimeout(element: AXUIElementRef, timeout: f32) -> AXError;
        fn AXUIElementCopyAttributeValue(
            element: AXUIElementRef,
            attribute: CFTypeRef,
            value: *mut CFTypeRef,
        ) -> AXError;
        fn AXValueGetValue(value: CFTypeRef, the_type: u32, out: *mut c_void) -> bool;
        fn AXUIElementSetAttributeValue(
            element: AXUIElementRef,
            attribute: CFTypeRef,
            value: CFTypeRef,
        ) -> AXError;
    }

    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFRelease(cf: CFTypeRef);
        static kCFBooleanTrue: CFTypeRef;
    }

    /// Last app asked to switch its accessibility tree on (Electron apps such
    /// as Claude, Slack or VS Code only expose the selection after this).
    static MANUAL_AX_PID: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);

    fn enable_manual_accessibility(app: CFTypeRef, pid: i32) {
        if MANUAL_AX_PID.swap(pid, std::sync::atomic::Ordering::SeqCst) == pid {
            return;
        }
        let attr = NSString::from_str("AXManualAccessibility");
        let attr_ref = objc2::rc::Retained::as_ptr(&attr) as CFTypeRef;
        let _ = unsafe { AXUIElementSetAttributeValue(app, attr_ref, kCFBooleanTrue) };
    }

    /// Owns one CoreFoundation reference and releases it on drop.
    struct Cf(CFTypeRef);

    impl Drop for Cf {
        fn drop(&mut self) {
            if !self.0.is_null() {
                unsafe { CFRelease(self.0) };
            }
        }
    }

    fn copy_attr(element: CFTypeRef, name: &str) -> Option<Cf> {
        let attr = NSString::from_str(name);
        let attr_ref = objc2::rc::Retained::as_ptr(&attr) as CFTypeRef;
        let mut value: CFTypeRef = std::ptr::null();
        let err = unsafe { AXUIElementCopyAttributeValue(element, attr_ref, &mut value) };
        if err == AX_SUCCESS && !value.is_null() {
            Some(Cf(value))
        } else {
            None
        }
    }

    fn frame_of(element: CFTypeRef) -> Option<Rect> {
        let pos_val = copy_attr(element, "AXPosition")?;
        let size_val = copy_attr(element, "AXSize")?;
        let mut pos = CGPoint::default();
        let mut size = CGSize::default();
        let ok_pos = unsafe {
            AXValueGetValue(
                pos_val.0,
                AX_VALUE_CGPOINT,
                &mut pos as *mut _ as *mut c_void,
            )
        };
        let ok_size = unsafe {
            AXValueGetValue(
                size_val.0,
                AX_VALUE_CGSIZE,
                &mut size as *mut _ as *mut c_void,
            )
        };
        (ok_pos && ok_size).then(|| Rect::new(pos.x, pos.y, size.width, size.height))
    }

    pub fn activate_pid(pid: i32) -> bool {
        unsafe {
            let app: *mut AnyObject = msg_send![
                class!(NSRunningApplication),
                runningApplicationWithProcessIdentifier: pid
            ];
            if app.is_null() {
                return false;
            }
            // NSApplicationActivateAllWindows = 1 << 0
            let ok: bool = msg_send![app, activateWithOptions: 1usize];
            ok
        }
    }

    pub fn frontmost_pid() -> Option<i32> {
        unsafe {
            let workspace: *mut AnyObject = msg_send![class!(NSWorkspace), sharedWorkspace];
            if workspace.is_null() {
                return None;
            }
            let app: *mut AnyObject = msg_send![workspace, frontmostApplication];
            if app.is_null() {
                return None;
            }
            let pid: i32 = msg_send![app, processIdentifier];
            (pid > 0).then_some(pid)
        }
    }

    pub fn frontmost_bundle_id() -> Option<String> {
        unsafe {
            let workspace: *mut AnyObject = msg_send![class!(NSWorkspace), sharedWorkspace];
            if workspace.is_null() {
                return None;
            }
            let app: *mut AnyObject = msg_send![workspace, frontmostApplication];
            if app.is_null() {
                return None;
            }
            let bundle_id: *mut NSString = msg_send![app, bundleIdentifier];
            if bundle_id.is_null() {
                return None;
            }
            Some((*bundle_id).to_string())
        }
    }

    /// Selected text of the focused element in the frontmost app.
    pub fn frontmost_selected_text() -> Option<String> {
        if !unsafe { AXIsProcessTrusted() } {
            return None;
        }
        let pid = frontmost_pid()?;
        if pid == std::process::id() as i32 {
            return None;
        }
        let app = Cf(unsafe { AXUIElementCreateApplication(pid) });
        if app.0.is_null() {
            return None;
        }
        unsafe { AXUIElementSetMessagingTimeout(app.0, AX_TIMEOUT_SECS) };
        let selected = copy_attr(app.0, "AXFocusedUIElement")
            .and_then(|focused| copy_attr(focused.0, "AXSelectedText"));
        let Some(selected) = selected else {
            // Next time this app will answer, if it is an Electron app.
            enable_manual_accessibility(app.0, pid);
            return None;
        };
        // AXSelectedText is a CFString, toll-free bridged to NSString.
        let object = selected.0 as *mut AnyObject;
        let is_string: bool = unsafe { msg_send![object, isKindOfClass: class!(NSString)] };
        if !is_string {
            return None;
        }
        let text = unsafe { (*(selected.0 as *const NSString)).to_string() };
        (!text.trim().is_empty()).then_some(text)
    }

    /// (focused window frame, focused element frame) of the frontmost app.
    pub fn frontmost_frames() -> Option<(Option<Rect>, Option<Rect>)> {
        if !unsafe { AXIsProcessTrusted() } {
            return None;
        }
        let pid = frontmost_pid()?;
        let app = Cf(unsafe { AXUIElementCreateApplication(pid) });
        if app.0.is_null() {
            return None;
        }
        unsafe { AXUIElementSetMessagingTimeout(app.0, AX_TIMEOUT_SECS) };
        let window = copy_attr(app.0, "AXFocusedWindow").and_then(|w| frame_of(w.0));
        let focused = copy_attr(app.0, "AXFocusedUIElement").and_then(|f| frame_of(f.0));
        if window.is_none() && focused.is_none() {
            return None;
        }
        Some((window, focused))
    }
}
