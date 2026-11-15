use std::f64::consts::{FRAC_PI_2, PI};

use super::*;

const SINE_LENGTH: f64 = 7.640_395_578_055_424;

fn sine() -> FunctionAttrs {
    FunctionAttrs::new("sin(x)", (0.0, 2.0 * PI), (1.0, 1.0)).unwrap()
}

// --- Attributes ------------------------------------------------------------

#[test]
fn parse_error_keeps_its_position() {
    let Err(FunctionAttrsError::Parse(e)) = FunctionAttrs::new("2x", (0.0, 1.0), (1.0, 1.0)) else {
        panic!("2x must not parse");
    };
    assert_eq!(e.span, 1..2, "the error points at `x`");
}

#[test]
fn unproven_domains_are_rejected() {
    for (src, range) in [("1/x", (-1.0, 1.0)), ("sqrt(x)", (0.0, 1.0))] {
        let r = FunctionAttrs::new(src, range, (1.0, 1.0));
        assert!(matches!(r, Err(FunctionAttrsError::Domain(_))), "{src} on {range:?}: {r:?}");
    }
    assert!(FunctionAttrs::new("sqrt(x)", (1.0, 4.0), (1.0, 1.0)).is_ok());
}

#[test]
fn range_and_scale_are_checked() {
    for range in [(1.0, 0.0), (0.0, 0.0), (0.0, f64::INFINITY), (f64::NAN, 1.0)] {
        assert_eq!(FunctionAttrs::new("x", range, (1.0, 1.0)), Err(FunctionAttrsError::Range), "{range:?}");
    }
    for scale in [(1.0, 0.0), (-1.0, 1.0), (1.0, f64::NAN)] {
        assert_eq!(FunctionAttrs::new("x", (0.0, 1.0), scale), Err(FunctionAttrsError::Scale), "{scale:?}");
    }
}

#[test]
fn equality_ignores_the_table_and_clones_share_it() {
    let a = sine();
    let b = sine();
    assert_eq!(a, b);
    assert!(!a.shares_table(&b), "built twice");
    let c = a.clone();
    assert!(a.shares_table(&c));
    assert_ne!(a, FunctionAttrs::new("sin(x)", (0.0, PI), (1.0, 1.0)).unwrap());
    assert_ne!(a, FunctionAttrs::new("sin(x)", (0.0, 2.0 * PI), (1.0, 2.0)).unwrap());
    assert_ne!(a, FunctionAttrs::new("cos(x)", (0.0, 2.0 * PI), (1.0, 1.0)).unwrap());
}

// --- Arc-length table ------------------------------------------------------

#[test]
fn sine_period_has_its_length() {
    let a = sine();
    assert!((a.length() - SINE_LENGTH).abs() < 1e-9, "L = {}", a.length());
}

#[test]
fn scaled_parabola_matches_the_closed_form() {
    let a = FunctionAttrs::new("x^2", (0.0, 1.0), (2.0, 3.0)).unwrap();
    let exact = 10f64.sqrt() + 3f64.asinh() / 3.0;
    assert!((a.length() - exact).abs() < 1e-9, "L = {}, exact {exact}", a.length());
}

#[test]
fn inverse_is_the_arc_length_fraction() {
    for a in [sine(), FunctionAttrs::new("x^2", (0.0, 1.0), (2.0, 3.0)).unwrap(), FunctionAttrs::new("exp(x)", (0.0, 3.0), (1.0, 1.0)).unwrap()] {
        let l = a.length();
        for k in 0..=50 {
            let t = k as f64 / 50.0;
            let (x, _) = a.x_at_length(t * l);
            let s = a.arc_length(a.range().0, x);
            assert!((s - t * l).abs() <= 1e-9 * l, "{}: t = {t}, s = {s}, expected {}", a.source(), t * l);
        }
    }
}

#[test]
fn long_sine_stays_within_the_interval_budget() {
    let a = FunctionAttrs::new("sin(x)", (0.0, 100.0 * PI), (1.0, 1.0)).unwrap();
    // About 230 intervals per period at 1e-11·L with cubic Hermite inverses.
    assert!(a.table_intervals() < 16_000, "{} intervals", a.table_intervals());
}

