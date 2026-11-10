//! Tints the running app's Dock / Cmd-Tab icon to match the Appearance
//! accent colour. "default" (Grey) keeps the bundled black-and-white icon.
//!
//! Only the running app's icon changes: the icon Finder shows for the .app
//! bundle is part of the signed bundle and is never touched.

#[cfg(target_os = "macos")]
use tauri::AppHandle;

/// Greyscale source icon (rounded body on a transparent 512x512 canvas).
#[cfg(target_os = "macos")]
const BASE_ICON_RESOURCE: &str = "resources/app_icon_base.png";

/// Pixels darker than this (0-255 luminance) are background and stay as is.
const BG_LUMA_START: f32 = 55.0;
/// Width of the blend from background into the tinted ribbon (anti-aliasing).
const BG_LUMA_RAMP: f32 = 30.0;
/// Luminance where a band takes the pure accent colour.
const PURE_ACCENT_LUMA: f32 = 0.6;
/// How far the brightest band moves from the accent toward white.
const MAX_WHITE_MIX: f32 = 0.75;

/// Accent id to RGB. Keep in sync with src/lib/constants/accents.ts.
/// Returns None for "default" (Grey) and unknown ids: keep the original icon.
pub fn accent_rgb(accent_id: &str) -> Option<[u8; 3]> {
    let hex = match accent_id {
        "terracotta" => 0xc0643f,
        "olive" => 0x6f7a3c,
        "ochre" => 0xc48a2c,
        "clay" => 0x9a5b47,
        "moss" => 0x4f6b4a,
        "sand" => 0xb39567,
        "electric" => 0x3d5afe,
        "coral" => 0xff5a5f,
        "violet" => 0x7c4dff,
        "tangerine" => 0xff8a1f,
        "magenta" => 0xe83e9a,
        "aqua" => 0x00b8c4,
        "sage" => 0x8fa58c,
        "rose" => 0xc49a9a,
        "slate" => 0x7f93a8,
        "lilac" => 0x9e95b5,
        "mushroom" => 0xa39587,
        "fogteal" => 0x7fa3a0,
        _ => return None,
    };
    Some([(hex >> 16) as u8, (hex >> 8) as u8, hex as u8])
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Tints one RGBA pixel. Dark background pixels are returned unchanged; light
/// ribbon pixels become shades of `accent` that keep their luminance order, so
/// the fading bands stay visible (darker bands = deeper accent, the brightest
/// band = a pale tint).
pub fn tint_pixel(px: [u8; 4], accent: [u8; 3]) -> [u8; 4] {
    let [r, g, b, a] = px;
    let luma = 0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32;
    let weight = ((luma - BG_LUMA_START) / BG_LUMA_RAMP).clamp(0.0, 1.0);
    if weight <= 0.0 {
        return px;
    }
    let l = luma / 255.0;
    let target = accent.map(|c| {
        let c = c as f32;
        if l <= PURE_ACCENT_LUMA {
            c * (l / PURE_ACCENT_LUMA)
        } else {
            let white_mix = (l - PURE_ACCENT_LUMA) / (1.0 - PURE_ACCENT_LUMA) * MAX_WHITE_MIX;
            lerp(c, 255.0, white_mix)
        }
    });
    let mix = |orig: u8, t: f32| lerp(orig as f32, t, weight).round().clamp(0.0, 255.0) as u8;
    [mix(r, target[0]), mix(g, target[1]), mix(b, target[2]), a]
}

/// Returns a tinted copy of an RGBA8 buffer.
pub fn tint_rgba(pixels: &[u8], accent: [u8; 3]) -> Vec<u8> {
    pixels
        .chunks_exact(4)
        .flat_map(|p| tint_pixel([p[0], p[1], p[2], p[3]], accent))
        .collect()
}

#[cfg(target_os = "macos")]
fn tinted_png(path: &std::path::Path, accent: [u8; 3]) -> Result<Vec<u8>, String> {
    let image = tauri::image::Image::from_path(path).map_err(|e| format!("load icon: {e}"))?;
    let (width, height) = (image.width(), image.height());
    let rgba = image.rgba();
    if rgba.len() != (width as usize) * (height as usize) * 4 {
        return Err("icon is not RGBA8".into());
    }
    let tinted = tint_rgba(rgba, accent);

    let mut out = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut out, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder
            .write_header()
            .map_err(|e| format!("encode icon: {e}"))?;
        writer
            .write_image_data(&tinted)
            .map_err(|e| format!("encode icon: {e}"))?;
        writer.finish().map_err(|e| format!("encode icon: {e}"))?;
    }
    Ok(out)
}

