//! macOS menu bar dropdown: a small popover-style panel that opens right under
//! the menu bar icon on left click, right click and Option-click, instead of
//! the plain native menu. On macOS 26+ it sits on a native Liquid Glass
//! background (NSGlassEffectView); older macOS gets the popover material. It hides when it loses
//! focus or when the icon is clicked again.

use serde::Serialize;
use specta::Type;
use tauri::{AppHandle, Emitter, Manager};

pub const POPOVER_LABEL: &str = "tray_popover";
/// Logical size of the panel. Width is fixed; the page reports its real
/// height through `tray_popover_resize`. Keep in sync with popover.css.
pub const POPOVER_WIDTH: f64 = 288.0;
pub const POPOVER_HEIGHT: f64 = 330.0;
/// Corner radius of the glass panel, matching macOS 26 menus.
const CORNER_RADIUS: f64 = 14.0;
/// Gap between the bottom of the menu bar icon and the panel.
const ICON_GAP: f64 = 1.0;

/// Where the panel's top-left corner goes, in logical pixels, so the arrow sits
/// centred right under the icon and the panel stays on screen.
pub fn popover_origin(
    icon_x: f64,
    icon_y: f64,
    icon_w: f64,
    icon_h: f64,
    screen_x: f64,
    screen_w: f64,
) -> (f64, f64) {
    let centre = icon_x + icon_w / 2.0;
    let min_x = screen_x + 8.0;
    let max_x = (screen_x + screen_w - POPOVER_WIDTH - 8.0).max(min_x);
    let x = (centre - POPOVER_WIDTH / 2.0).clamp(min_x, max_x);
    (x, icon_y + icon_h + ICON_GAP)
}

/// Horizontal arrow position inside the panel (px from its left edge).
pub fn arrow_offset(icon_x: f64, icon_w: f64, panel_x: f64) -> f64 {
    (icon_x + icon_w / 2.0 - panel_x).clamp(18.0, POPOVER_WIDTH - 18.0)
}

fn ensure_window(app: &AppHandle) -> Option<tauri::WebviewWindow> {
    if let Some(w) = app.get_webview_window(POPOVER_LABEL) {
        return Some(w);
    }
    let builder = tauri::WebviewWindowBuilder::new(
        app,
        POPOVER_LABEL,
        tauri::WebviewUrl::App("src/popover/index.html".into()),
    )
    .title("murmur")
    .inner_size(POPOVER_WIDTH, POPOVER_HEIGHT)
    .resizable(false)
    .decorations(false)
    .transparent(true)
    .shadow(true)
    .always_on_top(true)
    .skip_taskbar(true)
    .visible(false)
    .focused(true);
    match builder.build() {
        Ok(w) => {
            #[cfg(target_os = "macos")]
            apply_glass(&w);
            let handle = w.clone();
            w.on_window_event(move |event| {
                if let tauri::WindowEvent::Focused(false) = event {
                    let _ = handle.emit("popover-close", ());
                    let _ = handle.hide();
                }
            });
            Some(w)
        }
        Err(e) => {
            log::error!("Failed to create the menu bar popover: {e}");
            None
        }
    }
}

/// Opens the panel under the icon, or closes it when it is already open.
/// `rect` is the icon's position and size in physical pixels.
pub fn toggle(app: &AppHandle, rect: tauri::Rect) {
    let Some(window) = ensure_window(app) else {
        return;
    };
    if window.is_visible().unwrap_or(false) {
        let _ = window.emit("popover-close", ());
        let _ = window.hide();
        return;
    }
    let scale = window.scale_factor().unwrap_or(2.0);
    let pos = rect.position.to_logical::<f64>(scale);
    let size = rect.size.to_logical::<f64>(scale);
    let (screen_x, screen_w) = app
        .monitor_from_point(pos.x, pos.y)
        .ok()
        .flatten()
        .or_else(|| window.primary_monitor().ok().flatten())
        .map(|m| {
            let s = m.scale_factor();
            (m.position().x as f64 / s, m.size().width as f64 / s)
        })
        .unwrap_or((0.0, 1440.0));
    let (x, y) = popover_origin(pos.x, pos.y, size.width, size.height, screen_x, screen_w);
    let _ = window.set_position(tauri::Position::Logical(tauri::LogicalPosition { x, y }));
    let _ = window.emit("popover-open", arrow_offset(pos.x, size.width, x));
    let _ = window.show();
    let _ = window.set_focus();
}