/// A wildly oscillating graph is rejected within a bounded time: the
/// quadrature used to recurse forever below its rounding noise (review of
/// PR #102: `sin(100*x)` on [0, 1000] ran for minutes).
#[test]
fn oscillating_graph_is_rejected_quickly() {
    let start = std::time::Instant::now();
    let r = FunctionAttrs::new("sin(100*x)", (0.0, 1000.0), (1.0, 1.0));
    assert_eq!(r, Err(FunctionAttrsError::TooComplex));
    assert!(start.elapsed().as_secs() < 30, "took {:?}", start.elapsed());
}

/// A NaN curve parameter evaluates at the range start instead of panicking.
#[test]
fn nan_parameter_does_not_panic() {
    let a = sine();
    let c = (Point2::new(0.0, 0.0), HeadingAngle::new(0.0));
    assert_eq!(Function::at(f64::NAN, c, &a), Function::at(0.0, c, &a));
    assert!(Function::curvature(f64::NAN, c, &a).is_finite());
}

#[test]
fn default_attrs_are_the_unit_diagonal() {
    let a = FunctionAttrs::default();
    assert_eq!((a.source(), a.range(), a.scale()), ("x", (0.0, 1.0), (1.0, 1.0)));
    assert!((a.length() - 2f64.sqrt()).abs() < 1e-12);
    let _ = FRAC_PI_2;
}

// --- The kind --------------------------------------------------------------

use crate::document::Document;
use crate::geom::CurveParam;
use crate::geometry::{AnchorExprs, CurveExprs, EndpointExprs, GeometryId, GeometryRole, LineExprs};
use crate::vector::Vector;

fn add(doc: &mut Document, attrs: FunctionAttrs, o: (f64, f64), th: f64) -> GeometryId {
    doc.add_geometry_with::<Function>(
        GeometryRole::Result,
        (Point2::new(o.0, o.1), HeadingAngle::new(th)),
        KindAttrs::Function(attrs),
    )
}

fn close(a: Point2, b: Point2, tol: f64) -> bool {
    (a - b).length() < tol
}

#[test]
fn sine_over_one_period() {
    let mut doc = Document::new();
    let id = add(&mut doc, sine(), (0.0, 0.0), 0.0);
    let p = doc.committed_vector();
    assert!(close(Function::start_expr(id).evaluate(&doc, p), Point2::new(0.0, 0.0), 1e-15));
    assert!(close(Function::end_expr(id).evaluate(&doc, p), Point2::new(2.0 * PI, 0.0), 1e-12));
    let l = Function::length_expr(id).evaluate(&doc, p).value();
    assert!((l - SINE_LENGTH).abs() < 1e-9);
}

#[test]
fn frame_rotates_and_scales_stretch() {
    let mut doc = Document::new();
    let rotated = add(&mut doc, sine(), (0.0, 0.0), FRAC_PI_2);
    let parabola = FunctionAttrs::new("x^2", (0.0, 1.0), (2.0, 3.0)).unwrap();
    let scaled = add(&mut doc, parabola, (1.0, 1.0), 0.0);
    let p = doc.committed_vector();
    assert!(close(Function::end_expr(rotated).evaluate(&doc, p), Point2::new(0.0, 2.0 * PI), 1e-12));
    assert!(close(Function::end_expr(scaled).evaluate(&doc, p), Point2::new(3.0, 4.0), 1e-12));
}

#[test]
fn normalized_parameter_is_the_arc_length_fraction() {
    let mut doc = Document::new();
    let s = add(&mut doc, sine(), (0.0, 0.0), 0.0);
    let parabola = FunctionAttrs::new("x^2", (0.0, 1.0), (1.0, 1.0)).unwrap();
    let q = add(&mut doc, parabola.clone(), (0.0, 0.0), 0.0);
    let p = doc.committed_vector();
    let mid = Function::on_curve_expr(s, CurveParam::Normalized(0.5)).evaluate(&doc, p);
    assert!(close(mid, Point2::new(PI, 0.0), 1e-9), "{mid:?}");
    let at = Function::on_curve_expr(q, CurveParam::Normalized(0.3)).evaluate(&doc, p);
    let uni = Function::on_curve_expr(q, CurveParam::Uniform(0.3)).evaluate(&doc, p);
    assert_eq!(at, uni);
    assert!((at.y - at.x * at.x).abs() < 1e-15, "the sample lies on the graph");
    let len = parabola.arc_length(0.0, at.x);
    assert!((len - 0.3 * parabola.length()).abs() <= 1e-9 * parabola.length());
}