/// Sets NSApp's icon from PNG bytes, or restores the bundle icon for None.
/// Must run on the main thread.
#[cfg(target_os = "macos")]
fn set_ns_app_icon(png_bytes: Option<&[u8]>) {
    use objc2::rc::{Allocated, Retained};
    use objc2::runtime::AnyObject;
    use objc2::{class, msg_send};
    use objc2_foundation::NSData;

    unsafe {
        let app: Option<Retained<AnyObject>> = msg_send![class!(NSApplication), sharedApplication];
        let Some(app) = app else {
            log::warn!("dock icon: no NSApplication");
            return;
        };
        let image: Option<Retained<AnyObject>> = match png_bytes {
            Some(bytes) => {
                let data = NSData::with_bytes(bytes);
                let alloc: Allocated<AnyObject> = msg_send![class!(NSImage), alloc];
                let image: Option<Retained<AnyObject>> = msg_send![alloc, initWithData: &*data];
                if image.is_none() {
                    log::warn!("dock icon: NSImage could not read tinted PNG");
                    return;
                }
                image
            }
            None => None,
        };
        let _: () = msg_send![&*app, setApplicationIconImage: image.as_deref()];
    }
}

/// Applies the accent to the Dock / Cmd-Tab icon. "default" restores the
/// bundled icon. Safe to call from any thread.
#[cfg(target_os = "macos")]
pub fn apply_dock_icon(app: &AppHandle, accent_id: &str) {
    use tauri::Manager;

    let png = match accent_rgb(accent_id) {
        None => None,
        Some(accent) => {
            let path = match app
                .path()
                .resolve(BASE_ICON_RESOURCE, tauri::path::BaseDirectory::Resource)
            {
                Ok(p) => p,
                Err(e) => {
                    log::warn!("dock icon: cannot resolve base icon: {e}");
                    return;
                }
            };
            match tinted_png(&path, accent) {
                Ok(bytes) => Some(bytes),
                Err(e) => {
                    log::warn!("dock icon: {e}");
                    return;
                }
            }
        }
    };
    if let Err(e) = app.run_on_main_thread(move || set_ns_app_icon(png.as_deref())) {
        log::warn!("dock icon: main thread dispatch failed: {e}");
    }
}

#[cfg(not(target_os = "macos"))]
pub fn apply_dock_icon(_app: &tauri::AppHandle, _accent_id: &str) {}

#[cfg(test)]
mod tests {
    use super::*;

    const TERRACOTTA: [u8; 3] = [0xc0, 0x64, 0x3f];

    fn luma(p: [u8; 4]) -> f32 {
        0.2126 * p[0] as f32 + 0.7152 * p[1] as f32 + 0.0722 * p[2] as f32
    }

    #[test]
    fn background_pixels_stay_unchanged() {
        for px in [[8, 8, 9, 255], [42, 42, 46, 255], [0, 0, 0, 0]] {
            assert_eq!(tint_pixel(px, TERRACOTTA), px);
        }
    }

    #[test]
    fn ribbon_bands_take_accent_hue_and_keep_order() {
        let bands = [90u8, 120, 175, 244];
        let tinted: Vec<[u8; 4]> = bands
            .iter()
            .map(|&v| tint_pixel([v, v, v, 255], TERRACOTTA))
            .collect();
        for t in &tinted {
            // Terracotta is red-dominant: red > green > blue in every band.
            assert!(t[0] > t[1] && t[1] > t[2], "{t:?} not terracotta-ish");
            assert_eq!(t[3], 255);
        }
        for pair in tinted.windows(2) {
            assert!(luma(pair[0]) < luma(pair[1]), "bands lost their order");
        }
    }

    #[test]
    fn alpha_is_preserved() {
        assert_eq!(tint_pixel([240, 240, 240, 77], TERRACOTTA)[3], 77);
    }

    #[test]
    fn tint_rgba_maps_every_pixel() {
        let src = [8, 8, 9, 255, 244, 244, 242, 255];
        let out = tint_rgba(&src, TERRACOTTA);
        assert_eq!(out.len(), src.len());
        assert_eq!(&out[..4], &src[..4]);
        assert_ne!(&out[4..], &src[4..]);
    }

    #[test]
    fn default_and_unknown_accents_keep_original_icon() {
        assert_eq!(accent_rgb("default"), None);
        assert_eq!(accent_rgb("neon"), None);
        assert_eq!(accent_rgb("electric"), Some([0x3d, 0x5a, 0xfe]));
    }

    #[test]
    fn every_non_default_accent_has_a_colour() {
        for id in crate::settings::OVERLAY_ACCENTS {
            assert_eq!(accent_rgb(id).is_none(), id == "default", "{id}");
        }
    }
}
