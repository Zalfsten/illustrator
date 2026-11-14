use std::f64::consts::{FRAC_PI_2, FRAC_PI_8};

use super::*;

fn p(x: f64, y: f64) -> Point2 {
    Point2::new(x, y)
}

fn dist(a: Point2, b: Point2) -> f64 {
    (a - b).length()
}

/// Quarter circle of radius 1 around the origin, from heading 0 to π/2.
fn quarter() -> CubicPoints {
    let k = 4.0 / 3.0 * FRAC_PI_8.tan();
    [p(1.0, 0.0), p(1.0, k), p(k, 1.0), p(0.0, 1.0)]
}

fn polyline_length(b: &CubicPoints, n: usize) -> f64 {
    (0..n).map(|i| dist(point(b, i as f64 / n as f64), point(b, (i + 1) as f64 / n as f64))).sum()
}

#[test]
fn evenly_spaced_collinear_cubic_is_its_chord() {
    let b = [p(0.0, 0.0), p(1.0, 1.0), p(2.0, 2.0), p(3.0, 3.0)];
    let chord = 18f64.sqrt();
    assert!((arc_length(&b, 0.0, 1.0) - chord).abs() < 1e-13 * chord);
    for u in [0.1, 0.25, 0.5, 0.9] {
        assert!((param_at_length(&b, u * chord) - u).abs() < 1e-12, "u = {u}");
    }
}

#[test]
fn quarter_circle_length() {
    let b = quarter();
    let len = arc_length(&b, 0.0, 1.0);
    assert!((len - FRAC_PI_2).abs() < 3e-4 * FRAC_PI_2, "len = {len}");
    let reference = polyline_length(&b, 100_000);
    assert!((len - reference).abs() < 1e-9, "len = {len}, polyline = {reference}");
}

#[test]
fn split_halves_reproduce_the_curve() {
    let b = [p(0.0, 0.0), p(1.0, 3.0), p(4.0, -1.0), p(5.0, 2.0)];
    let u = 0.3;
    let (left, right) = split(&b, u);
    for k in 0..=10 {
        let s = k as f64 / 10.0;
        assert!(dist(point(&left, s), point(&b, u * s)) < 1e-13);
        assert!(dist(point(&right, s), point(&b, u + (1.0 - u) * s)) < 1e-13);
    }
    assert_eq!(left[3], right[0]);
}

#[test]
fn sub_reproduces_the_middle_part() {
    let b = [p(0.0, 0.0), p(1.0, 3.0), p(4.0, -1.0), p(5.0, 2.0)];
    let (u0, u1) = (0.2, 0.7);
    let m = sub(&b, u0, u1);
    for k in 0..=10 {
        let s = k as f64 / 10.0;
        assert!(dist(point(&m, s), point(&b, u0 + (u1 - u0) * s)) < 1e-13);
    }
    assert_eq!(sub(&b, 0.0, 1.0), b);
}

#[test]
fn extrema_of_the_arch() {
    let b = [p(0.0, 0.0), p(1.0, 2.0), p(2.0, 2.0), p(3.0, 0.0)];
    let ext = axis_extrema(&b);
    assert_eq!(ext.len(), 1);
    assert!((ext[0] - 0.5).abs() < 1e-15);
    assert!((point(&b, ext[0]).y - 1.5).abs() < 1e-15);
}

#[test]
fn param_at_length_inverts_length() {
    // Speed 0 at the start and fast at the end: strongly non-uniform.
    let b = [p(0.0, 0.0), p(0.0, 0.0), p(1.0, 0.0), p(10.0, 5.0)];
    let total = arc_length(&b, 0.0, 1.0);
    for u in [0.05, 0.3, 0.5, 0.8, 0.99] {
        let len = arc_length(&b, 0.0, u);
        assert!((param_at_length(&b, len) - u).abs() < 1e-10, "u = {u}");
    }
    for f in [0.1, 0.5, 0.9] {
        let u = param_at_length(&b, f * total);
        assert!((arc_length(&b, 0.0, u) - f * total).abs() < 1e-12 * total, "f = {f}");
    }
    assert_eq!(param_at_length(&b, -1.0), 0.0);
    assert_eq!(param_at_length(&b, 2.0 * total), 1.0);
}