/// Puts a native glass view behind the (transparent) web page, rounds it, and
/// lets macOS draw the window shadow from that rounded shape, so there is no
/// square shadow around the panel.
#[cfg(target_os = "macos")]
fn apply_glass(window: &tauri::WebviewWindow) {
    use objc2::msg_send;
    use objc2::runtime::{AnyClass, AnyObject};
    use objc2_foundation::NSRect;

    let Ok(ptr) = window.ns_window() else {
        return;
    };
    // SAFETY: plain AppKit calls on the main-thread window Tauri just made.
    unsafe {
        let ns_window = ptr as *mut AnyObject;
        let content: *mut AnyObject = msg_send![ns_window, contentView];
        if content.is_null() {
            return;
        }
        let bounds: NSRect = msg_send![content, bounds];
        let view: *mut AnyObject = if let Some(cls) = AnyClass::get(c"NSGlassEffectView") {
            let v: *mut AnyObject = msg_send![cls, alloc];
            let v: *mut AnyObject = msg_send![v, initWithFrame: bounds];
            let _: () = msg_send![v, setCornerRadius: CORNER_RADIUS];
            v
        } else {
            let Some(cls) = AnyClass::get(c"NSVisualEffectView") else {
                return;
            };
            let v: *mut AnyObject = msg_send![cls, alloc];
            let v: *mut AnyObject = msg_send![v, initWithFrame: bounds];
            let _: () = msg_send![v, setMaterial: 6isize]; // .popover
            let _: () = msg_send![v, setBlendingMode: 0isize]; // .behindWindow
            let _: () = msg_send![v, setState: 1isize]; // .active
            let _: () = msg_send![v, setWantsLayer: true];
            let layer: *mut AnyObject = msg_send![v, layer];
            if !layer.is_null() {
                let _: () = msg_send![layer, setCornerRadius: CORNER_RADIUS];
                let _: () = msg_send![layer, setMasksToBounds: true];
            }
            v
        };
        let _: () = msg_send![view, setAutoresizingMask: 18usize]; // width + height
        let below: isize = -1; // NSWindowBelow
        let none: *mut AnyObject = std::ptr::null_mut();
        let _: () = msg_send![content, addSubview: view, positioned: below, relativeTo: none];
        let _: () = msg_send![ns_window, setHasShadow: true];
        let _: () = msg_send![ns_window, invalidateShadow];
    }
}

/// The page tells us its real height so the glass fits the content exactly.
#[tauri::command]
#[specta::specta]
pub fn tray_popover_resize(app: AppHandle, height: f64) {
    let Some(w) = app.get_webview_window(POPOVER_LABEL) else {
        return;
    };
    let h = height.clamp(120.0, 600.0).round();
    let _ = w.set_size(tauri::Size::Logical(tauri::LogicalSize {
        width: POPOVER_WIDTH,
        height: h,
    }));
    #[cfg(target_os = "macos")]
    if let Ok(ptr) = w.ns_window() {
        use objc2::{msg_send, runtime::AnyObject};
        // SAFETY: AppKit call on the window Tauri owns.
        unsafe {
            let _: () = msg_send![ptr as *mut AnyObject, invalidateShadow];
        }
    }
}

pub fn hide(app: &AppHandle) {
    if let Some(w) = app.get_webview_window(POPOVER_LABEL) {
        let _ = w.emit("popover-close", ());
        let _ = w.hide();
    }
}

#[derive(Serialize, Type, Clone)]
pub struct PopoverState {
    pub version: String,
    pub busy: bool,
    pub show_in_dock: bool,
    pub update_checks_enabled: bool,
}

#[tauri::command]
#[specta::specta]
pub fn tray_popover_state(app: AppHandle) -> PopoverState {
    let settings = crate::settings::get_settings(&app);
    PopoverState {
        version: app.package_info().version.to_string(),
        busy: crate::tray::dictation_in_progress(&app) || crate::overlay::is_session_showing(),
        show_in_dock: settings.show_in_dock,
        update_checks_enabled: crate::settings::update_checks_effectively_enabled(&settings),
    }
}