#[test]
fn origin_direction_and_no_radius() {
    let mut doc = Document::new();
    let id = add(&mut doc, sine(), (2.0, 1.0), 0.3);
    let p = doc.committed_vector();
    assert_eq!(Function::origin_expr(id).evaluate(&doc, p), Point2::new(2.0, 1.0));
    let d = Function::direction_expr(id).evaluate(&doc, p);
    assert!((d - Vec2::new(0.3f64.cos(), 0.3f64.sin())).length() < 1e-15);
    assert!(doc.radius_expr(id).is_none(), "no circular capability");
    assert!(doc.direction_expr(id).is_some() && doc.origin_expr(id).is_some());
}

#[test]
fn tangent_is_the_chart_derivative_and_normal_points_left() {
    let mut doc = Document::new();
    let id = add(&mut doc, sine(), (0.0, 0.0), 0.0);
    let p = doc.committed_vector();
    let t = 0.0;
    let v = Function::tangent_expr(id, CurveParam::Normalized(t)).evaluate(&doc, p);
    // γ′(0) = L · unit tangent of the graph at x = 0, i.e. L·(1, 1)/√2.
    let k = SINE_LENGTH / 2f64.sqrt();
    assert!((v - Vec2::new(k, k)).length() < 1e-6, "{v:?}");
    let n = Function::normal_expr(id, CurveParam::Normalized(t)).evaluate(&doc, p);
    assert!((n - v.left_normal()).length() < 1e-15);
    // Finite difference of the chart itself.
    let h = 1e-6;
    let fd = (Function::on_curve_expr(id, CurveParam::Normalized(0.3 + h)).evaluate(&doc, p)
        - Function::on_curve_expr(id, CurveParam::Normalized(0.3 - h)).evaluate(&doc, p))
        * (0.5 / h);
    let an = Function::tangent_expr(id, CurveParam::Normalized(0.3)).evaluate(&doc, p);
    assert!((fd - an).length() < 1e-5 * an.length(), "fd {fd:?} vs {an:?}");
}

/// Analytical Jacobian of `eval` (rows) over its parameter indices vs central differences.
fn check_fd(doc: &Document, p: &Vector, indices: &[usize], rows: usize, analytic: &[f64], eval: impl Fn(&Vector) -> Vec<f64>) {
    let n = indices.len();
    for (k, &idx) in indices.iter().enumerate() {
        let (mut pp, mut pm) = (p.clone(), p.clone());
        pp[idx] += 1e-6;
        pm[idx] -= 1e-6;
        let (a, b) = (eval(&pp), eval(&pm));
        for i in 0..rows {
            let fd = (a[i] - b[i]) / 2e-6;
            let an = analytic[i * n + k];
            assert!((fd - an).abs() < 1e-5 * (1.0 + an.abs()), "row {i}, column {idx}: {an} vs fd {fd}");
        }
    }
    let _ = doc;
}

