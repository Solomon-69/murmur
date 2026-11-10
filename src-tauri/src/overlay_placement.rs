//! Pure geometry for placing the recording bar. Kept free of Tauri types so it
//! can be unit tested on any platform. All values are logical points in one
//! global, top-left-origin space (what tao and the macOS Accessibility API use).

use crate::settings::{OverlayHorizontal, OverlayPosition};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl Rect {
    pub fn new(x: f64, y: f64, w: f64, h: f64) -> Self {
        Self { x, y, w, h }
    }

    pub fn right(&self) -> f64 {
        self.x + self.w
    }

    pub fn bottom(&self) -> f64 {
        self.y + self.h
    }

    pub fn center_x(&self) -> f64 {
        self.x + self.w / 2.0
    }

    pub fn contains_point(&self, px: f64, py: f64) -> bool {
        px >= self.x && px < self.right() && py >= self.y && py < self.bottom()
    }

    fn is_sane(&self) -> bool {
        self.w.is_finite() && self.h.is_finite() && self.w > 0.0 && self.h > 0.0
    }
}

/// Smallest focused text box we trust to centre on. Anything narrower (a
/// button, a search field in a toolbar) centres on the whole window instead.
pub const MIN_FOCUS_TARGET_WIDTH: f64 = 200.0;

/// Pick what to centre on: the focused text box when it is a sensible size and
/// sits inside its window, otherwise the window itself.
pub fn pick_target(window: Option<Rect>, focused: Option<Rect>) -> Option<Rect> {
    let window = window.filter(Rect::is_sane);
    let focused = focused.filter(Rect::is_sane);
    match (window, focused) {
        (Some(win), Some(f))
            if f.w >= MIN_FOCUS_TARGET_WIDTH
                && f.x >= win.x - 1.0
                && f.right() <= win.right() + 1.0 =>
        {
            Some(Rect::new(f.x, win.y, f.w, win.h))
        }
        (Some(win), _) => Some(win),
        (None, Some(f)) if f.w >= MIN_FOCUS_TARGET_WIDTH => Some(f),
        _ => None,
    }
}

fn clamp_x(x: f64, width: f64, screen: &Rect) -> f64 {
    let max = (screen.right() - width).max(screen.x);
    x.clamp(screen.x, max)
}

/// Left edge of the bar window for the chosen horizontal mode. `screen` is the
/// full monitor; `target` is the active window/text box (if one was found).
pub fn bar_x(
    mode: OverlayHorizontal,
    screen: &Rect,
    target: Option<&Rect>,
    offset_x: f64,
    width: f64,
) -> f64 {
    let screen_centered = screen.x + (screen.w - width) / 2.0;
    let x = match mode {
        OverlayHorizontal::Screen => screen_centered,
        OverlayHorizontal::Custom => screen_centered + offset_x,
        OverlayHorizontal::ActiveWindow => match target {
            Some(t) => t.center_x() - width / 2.0,
            None => screen_centered,
        },
    };
    clamp_x(x, width, screen)
}