/// Runs a dropdown button: the same actions as the native tray menu.
#[tauri::command]
#[specta::specta]
pub fn tray_popover_action(app: AppHandle, action: String) -> Result<(), String> {
    match action.as_str() {
        "cancel" | "settings" | "copy_last_transcript" | "quit" => {
            if action != "cancel" {
                hide(&app);
            }
            crate::handle_tray_menu_action(&app, &action);
            Ok(())
        }
        other => Err(format!("Unknown action '{other}'")),
    }
}

/// "Update App": checks the latest GitHub release of Solomon-69/murmur (the
/// updater feed in tauri.conf.json), downloads and installs it, then restarts.
/// Returns "latest" when this is already the newest version.
#[tauri::command]
#[specta::specta]
pub async fn update_app(app: AppHandle) -> Result<String, String> {
    use tauri_plugin_updater::UpdaterExt;
    let settings = crate::settings::get_settings(&app);
    if !crate::settings::update_checks_effectively_enabled(&settings) {
        return Err("Updates are turned off".to_string());
    }
    let updater = app.updater().map_err(|e| e.to_string())?;
    let Some(update) = updater.check().await.map_err(|e| e.to_string())? else {
        return Ok("latest".to_string());
    };
    let version = update.version.clone();
    let _ = app.emit("update-app-progress", format!("Downloading {version}"));
    update
        .download_and_install(|_, _| {}, || {})
        .await
        .map_err(|e| e.to_string())?;
    let _ = app.emit("update-app-progress", format!("Installed {version}"));
    app.restart();
}

/// Automatic update check, so the app updates even when only the menu bar
/// icon is open (the main window, which used to run the check, may never
/// open). First check a minute after launch, then every 6 hours. A found
/// update is downloaded and installed; the app restarts into it only while
/// no dictation is running, otherwise the new version starts next launch.
pub fn start_auto_update(app: &AppHandle) {
    let app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(60));
        loop {
            tauri::async_runtime::block_on(auto_update_once(&app));
            std::thread::sleep(std::time::Duration::from_secs(6 * 60 * 60));
        }
    });
}

async fn auto_update_once(app: &AppHandle) {
    use tauri_plugin_updater::UpdaterExt;
    let settings = crate::settings::get_settings(app);
    if !crate::settings::update_checks_effectively_enabled(&settings) {
        return;
    }
    log::info!("Automatic update check");
    let update = match app.updater() {
        Ok(u) => match u.check().await {
            Ok(Some(update)) => update,
            Ok(None) => return,
            Err(e) => {
                log::warn!("Automatic update check failed: {e}");
                return;
            }
        },
        Err(e) => {
            log::warn!("Updater unavailable: {e}");
            return;
        }
    };
    let version = update.version.clone();
    log::info!("Installing update {version}");
    if let Err(e) = update.download_and_install(|_, _| {}, || {}).await {
        log::warn!("Automatic update {version} failed: {e}");
        return;
    }
    let busy = crate::tray::dictation_in_progress(app) || crate::overlay::is_session_showing();
    if busy {
        log::info!("Update {version} installed; starts at next launch");
    } else {
        log::info!("Update {version} installed; restarting");
        app.restart();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arrow_sits_under_the_icon() {
        let (x, y) = popover_origin(1000.0, 0.0, 24.0, 24.0, 0.0, 1440.0);
        assert_eq!(x, 1012.0 - POPOVER_WIDTH / 2.0);
        assert_eq!(y, 24.0 + ICON_GAP);
        assert_eq!(arrow_offset(1000.0, 24.0, x), POPOVER_WIDTH / 2.0);
    }

    #[test]
    fn panel_stays_on_screen_near_the_right_edge() {
        let (x, _) = popover_origin(1420.0, 0.0, 20.0, 24.0, 0.0, 1440.0);
        assert!(x + POPOVER_WIDTH <= 1440.0);
        let arrow = arrow_offset(1420.0, 20.0, x);
        assert!(arrow > POPOVER_WIDTH / 2.0 && arrow <= POPOVER_WIDTH - 18.0);
    }
}
