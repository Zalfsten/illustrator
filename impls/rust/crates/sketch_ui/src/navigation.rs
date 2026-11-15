//! Canvas navigation input: device events → navigation intents
//! (system-architecture: zoom around an anchor, pan by a screen displacement).
//!
//! Pure functions, so the desktop bindings are testable without an egui frame.
//! The wheel's unit tells a mouse wheel (lines) from a touchpad (points) on
//! most native platforms. Browsers differ: Firefox reports a notch as 3 lines,
//! Chromium as 50–120 pixels. So one event zooms by at most one notch, and a
//! large, purely vertical pixel event counts as a notch. [`WheelMode`]
//! overrides the classification where it still guesses wrong.

use eframe::egui::{Modifiers, MouseWheelUnit, Vec2};

/// What the mouse wheel / two-finger scroll does (session setting).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WheelMode {
    /// Line / page units (mouse wheel) zoom, point units (touchpad) pan.
    #[default]
    Automatic,
    Zoom,
    Pan,
}

/// A change of the view requested by input.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NavIntent {
    /// Multiply the scale by `factor` around the pointer.
    Zoom { factor: f64 },
    /// Move the view's content by a screen displacement.
    Pan(Vec2),
}

/// Zoom factor per wheel line (one mouse-wheel notch).
pub const ZOOM_PER_LINE: f64 = 1.2;
/// Points per line equivalent for touchpad zooming.
const POINTS_PER_LINE: f64 = 50.0;
/// Lines per page.
const LINES_PER_PAGE: f64 = 10.0;
/// Smallest purely vertical pixel delta read as a mouse-wheel notch in
/// Automatic mode (Chromium: about 53 px on Linux, 100–120 px elsewhere;
/// touchpad scroll events are mostly smaller and rarely purely vertical).
pub const NOTCH_MIN_POINTS: f32 = 40.0;

/// Wheel lines `delta_y` corresponds to.
fn lines(unit: MouseWheelUnit, delta_y: f32) -> f64 {
    let d = f64::from(delta_y);
    match unit {
        MouseWheelUnit::Line => d,
        MouseWheelUnit::Point => d / POINTS_PER_LINE,
        MouseWheelUnit::Page => d * LINES_PER_PAGE,
    }
}

/// The intent of one wheel event (first match wins): Ctrl / Cmd zooms, Shift
/// pans horizontally, then the mode decides (Automatic: lines, pages and
/// notch-sized vertical point deltas zoom, other points pan). `delta` points where the content should move.
pub fn wheel_intent(unit: MouseWheelUnit, delta: Vec2, mods: Modifiers, mode: WheelMode) -> Option<NavIntent> {
    if delta == Vec2::ZERO {
        return None;
    }
    // At most one notch per event: browsers report a notch as several lines
    // or about a hundred pixels; fractions (high-resolution wheels, touchpads)
    // stay smooth.
    let zoom = || NavIntent::Zoom {
        factor: ZOOM_PER_LINE.powf(lines(unit, delta.y).clamp(-1.0, 1.0)),
    };
    if mods.ctrl || mods.command || mods.mac_cmd {
        return Some(zoom());
    }
    if mods.shift {
        return Some(NavIntent::Pan(Vec2::new(delta.x + delta.y, 0.0)));
    }
    let zooms = match mode {
        WheelMode::Zoom => true,
        WheelMode::Pan => false,
        WheelMode::Automatic => match unit {
            MouseWheelUnit::Line | MouseWheelUnit::Page => true,
            MouseWheelUnit::Point => delta.x == 0.0 && delta.y.abs() >= NOTCH_MIN_POINTS,
        },
    };
    Some(if zooms { zoom() } else { NavIntent::Pan(delta) })
}

/// `Zoom N %` text: whole percent below 1000, three significant digits above
/// (with a thin space as thousands separator).
pub fn zoom_label(percent: f64) -> String {
    let rounded = if percent < 1000.0 {
        percent.round()
    } else {
        let mag = 10f64.powi(percent.log10().floor() as i32 - 2);
        (percent / mag).round() * mag
    };
    let digits = format!("{}", rounded as u64);
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push('\u{2009}');
        }
        out.push(c);
    }
    format!("Zoom {out} %")
}

#[cfg(test)]
#[path = "tests_navigation.rs"]
mod tests_navigation;
