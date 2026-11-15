use super::*;
use eframe::egui::Rect;

#[test]
fn round_trip_world_screen() {
    let vp = Viewport::new(100.0, Pos2::new(200.0, 300.0));
    let w = Point2::new(1.5, 2.0);
    let s = vp.to_screen(w);
    let back = vp.to_world(s);
    assert!((back.x - w.x).abs() < 1e-5);
    assert!((back.y - w.y).abs() < 1e-5);
}

#[test]
fn increasing_world_y_decreases_screen_y() {
    let vp = Viewport::new(50.0, Pos2::new(0.0, 400.0));
    let low = vp.to_screen(Point2::new(0.0, 0.0));
    let high = vp.to_screen(Point2::new(0.0, 2.0));
    assert!(high.y < low.y);
}

#[test]
fn px_to_world_divides_by_scale() {
    let vp = Viewport::new(40.0, Pos2::new(0.0, 0.0));
    assert!((vp.px_to_world(12.0) - 12.0 / 40.0).abs() < 1e-12);
}

#[test]
fn fit_maps_bounds_inside_canvas() {
    let canvas = Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(400.0, 300.0));
    let world_min = Point2::new(0.0, 0.0);
    let world_max = Point2::new(2.0, 2.0);
    let vp = Viewport::fit(canvas, world_min, world_max, 20.0);
    assert!(vp.scale > 0.0);

    let corners = [
        world_min,
        Point2::new(world_max.x, world_min.y),
        Point2::new(world_min.x, world_max.y),
        world_max,
    ];
    for c in corners {
        let s = vp.to_screen(c);
        assert!(
            s.x >= canvas.min.x - 1e-3 && s.x <= canvas.max.x + 1e-3,
            "x out of canvas: {s:?}"
        );
        assert!(
            s.y >= canvas.min.y - 1e-3 && s.y <= canvas.max.y + 1e-3,
            "y out of canvas: {s:?}"
        );
    }
}

#[test]
fn world_box_adds_a_margin_and_a_minimum_size() {
    assert_eq!(world_box([]), None);
    let (min, max) = world_box([Point2::new(40.0, 20.0), Point2::new(60.0, 30.0)]).unwrap();
    assert!((min.x - 38.0).abs() < 1e-9 && (max.x - 62.0).abs() < 1e-9);
    assert!((min.y - 19.0).abs() < 1e-9 && (max.y - 31.0).abs() < 1e-9);
    let (min, max) = world_box([Point2::new(5.0, 5.0)]).unwrap();
    assert!((max.x - min.x - 1.2).abs() < 1e-9 && (max.y - min.y - 1.2).abs() < 1e-9);
    assert!((min.x + max.x - 10.0).abs() < 1e-9, "centred on the point");
}

// --- View state (system-architecture) ---------------------------------------

const BOUNDS: (f64, f64) = (1e-6, 1e9);

fn canvas(w: f32, h: f32) -> Rect {
    Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(w, h))
}

fn near(a: Point2, b: Point2, tol: f64) -> bool {
    (a - b).length() < tol
}

#[test]
fn zoom_keeps_the_anchor_point_fixed() {
    let c = canvas(800.0, 600.0);
    let mut v = ViewState { center: Point2::new(1.0, 2.0), scale: 50.0 };
    let anchor = v.viewport(c).to_screen(Point2::new(3.0, 4.0));
    v.zoom(c, anchor, 2.0, BOUNDS);
    assert_eq!(v.scale, 100.0);
    assert!(near(v.viewport(c).to_world(anchor), Point2::new(3.0, 4.0), 1e-4));
}

#[test]
fn pan_moves_every_screen_image() {
    let c = canvas(800.0, 600.0);
    let mut v = ViewState { center: Point2::new(0.0, 0.0), scale: 40.0 };
    let p = Point2::new(1.5, -2.0);
    let before = v.viewport(c).to_screen(p);
    v.pan(ScreenVec::new(50.0, 0.0));
    let after = v.viewport(c).to_screen(p);
    assert!((after.x - before.x - 50.0).abs() < 1e-3 && (after.y - before.y).abs() < 1e-3);
    assert_eq!(v.scale, 40.0);
}