#[test]
fn jacobians_match_finite_differences() {
    use crate::matrix::DenseMatrix;
    let mut doc = Document::new();
    let id = add(&mut doc, FunctionAttrs::new("x*sin(x)", (-1.0, 2.0), (1.5, 0.7)).unwrap(), (0.4, -0.2), 0.8);
    let p = doc.clone_vector();
    let points = [
        Function::on_curve_expr(id, CurveParam::Normalized(0.37)),
        Function::start_expr(id),
        Function::end_expr(id),
        Function::origin_expr(id),
    ];
    for e in points {
        let idx = e.param_indices(&doc);
        let mut buf = vec![0.0; 2 * idx.len()];
        e.accumulate_jacobian(&doc, &p, &mut DenseMatrix::packed(&mut buf, 2, idx.len()), 1.0);
        check_fd(&doc, &p, &idx, 2, &buf, |q| {
            let v = e.evaluate(&doc, q);
            vec![v.x, v.y]
        });
    }
    let vecs = [
        Function::tangent_expr(id, CurveParam::Normalized(0.61)),
        Function::normal_expr(id, CurveParam::Normalized(0.61)),
        Function::direction_expr(id),
    ];
    for e in vecs {
        let idx = e.param_indices(&doc);
        assert_eq!(idx.len(), 1, "vectors depend on θ only");
        let mut buf = vec![0.0; 2];
        e.accumulate_jacobian(&doc, &p, &mut DenseMatrix::packed(&mut buf, 2, 1), 1.0);
        check_fd(&doc, &p, &idx, 2, &buf, |q| {
            let v = e.evaluate(&doc, q);
            vec![v.x, v.y]
        });
    }
    assert_eq!(Function::length_expr(id).param_count(&doc), 0, "length depends on attributes only");
}

// --- Attribute forwarding at Document level (review of #87) ----------------

/// Everything a Document computes from a function curve at fixed State.
fn observe(doc: &Document, id: GeometryId) -> Vec<f64> {
    use crate::matrix::DenseMatrix;
    let p = doc.committed_vector();
    let mut out = Vec::new();
    let pt = Function::on_curve_expr(id, CurveParam::Normalized(0.4));
    let v = pt.evaluate(doc, p);
    out.extend([v.x, v.y]);
    let mut buf = vec![0.0; 6];
    pt.accumulate_jacobian(doc, p, &mut DenseMatrix::packed(&mut buf, 2, 3), 1.0);
    out.extend(&buf);
    let jet = Function::tangent_expr(id, CurveParam::Normalized(0.4)).jet(doc, p);
    let mut row = vec![0.0; 1];
    jet.contract_into(&mut DenseMatrix::packed(&mut row, 1, 1), 0, Vec2::new(1.0, 0.0), 1.0);
    out.extend([jet.value.x, jet.value.y, row[0]]);
    for piece in doc.curve_shape(id, p, 1e-6).unwrap() {
        out.extend([piece.end_point().x, piece.end_point().y]);
    }
    out.push(doc.curve_curvature(id, p, 0.4).unwrap());
    out
}

#[test]
fn attribute_changes_reach_every_document_query() {
    let base = FunctionAttrs::new("sin(x)", (0.0, 3.0), (1.0, 1.0)).unwrap();
    let variants = [
        FunctionAttrs::new("cos(x)", (0.0, 3.0), (1.0, 1.0)).unwrap(),
        FunctionAttrs::new("sin(x)", (0.0, 2.0), (1.0, 1.0)).unwrap(),
        FunctionAttrs::new("sin(x)", (0.0, 3.0), (1.0, 2.0)).unwrap(),
    ];
    for changed in variants {
        let mut doc = Document::new();
        let id = add(&mut doc, base.clone(), (0.5, 0.25), 0.4);
        let block = doc.committed_vector().as_slice().to_vec();
        let before = observe(&doc, id);
        assert!(doc.set_geometry_attrs(id, KindAttrs::Function(changed.clone())));
        let after = observe(&doc, id);
        assert_eq!(doc.committed_vector().as_slice(), &block[..], "canonical is the identity");
        // Point, Jacobian θ column, tangent jet value and row, shape, curvature.
        let differs = |r: std::ops::Range<usize>| r.into_iter().any(|i| (before[i] - after[i]).abs() > 1e-9);
        assert!(differs(0..2), "{}: evaluate", changed.source());
        assert!(differs(2..8), "{}: point Jacobian", changed.source());
        assert!(differs(8..11), "{}: vector jet", changed.source());
        assert!(before.len() != after.len() || differs(11..before.len() - 1), "{}: shape", changed.source());
        assert!((before[before.len() - 1] - after[after.len() - 1]).abs() > 1e-9, "{}: curvature", changed.source());
    }
}

// --- Piece bounds, shape and Mesh ------------------------------------------