/// Top edge of the bar window. `bottom_limit` is the lowest y the bar's
/// bottom may reach on this screen (the work area bottom on macOS, so the
/// Dock is never covered).
#[allow(clippy::too_many_arguments)]
pub fn bar_y(
    position: OverlayPosition,
    mode: OverlayHorizontal,
    screen: &Rect,
    bottom_limit: f64,
    target: Option<&Rect>,
    height: f64,
    top_offset: f64,
    bottom_offset: f64,
) -> f64 {
    // The bar stays at the screen edge vertically, so it never covers the
    // text box being typed in; only its horizontal centre follows the window.
    let _ = (mode, target);
    match position {
        OverlayPosition::Top => screen.y + top_offset,
        OverlayPosition::Bottom => (bottom_limit - height - bottom_offset).max(screen.y + top_offset),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const W: f64 = 266.0;
    const H: f64 = 50.0;

    fn screen() -> Rect {
        Rect::new(0.0, 0.0, 1470.0, 956.0)
    }

    #[test]
    fn screen_mode_centres_on_monitor() {
        let x = bar_x(OverlayHorizontal::Screen, &screen(), None, 0.0, W);
        assert_eq!(x + W / 2.0, 735.0);
    }

    #[test]
    fn active_window_centres_on_target() {
        // A window with a sidebar: its chat box spans 300..1400.
        let win = Rect::new(100.0, 40.0, 1300.0, 880.0);
        let chat = Rect::new(300.0, 800.0, 1100.0, 60.0);
        let target = pick_target(Some(win), Some(chat)).unwrap();
        let x = bar_x(OverlayHorizontal::ActiveWindow, &screen(), Some(&target), 0.0, W);
        assert!((x + W / 2.0 - 850.0).abs() < 0.001, "bar centre {}", x + W / 2.0);
    }

    #[test]
    fn active_window_without_target_falls_back_to_screen() {
        let x = bar_x(OverlayHorizontal::ActiveWindow, &screen(), None, 0.0, W);
        assert_eq!(x + W / 2.0, 735.0);
    }

    #[test]
    fn tiny_focus_uses_whole_window() {
        let win = Rect::new(100.0, 40.0, 800.0, 600.0);
        let button = Rect::new(120.0, 50.0, 40.0, 20.0);
        assert_eq!(pick_target(Some(win), Some(button)), Some(win));
    }

    #[test]
    fn focus_outside_window_is_ignored() {
        let win = Rect::new(100.0, 40.0, 800.0, 600.0);
        let stray = Rect::new(1000.0, 50.0, 400.0, 20.0);
        assert_eq!(pick_target(Some(win), Some(stray)), Some(win));
    }

    #[test]
    fn custom_offset_moves_and_clamps() {
        let x = bar_x(OverlayHorizontal::Custom, &screen(), None, 120.0, W);
        assert_eq!(x + W / 2.0, 855.0);
        let far = bar_x(OverlayHorizontal::Custom, &screen(), None, 5000.0, W);
        assert_eq!(far, 1470.0 - W);
        let far_left = bar_x(OverlayHorizontal::Custom, &screen(), None, -5000.0, W);
        assert_eq!(far_left, 0.0);
    }

    #[test]
    fn window_partly_off_screen_is_clamped() {
        let win = Rect::new(-600.0, 40.0, 800.0, 600.0);
        let x = bar_x(OverlayHorizontal::ActiveWindow, &screen(), Some(&win), 0.0, W);
        assert_eq!(x, 0.0);
    }

    #[test]
    fn bottom_follows_window_and_stays_on_screen() {
        let s = screen();
        let win = Rect::new(100.0, 40.0, 1000.0, 500.0);
        let y = bar_y(
            OverlayPosition::Bottom,
            OverlayHorizontal::ActiveWindow,
            &s,
            900.0,
            Some(&win),
            H,
            46.0,
            15.0,
        );
        assert_eq!(y, 900.0 - H - 15.0, "bar never covers the text box");
        // A window running past the work area bottom keeps the bar above it.
        let tall = Rect::new(100.0, 40.0, 1000.0, 2000.0);
        let y = bar_y(
            OverlayPosition::Bottom,
            OverlayHorizontal::ActiveWindow,
            &s,
            900.0,
            Some(&tall),
            H,
            46.0,
            15.0,
        );
        assert_eq!(y, 900.0 - H - 15.0);
    }

    #[test]
    fn screen_mode_ignores_window_vertically() {
        let s = screen();
        let win = Rect::new(100.0, 40.0, 1000.0, 500.0);
        let y = bar_y(
            OverlayPosition::Bottom,
            OverlayHorizontal::Screen,
            &s,
            900.0,
            Some(&win),
            H,
            46.0,
            15.0,
        );
        assert_eq!(y, 900.0 - H - 15.0);
        let top = bar_y(
            OverlayPosition::Top,
            OverlayHorizontal::Screen,
            &s,
            900.0,
            Some(&win),
            H,
            46.0,
            15.0,
        );
        assert_eq!(top, 46.0);
    }
}
