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