#[test]
fn piece_bounds_are_upper_bounds() {
    for attrs in [sine(), FunctionAttrs::new("exp(x)", (0.0, 5.0), (1.0, 1.0)).unwrap()] {
        let c = (Point2::new(0.0, 0.0), HeadingAngle::new(0.0));
        let n = 64;
        let mut total = 0.0;
        for k in 0..n {
            let (t0, t1) = (k as f64 / n as f64, (k + 1) as f64 / n as f64);
            let b = Function::piece_bound(t0, t1, c, &attrs);
            total += b.length;
            for t in [t0, 0.5 * (t0 + t1), t1] {
                let kappa = Function::curvature(t, c, &attrs);
                assert!(b.curvature >= kappa, "{}: piece {k}, κ({t}) = {kappa} > bound {}", attrs.source(), b.curvature);
            }
        }
        assert!(total >= attrs.length(), "{}: Σ pieces {total} < L {}", attrs.source(), attrs.length());
        assert!(total <= attrs.length() * (1.0 + 1e-6), "{}: bounds stay tight", attrs.source());
    }
}

/// Distance from `q` to the unscaled graph of `attrs` (frame at the origin, θ = 0).
fn distance_to_graph(attrs: &FunctionAttrs, q: Point2, mut x: f64) -> f64 {
    let e = attrs.expression();
    let (x0, x1) = attrs.range();
    for _ in 0..40 {
        let j = e.eval(x).unwrap();
        let (dx, dy) = (x - q.x, j.value - q.y);
        let g = dx + dy * j.d1;
        let h = 1.0 + j.d1 * j.d1 + dy * j.d2;
        if h <= 0.0 {
            break;
        }
        x = (x - g / h).clamp(x0, x1);
    }
    let y = e.eval(x).unwrap().value;
    (Point2::new(x, y) - q).length()
}

/// Distance from `q` to the cubic piece `b`, by Newton projection from `u`.
fn distance_to_piece(piece: &ShapePiece, q: Point2) -> f64 {
    use crate::geometry::cubic;
    let ShapePiece::Cubic { start, c1, c2, end } = *piece else { panic!("cubic expected") };
    let b = [start, c1, c2, end];
    let mut best = f64::INFINITY;
    for u0 in [0.0, 0.25, 0.5, 0.75, 1.0] {
        let mut u: f64 = u0;
        for _ in 0..30 {
            let r = cubic::point(&b, u) - q;
            let (d1, d2) = (cubic::derivative(&b, u), cubic::second_derivative(&b, u));
            let h = d1.dot(d1) + r.dot(d2);
            if h <= 0.0 {
                break;
            }
            u = (u - r.dot(d1) / h).clamp(0.0, 1.0);
        }
        best = best.min((cubic::point(&b, u) - q).length());
    }
    best
}

#[test]
fn shape_stays_within_the_tolerance() {
    let delta = 1e-6;
    for attrs in [sine(), FunctionAttrs::new("exp(x)", (0.0, 2.0), (1.0, 1.0)).unwrap()] {
        let c = (Point2::new(0.0, 0.0), HeadingAngle::new(0.0));
        let shape = Function::shape(c, &attrs, delta);
        assert!(shape.iter().all(|s| matches!(s, ShapePiece::Cubic { .. })));
        let mut worst: f64 = 0.0;
        for piece in &shape {
            for k in 0..=32 {
                let q = piece.point_at(k as f64 / 32.0);
                worst = worst.max(distance_to_graph(&attrs, q, q.x));
            }
        }
        for k in 0..=2000 {
            let q = Function::at(k as f64 / 2000.0, c, &attrs);
            let d = shape.iter().map(|p| distance_to_piece(p, q)).fold(f64::INFINITY, f64::min);
            worst = worst.max(d);
        }
        assert!(worst <= delta, "{}: deviation {worst}", attrs.source());
        let len: f64 = shape.iter().map(ShapePiece::length).sum();
        assert!((len - attrs.length()).abs() <= delta, "{}: length {len} vs {}", attrs.source(), attrs.length());
    }
}

