use super::*;

const LINE: MouseWheelUnit = MouseWheelUnit::Line;
const POINT: MouseWheelUnit = MouseWheelUnit::Point;
const PAGE: MouseWheelUnit = MouseWheelUnit::Page;

fn up(y: f32) -> Vec2 {
    Vec2::new(0.0, y)
}

fn ctrl() -> Modifiers {
    Modifiers { ctrl: true, ..Modifiers::NONE }
}

fn shift() -> Modifiers {
    Modifiers { shift: true, ..Modifiers::NONE }
}

fn is_zoom(i: Option<NavIntent>) -> bool {
    matches!(i, Some(NavIntent::Zoom { .. }))
}

#[test]
fn ctrl_wheel_zooms_in_every_mode() {
    for mode in [WheelMode::Automatic, WheelMode::Zoom, WheelMode::Pan] {
        for unit in [LINE, POINT, PAGE] {
            assert!(is_zoom(wheel_intent(unit, up(1.0), ctrl(), mode)), "{mode:?} {unit:?}");
        }
    }
}

#[test]
fn shift_wheel_pans_horizontally() {
    for mode in [WheelMode::Automatic, WheelMode::Zoom] {
        assert_eq!(wheel_intent(POINT, up(30.0), shift(), mode), Some(NavIntent::Pan(Vec2::new(30.0, 0.0))));
        // Lines are converted to points: one native notch pans 50 points,
        // a Firefox notch (3 lines) 150 (review of PR #106).
        assert_eq!(wheel_intent(LINE, up(1.0), shift(), mode), Some(NavIntent::Pan(Vec2::new(50.0, 0.0))));
        assert_eq!(wheel_intent(LINE, up(3.0), shift(), mode), Some(NavIntent::Pan(Vec2::new(150.0, 0.0))));
    }
}

#[test]
fn pans_convert_lines_and_pages_to_points() {
    let none = Modifiers::NONE;
    assert_eq!(wheel_intent(LINE, Vec2::new(0.5, -1.0), none, WheelMode::Pan), Some(NavIntent::Pan(Vec2::new(25.0, -50.0))));
    assert_eq!(wheel_intent(PAGE, up(1.0), none, WheelMode::Pan), Some(NavIntent::Pan(up(500.0))));
    assert_eq!(wheel_intent(POINT, up(7.0), none, WheelMode::Pan), Some(NavIntent::Pan(up(7.0))));
}

#[test]
fn mode_decides_without_modifiers() {
    let none = Modifiers::NONE;
    // (mode, unit, zooms?)
    for (mode, unit, zooms) in [
        (WheelMode::Automatic, LINE, true),
        (WheelMode::Automatic, PAGE, true),
        (WheelMode::Automatic, POINT, false),
        (WheelMode::Zoom, LINE, true),
        (WheelMode::Zoom, POINT, true),
        (WheelMode::Pan, LINE, false),
        (WheelMode::Pan, POINT, false),
    ] {
        let i = wheel_intent(unit, Vec2::new(2.0, 5.0), none, mode);
        assert_eq!(is_zoom(i), zooms, "{mode:?} {unit:?}");
        if !zooms {
            let per = if unit == LINE { 50.0 } else { 1.0 };
            assert_eq!(i, Some(NavIntent::Pan(Vec2::new(2.0, 5.0) * per)));
        }
    }
}

#[test]
fn one_line_zooms_by_1_2() {
    let Some(NavIntent::Zoom { factor }) = wheel_intent(LINE, up(1.0), Modifiers::NONE, WheelMode::Automatic) else {
        panic!("zoom expected")
    };
    assert!((factor - 1.2).abs() < 1e-12);
    let Some(NavIntent::Zoom { factor }) = wheel_intent(LINE, up(-1.0), Modifiers::NONE, WheelMode::Automatic) else {
        panic!()
    };
    assert!((factor - 1.0 / 1.2).abs() < 1e-12, "wheel down zooms out");
    assert_eq!(wheel_intent(LINE, Vec2::ZERO, Modifiers::NONE, WheelMode::Zoom), None);
}

fn factor(i: Option<NavIntent>) -> f64 {
    match i {
        Some(NavIntent::Zoom { factor }) => factor,
        other => panic!("zoom expected, got {other:?}"),
    }
}

#[test]
fn one_event_zooms_at_most_one_notch() {
    let none = Modifiers::NONE;
    // Firefox: a notch is 3 lines; Chromium with Ctrl: 100 px.
    assert!((factor(wheel_intent(LINE, up(3.0), none, WheelMode::Automatic)) - 1.2).abs() < 1e-12);
    assert!((factor(wheel_intent(LINE, up(-3.0), none, WheelMode::Automatic)) - 1.0 / 1.2).abs() < 1e-12);
    assert!((factor(wheel_intent(PAGE, up(1.0), none, WheelMode::Automatic)) - 1.2).abs() < 1e-12);
    assert!((factor(wheel_intent(POINT, up(100.0), ctrl(), WheelMode::Pan)) - 1.2).abs() < 1e-12);
    // Fractions stay smooth: half a line, 5 px of touchpad.
    assert!((factor(wheel_intent(LINE, up(0.5), none, WheelMode::Automatic)) - 1.2f64.sqrt()).abs() < 1e-12);
    assert!((factor(wheel_intent(POINT, up(5.0), none, WheelMode::Zoom)) - 1.2f64.powf(0.1)).abs() < 1e-12);
}

#[test]
fn automatic_reads_large_vertical_points_as_a_wheel_notch_in_the_browser() {
    let none = Modifiers::NONE;
    let auto = WheelMode::Automatic;
    let browser = |d: Vec2, mode| wheel_intent_with(POINT, d, none, mode, true);
    // Chromium mouse wheel: 53 px (Linux) or 100–120 px per notch.
    for y in [53.0, -53.0, 100.0, 120.0, NOTCH_MIN_POINTS] {
        assert!(is_zoom(browser(up(y), auto)), "{y}");
    }
    // Touchpad: small or diagonal deltas pan.
    for d in [up(12.0), up(NOTCH_MIN_POINTS - 1.0), Vec2::new(1.0, 60.0)] {
        assert_eq!(browser(d, auto), Some(NavIntent::Pan(d)), "{d:?}");
    }
    // The explicit Pan mode still pans a notch.
    assert_eq!(browser(up(100.0), WheelMode::Pan), Some(NavIntent::Pan(up(100.0))));
}

/// Natively the unit is reliable: a fast vertical touchpad flick stays a pan
/// (review of PR #106).
#[test]
fn native_points_always_pan_in_automatic_mode() {
    let d = up(120.0);
    assert_eq!(wheel_intent_with(POINT, d, Modifiers::NONE, WheelMode::Automatic, false), Some(NavIntent::Pan(d)));
    assert_eq!(NOTCH_RULE, cfg!(target_arch = "wasm32"));
}

#[test]
fn zoom_label_formatting() {
    assert_eq!(zoom_label(37.2), "Zoom 37 %");
    assert_eq!(zoom_label(100.0), "Zoom 100 %");
    assert_eq!(zoom_label(1249.7), "Zoom 1\u{2009}250 %");
    assert_eq!(zoom_label(12_513.0), "Zoom 12\u{2009}500 %");
}
