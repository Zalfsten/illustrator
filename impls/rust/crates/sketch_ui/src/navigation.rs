//! Canvas navigation input: device events → navigation intents
//! (system-architecture: zoom around an anchor, pan by a screen displacement).
//!
//! Pure functions, so the desktop bindings are testable without an egui frame.
//! The wheel's unit tells a mouse wheel (lines) from a touchpad (points) on
//! most native platforms. Browsers differ: Firefox reports a notch as 3 lines,
//! Chromium as 50–120 pixels. So one event zooms by at most one notch, and in
//! the browser a large, purely vertical pixel event counts as a notch.
//! [`WheelMode`] overrides the classification where it still guesses wrong.
//! Pans convert lines and pages to points, as egui's own scrolling does.

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
/// Points per wheel line: the zoom of touchpad points, and the pan distance
/// of one line (one mouse-wheel notch natively).
const POINTS_PER_LINE: f64 = 50.0;
/// Lines per page.
const LINES_PER_PAGE: f64 = 10.0;
/// Whether the notch rule applies: only in the browser, where Chromium reports
/// mouse wheels in pixels. Native platforms report wheels in lines and
/// touchpads in points, so there a fast touchpad flick stays a pan.
const NOTCH_RULE: bool = cfg!(target_arch = "wasm32");
/// Smallest purely vertical pixel delta read as a mouse-wheel notch in
/// Automatic mode in the browser (Chromium: about 53 px on Linux, 100–120 px elsewhere;
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

/// `delta` in screen points (pan distance).
fn points(unit: MouseWheelUnit, delta: Vec2) -> Vec2 {
    let per_unit = match unit {
        MouseWheelUnit::Point => 1.0,
        MouseWheelUnit::Line => POINTS_PER_LINE,
        MouseWheelUnit::Page => POINTS_PER_LINE * LINES_PER_PAGE,
    };
    delta * per_unit as f32
}

/// The intent of one wheel event (first match wins): Ctrl / Cmd zooms, Shift
/// pans horizontally, then the mode decides (Automatic: lines and pages zoom,
/// points pan, and in the browser notch-sized vertical point deltas zoom).
/// `delta` points where the content should move; pans are in screen points.
pub fn wheel_intent(unit: MouseWheelUnit, delta: Vec2, mods: Modifiers, mode: WheelMode) -> Option<NavIntent> {
    wheel_intent_with(unit, delta, mods, mode, NOTCH_RULE)
}

/// [`wheel_intent`] with the notch rule switched explicitly (tests both builds).
pub(crate) fn wheel_intent_with(
    unit: MouseWheelUnit,
    delta: Vec2,
    mods: Modifiers,
    mode: WheelMode,
    notch_rule: bool,
) -> Option<NavIntent> {
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
        let d = points(unit, delta);
        return Some(NavIntent::Pan(Vec2::new(d.x + d.y, 0.0)));
    }
    let zooms = match mode {
        WheelMode::Zoom => true,
        WheelMode::Pan => false,
        WheelMode::Automatic => match unit {
            MouseWheelUnit::Line | MouseWheelUnit::Page => true,
            MouseWheelUnit::Point => notch_rule && delta.x == 0.0 && delta.y.abs() >= NOTCH_MIN_POINTS,
        },
    };
    Some(if zooms { zoom() } else { NavIntent::Pan(points(unit, delta)) })
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