#[test]
fn mesh_of_a_steep_curve_stays_under_the_cap_and_within_tolerance() {
    use crate::mesh::{Mesh, MeshFilter, MAX_MESH_EDGES};
    let mut doc = Document::new();
    let attrs = FunctionAttrs::new("exp(x)", (0.0, 5.0), (1.0, 1.0)).unwrap();
    let id = add(&mut doc, attrs.clone(), (0.0, 0.0), 0.0);
    let tol = 1e-3;
    let mesh = Mesh::from_document(&doc, doc.committed_vector(), MeshFilter::RESULT_ONLY, tol);
    let edges: Vec<_> = mesh.edges.iter().filter(|e| e.geometry == id).collect();
    assert!(!edges.is_empty() && edges.len() < MAX_MESH_EDGES, "{} edges", edges.len());
    let seg_dist = |q: Point2, a: Point2, b: Point2| {
        let d = b - a;
        let s = ((q - a).dot(d) / d.dot(d)).clamp(0.0, 1.0);
        (a + d * s - q).length()
    };
    let c = (Point2::new(0.0, 0.0), HeadingAngle::new(0.0));
    for k in 0..=5000 {
        let q = Function::at(k as f64 / 5000.0, c, &attrs);
        let d = edges.iter().map(|e| seg_dist(q, e.a, e.b)).fold(f64::INFINITY, f64::min);
        assert!(d <= tol * (1.0 + 1e-9), "sample {k}: {d} from the polyline");
    }
}

/// Sharp peaks and interval overestimation (review of PR #102): the Mesh's
/// grid adapts, so neither inflates the demand to the cap, and every edge is
/// checked. Before, the bell curve kept 1024 unchecked edges 0.215 off the
/// graph, and the rational one fell back to equal unchecked edges.
#[test]
fn mesh_of_peaked_curves_stays_under_the_cap_and_within_tolerance() {
    use crate::mesh::{Mesh, MeshFilter, MAX_MESH_EDGES};
    let tol = 1e-3;
    for source in ["1/((x-1)^2 + 0.01)", "1/(x*x - 2*x + 1.01)", "1/((x-1)^2 + 0.0001)"] {
        let attrs = FunctionAttrs::new(source, (0.0, 2.0), (1.0, 1.0)).unwrap();
        let mut doc = Document::new();
        let id = add(&mut doc, attrs.clone(), (0.0, 0.0), 0.0);
        let mesh = Mesh::from_document(&doc, doc.committed_vector(), MeshFilter::RESULT_ONLY, tol);
        let edges: Vec<_> = mesh.edges.iter().filter(|e| e.geometry == id).collect();
        assert!(edges.len() < MAX_MESH_EDGES / 4, "{source}: {} edges", edges.len());
        // Every edge against the exact graph between its end points (heading
        // 0, scales 1: an edge's x range is the graph's x range).
        for e in &edges {
            let d = e.b - e.a;
            for k in 1..32 {
                let x = e.a.x + (e.b.x - e.a.x) * k as f64 / 32.0;
                let q = Point2::new(x, attrs.expression().eval(x).unwrap().value);
                let s = ((q - e.a).dot(d) / d.dot(d)).clamp(0.0, 1.0);
                let dist = (e.a + d * s - q).length();
                assert!(dist <= tol * (1.0 + 1e-9), "{source}: {dist} off the edge at x = {x}");
            }
        }
    }
}

#[test]
fn equal_length_with_a_function_curve_solves() {
    use crate::constraint_kinds::{EqualLength, FixPoint, VectorsParallel};
    use crate::geometry_kinds::Segment;
    use crate::interact::InteractiveSolve;
    let mut doc = Document::new();
    let f = add(&mut doc, sine(), (0.0, 0.0), 0.0);
    let s = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(0.0, -2.0), Point2::new(1.0, -2.0)));
    doc.add_constraint(FixPoint { point: Segment::start_expr(s), target: Point2::new(0.0, -2.0) });
    doc.add_constraint(VectorsParallel { a: Segment::direction_expr(s), b: Function::direction_expr(f) });
    doc.add_constraint(EqualLength { a: Function::length_expr(f), b: Segment::length_expr(s) });
    InteractiveSolve::new().structural_only(&mut doc).expect("solvable");
    let p = doc.committed_vector();
    let len = Segment::length_expr(s).evaluate(&doc, p).value();
    assert!((len - SINE_LENGTH).abs() < 1e-6, "segment length {len}");
}