#[test]
fn curvature_of_the_quarter_circle_is_about_one() {
    // Reference values computed independently from the closed-form
    // derivatives: 0.97855 at the ends, 0.99376 in the middle.
    let b = quarter();
    assert!((curvature(&b, 0.0) - 0.978_553_390_593_274).abs() < 1e-12);
    assert!((curvature(&b, 1.0) - 0.978_553_390_593_274).abs() < 1e-12);
    assert!((curvature(&b, 0.5) - 0.993_757_645_304_129).abs() < 1e-12);
    let mirrored = [b[3], b[2], b[1], b[0]];
    assert!(curvature(&mirrored, 0.5) < 0.0, "clockwise turns right");
}

// --- Hermite builder -------------------------------------------------------

use std::f64::consts::PI;

fn cubic_of(piece: &ShapePiece) -> CubicPoints {
    let ShapePiece::Cubic { start, c1, c2, end } = *piece else {
        panic!("expected a cubic piece, got {piece:?}");
    };
    [start, c1, c2, end]
}

/// Distance from `q` to the cubic `b`, by Newton projection from `u`.
fn distance_to_cubic(b: &CubicPoints, q: Point2, mut u: f64) -> f64 {
    for _ in 0..30 {
        let r = point(b, u) - q;
        let (d1, d2) = (derivative(b, u), second_derivative(b, u));
        let g = r.dot(d1);
        let h = d1.dot(d1) + r.dot(d2);
        if h <= 0.0 {
            break;
        }
        u = (u - g / h).clamp(0.0, 1.0);
    }
    (point(b, u) - q).length()
}

/// Two-sided distance check of `shape` against the curve `gamma` on
/// `[0, 1]`: shape → curve with `dist_to_curve`, curve → shape by projection
/// onto the piece covering each curve sample's parameter.
fn max_two_sided(
    shape: &[ShapePiece],
    knots: &[(f64, f64)],
    gamma: impl Fn(f64) -> Point2,
    dist_to_curve: impl Fn(Point2, f64) -> f64,
) -> f64 {
    let mut worst: f64 = 0.0;
    for (piece, &(t0, t1)) in shape.iter().zip(knots) {
        let b = cubic_of(piece);
        for k in 0..=64 {
            let u = k as f64 / 64.0;
            let t = t0 + u * (t1 - t0);
            worst = worst.max(dist_to_curve(point(&b, u), t));
            worst = worst.max(distance_to_cubic(&b, gamma(t), u));
        }
    }
    worst
}

/// Parameter intervals of the pieces, recovered from their joints.
fn intervals(shape: &[ShapePiece], gamma: impl Fn(f64) -> Point2) -> Vec<(f64, f64)> {
    // Joints are exact curve points; find each joint's parameter by scanning.
    let mut out = Vec::new();
    let mut t0 = 0.0;
    for piece in shape {
        let end = piece.end_point();
        let mut t1 = 1.0;
        let mut best = f64::INFINITY;
        for k in 0..=1 << 16 {
            let t = k as f64 / f64::from(1u32 << 16);
            let d = (gamma(t) - end).length();
            if t > t0 && d < best {
                best = d;
                t1 = t;
            }
        }
        out.push((t0, t1));
        t0 = t1;
    }
    out
}

fn circle(r: f64, turns: f64) -> (impl Fn(f64) -> Point2, impl Fn(f64) -> Vec2) {
    let w = 2.0 * PI * turns;
    (
        move |t: f64| p(r * (w * t).cos(), r * (w * t).sin()),
        move |t: f64| Vec2::new(-r * w * (w * t).sin(), r * w * (w * t).cos()),
    )
}