#[test]
fn wider_canvas_shows_more() {
    let v = ViewState { center: Point2::new(0.0, 0.0), scale: 100.0 };
    let (narrow, wide) = (canvas(400.0, 300.0), canvas(800.0, 300.0));
    // Same center and scale: the center maps to each canvas's center.
    assert!((v.viewport(narrow).to_screen(v.center) - narrow.center()).length() < 1e-3);
    assert!((v.viewport(wide).to_screen(v.center) - wide.center()).length() < 1e-3);
    // The narrow view spans x ∈ [-2, 2] at scale 100; x = -2.2 lies just outside it.
    let p = Point2::new(-2.2, 0.0);
    assert!(!narrow.contains(v.viewport(narrow).to_screen(p)));
    assert!(wide.contains(v.viewport(wide).to_screen(p)));
}

#[test]
fn fit_puts_the_box_inside_the_canvas() {
    let c = canvas(800.0, 600.0);
    let v = ViewState::fit(c, Point2::new(40.0, 20.0), Point2::new(60.0, 35.0), 40.0, BOUNDS);
    let vp = v.viewport(c);
    for p in [Point2::new(40.0, 20.0), Point2::new(60.0, 35.0), Point2::new(40.0, 35.0)] {
        assert!(c.contains(vp.to_screen(p)), "{p:?}");
    }
}

#[test]
fn zoom_stops_at_its_bounds_with_the_anchor_fixed() {
    let c = canvas(800.0, 600.0);
    let bounds = (10.0, 1000.0);
    let mut v = ViewState { center: Point2::new(0.0, 0.0), scale: 100.0 };
    let anchor = Pos2::new(600.0, 150.0);
    let w = v.viewport(c).to_world(anchor);
    for _ in 0..5 {
        v.zoom(c, anchor, 10.0, bounds);
    }
    assert_eq!(v.scale, 1000.0);
    assert!(near(v.viewport(c).to_world(anchor), w, 1e-6));
    for _ in 0..5 {
        v.zoom(c, anchor, 0.1, bounds);
    }
    assert_eq!(v.scale, 10.0);
    assert!(near(v.viewport(c).to_world(anchor), w, 1e-4));
}

#[test]
fn one_hundred_percent_is_print_size() {
    let settings = constraint_runtime::DocumentSettings { scale: 10.0, ..Default::default() };
    let s = print_scale(&settings);
    assert!((s - 10.0 * 96.0 / 25.4).abs() < 1e-12 && (s - 37.795).abs() < 1e-3);
    let v = ViewState { center: Point2::new(0.0, 0.0), scale: s };
    assert!((v.percent(s) - 100.0).abs() < 1e-12);
    let (lo, hi) = zoom_bounds(&settings);
    assert!((lo / s - 0.01).abs() < 1e-12 && (hi / s - 1000.0).abs() < 1e-9);
}

/// Zoomed in far from the world origin the screen origin is millions of
/// units away; kept in `f64`, the mapping stays exact and a sub-pixel pan
/// moves the drawing smoothly (review of PR #106).
#[test]
fn mapping_stays_exact_far_from_the_origin_at_high_zoom() {
    let c = canvas(800.0, 600.0);
    let mut v = ViewState { center: Point2::new(1000.0, -1000.0), scale: 37_795.0 };
    assert!(near(v.viewport(c).to_world(c.center()), v.center, 1e-9));
    let p = Point2::new(1000.001, -1000.002);
    let before = v.viewport(c).to_screen(p);
    v.pan(egui::vec2(0.5, 0.25));
    let after = v.viewport(c).to_screen(p);
    assert!(((after.x - before.x) - 0.5).abs() < 1e-3 && ((after.y - before.y) - 0.25).abs() < 1e-3, "{before:?} → {after:?}");
}