#[test]
fn hermite_circle_stays_within_the_tolerance() {
    let r = 2.0;
    for turns in [0.25, 1.0] {
        let (g, dg) = circle(r, turns);
        for rel in [1e-3, 1e-6, 1e-9] {
            let delta = rel * r;
            let shape = hermite_shape(&g, &dg, &[0.0, 1.0], delta, sampled_deviation(&g));
            assert!(shape.iter().all(|s| matches!(s, ShapePiece::Cubic { .. })));
            let iv = intervals(&shape, &g);
            let worst = max_two_sided(&shape, &iv, &g, |q, _| ((q - p(0.0, 0.0)).length() - r).abs());
            assert!(worst <= delta, "turns {turns}, δ {delta}: deviation {worst}");
            let len: f64 = shape.iter().map(ShapePiece::length).sum();
            let exact = 2.0 * PI * r * turns;
            assert!((len - exact).abs() <= delta, "turns {turns}, δ {delta}: length {len} vs {exact}");
        }
    }
}

#[test]
fn hermite_sine_graph_stays_within_the_tolerance() {
    let w = 4.0 * PI;
    let g = move |t: f64| p(w * t, (w * t).sin());
    let dg = move |t: f64| Vec2::new(w, w * (w * t).cos());
    let ddg = move |t: f64| Vec2::new(0.0, -w * w * (w * t).sin());
    // Distance from q to the graph by Newton projection from parameter t.
    let to_graph = move |q: Point2, mut t: f64| {
        for _ in 0..30 {
            let r = g(t) - q;
            let h = dg(t).dot(dg(t)) + r.dot(ddg(t));
            if h <= 0.0 {
                break;
            }
            t = (t - r.dot(dg(t)) / h).clamp(0.0, 1.0);
        }
        (g(t) - q).length()
    };
    for delta in [1e-3, 1e-6, 1e-9] {
        let shape = hermite_shape(g, dg, &[0.0, 1.0], delta, sampled_deviation(g));
        let iv = intervals(&shape, g);
        let worst = max_two_sided(&shape, &iv, g, to_graph);
        assert!(worst <= delta, "δ {delta}: deviation {worst}");
    }
}

#[test]
fn hermite_ends_and_joints_lie_on_the_curve() {
    let (g, dg) = circle(1.0, 0.75);
    let knots = [0.0, 0.3, 1.0];
    let shape = hermite_shape(&g, &dg, &knots, 1e-6, sampled_deviation(&g));
    assert_eq!(shape[0].start_point(), g(0.0));
    assert_eq!(shape.last().unwrap().end_point(), g(1.0));
    for w in shape.windows(2) {
        assert_eq!(w[0].end_point(), w[1].start_point(), "joints are shared exactly");
    }
    assert!(shape.iter().any(|s| s.end_point() == g(0.3)), "a knot is a joint");
}

#[test]
fn hermite_uses_the_given_deviation() {
    let (g, dg) = circle(1.0, 1.0);
    let calls = std::cell::Cell::new(0);
    let zero = |_: f64, _: f64, _: &ShapePiece| {
        calls.set(calls.get() + 1);
        0.0
    };
    let shape = hermite_shape(&g, &dg, &[0.0, 0.5, 1.0], 1e-9, zero);
    assert_eq!(shape.len(), 8, "4 initial pieces per knot interval, none halved");
    assert_eq!(calls.get(), 8);
}

#[test]
fn hermite_stops_at_the_piece_cap() {
    let (g, dg) = circle(1.0, 1.0);
    let shape = hermite_shape(&g, &dg, &[0.0, 1.0], 1.0, |_, _, _| f64::INFINITY);
    assert_eq!(shape.len(), MAX_SHAPE_PIECES);
}

#[test]
fn hermite_is_deterministic() {
    let w = 4.0 * PI;
    let g = move |t: f64| p(w * t, (w * t).sin());
    let dg = move |t: f64| Vec2::new(w, w * (w * t).cos());
    let a = hermite_shape(g, dg, &[0.0, 1.0], 1e-7, sampled_deviation(g));
    let b = hermite_shape(g, dg, &[0.0, 1.0], 1e-7, sampled_deviation(g));
    assert_eq!(a, b);
}
