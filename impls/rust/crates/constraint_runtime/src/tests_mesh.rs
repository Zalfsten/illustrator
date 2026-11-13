use super::*;
use crate::document::Document;
use crate::geom::{HeadingAngle, Length, Point2, TurnAngle};
use crate::geometry_kinds::{Arc, Segment};
use crate::geometry::{Caps, CurveExprs, EndpointExprs, GeometryCaps, GeometryId, GeometryRole};

/// Chord tolerance (world units) for tests that do not probe it.
const EPS: f64 = 0.01;

#[test]
fn one_segment_becomes_one_edge() {
    let mut doc = Document::new();
    let id = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(0.0, 0.0), Point2::new(3.0, 4.0)));
    let mesh = Mesh::from_document(&doc, doc.vector(), MeshFilter::RESULT_ONLY, EPS);
    assert_eq!(mesh.edges.len(), 1);
    let e = &mesh.edges[0];
    assert_eq!(e.geometry, id);
    assert_eq!(e.role, GeometryRole::Result);
    assert_eq!(e.a, Point2::new(0.0, 0.0));
    assert_eq!(e.b, Point2::new(3.0, 4.0));
}

#[test]
fn hit_endpoint_near_start_end_and_miss() {
    let mut doc = Document::new();
    let id = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(0.0, 0.0), Point2::new(10.0, 0.0)));
    let mesh = Mesh::from_document(&doc, doc.vector(), MeshFilter::RESULT_ONLY, EPS);

    let hit_s = mesh.hit_endpoint(Point2::new(0.1, 0.0), 1.0);
    assert_eq!(hit_s, Some(Segment::start_expr(id)));

    let hit_e = mesh.hit_endpoint(Point2::new(9.9, 0.0), 1.0);
    assert_eq!(hit_e, Some(Segment::end_expr(id)));

    assert_eq!(mesh.hit_endpoint(Point2::new(5.0, 5.0), 1.0), None);
}

#[test]
fn hit_geometry_near_midpoint_and_miss() {
    let mut doc = Document::new();
    let id = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(0.0, 0.0), Point2::new(10.0, 0.0)));
    let mesh = Mesh::from_document(&doc, doc.vector(), MeshFilter::RESULT_ONLY, EPS);

    assert_eq!(
        mesh.hit_geometry(Point2::new(5.0, 0.5), 1.0),
        Some(id)
    );
    assert_eq!(mesh.hit_geometry(Point2::new(5.0, 5.0), 1.0), None);
}

#[test]
fn mesh_tracks_state_change() {
    let mut doc = Document::new();
    let id = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)));
    doc.vector_mut()[2] = 5.0;
    doc.vector_mut()[3] = 0.0;
    let mesh = Mesh::from_document(&doc, doc.vector(), MeshFilter::RESULT_ONLY, EPS);
    assert_eq!(mesh.edges[0].geometry, id);
    assert_eq!(mesh.edges[0].b, Point2::new(5.0, 0.0));
}

#[test]
fn construction_omitted_from_result_only_mesh() {
    let mut doc = Document::new();
    let result = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)));
    let construction = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(0.0, 1.0), Point2::new(1.0, 1.0)));
    assert!(doc.set_geometry_role(construction, GeometryRole::Construction));

    let result_only =
        Mesh::from_document(&doc, doc.vector(), MeshFilter::RESULT_ONLY, EPS);
    assert_eq!(result_only.edges.len(), 1);
    assert_eq!(result_only.edges[0].geometry, result);
    assert!(result_only
        .edges
        .iter()
        .all(|e| e.geometry != construction));

    let included =
        Mesh::from_document(&doc, doc.vector(), MeshFilter::INCLUDE_CONSTRUCTION, EPS);
    assert_eq!(included.edges.len(), 2);
    assert!(included.edges.iter().any(|e| e.geometry == construction));
    assert!(included
        .edges
        .iter()
        .any(|e| e.geometry == construction && e.role == GeometryRole::Construction));
}

#[test]
fn arc_tessellates_to_multiple_edges() {
    let mut doc = Document::new();
    let id = doc.add_geometry::<Arc>(
        GeometryRole::Result,
        (
            Point2::new(0.0, 0.0),
            Length::new(1.0),
            HeadingAngle::new(0.0),
            TurnAngle::new(std::f64::consts::FRAC_PI_2),
        ),
    );
    let mesh = Mesh::from_document(&doc, doc.vector(), MeshFilter::RESULT_ONLY, EPS);
    assert!(mesh.edges.len() > 1);
    assert!(mesh.edges.iter().all(|e| e.geometry == id));
    let start = Arc::start_expr(id).evaluate(&doc, doc.vector());
    let end = Arc::end_expr(id).evaluate(&doc, doc.vector());
    assert_eq!(mesh.edges[0].a, start);
    assert_eq!(mesh.edges.last().unwrap().b, end);
    assert_eq!(
        mesh.hit_endpoint(Point2::new(1.0, 0.05), 0.2),
        Some(Arc::start_expr(id))
    );
    assert_eq!(
        mesh.hit_endpoint(Point2::new(0.05, 1.0), 0.2),
        Some(Arc::end_expr(id))
    );
}

#[test]
fn zero_curvature_is_one_edge_high_curvature_is_denser() {
    let mut doc = Document::new();
    let seg = doc.add_geometry::<Segment>(
        GeometryRole::Result,
        (Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)),
    );
    let tight = doc.add_geometry::<Arc>(
        GeometryRole::Result,
        (
            Point2::new(0.0, 0.0),
            Length::new(0.25),
            HeadingAngle::new(0.0),
            TurnAngle::new(std::f64::consts::PI),
        ),
    );
    let mesh = Mesh::from_document(&doc, doc.vector(), MeshFilter::RESULT_ONLY, EPS);
    assert_eq!(
        mesh.edges.iter().filter(|e| e.geometry == seg).count(),
        1
    );
    let n_tight = mesh.edges.iter().filter(|e| e.geometry == tight).count();
    assert!(n_tight >= 4, "semicircle should subdivide, got {n_tight}");
}

#[test]
fn segment_and_arc_mesh_side_by_side() {
    let mut doc = Document::new();
    let seg = doc.add_geometry::<Segment>(
        GeometryRole::Result,
        (Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)),
    );
    let arc = doc.add_geometry::<Arc>(
        GeometryRole::Result,
        (
            Point2::new(2.0, 0.0),
            Length::new(1.0),
            HeadingAngle::new(0.0),
            TurnAngle::new(std::f64::consts::PI),
        ),
    );
    let mesh = Mesh::from_document(&doc, doc.vector(), MeshFilter::RESULT_ONLY, EPS);
    let seg_edges: Vec<_> = mesh.edges.iter().filter(|e| e.geometry == seg).collect();
    let arc_edges: Vec<_> = mesh.edges.iter().filter(|e| e.geometry == arc).collect();
    assert_eq!(seg_edges.len(), 1);
    assert!(arc_edges.len() > 1);
}

#[test]
fn result_only_hits_ignore_construction() {
    let mut doc = Document::new();
    let _result = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(0.0, 0.0), Point2::new(10.0, 0.0)));
    let construction = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(0.0, 5.0), Point2::new(10.0, 5.0)));
    assert!(doc.set_geometry_role(construction, GeometryRole::Construction));

    let mesh = Mesh::from_document(&doc, doc.vector(), MeshFilter::RESULT_ONLY, EPS);
    assert_eq!(mesh.hit_endpoint(Point2::new(0.0, 5.0), 1.0), None);
    assert_eq!(mesh.hit_geometry(Point2::new(5.0, 5.0), 1.0), None);

    let full =
        Mesh::from_document(&doc, doc.vector(), MeshFilter::INCLUDE_CONSTRUCTION, EPS);
    assert_eq!(
        full.hit_endpoint(Point2::new(0.0, 5.0), 1.0),
        Some(Segment::start_expr(construction))
    );
    assert_eq!(
        full.hit_geometry(Point2::new(5.0, 5.0), 1.0),
        Some(construction)
    );
}

/// Mesh of one arc built as if its kind lacked the endpoints capability.
fn arc_mesh_without_endpoints() -> (Document, GeometryId, Mesh) {
    let mut doc = Document::new();
    let id = doc.add_geometry::<Arc>(
        GeometryRole::Result,
        (
            Point2::new(1.0, 1.0),
            Length::new(2.0),
            HeadingAngle::new(0.0),
            TurnAngle::new(std::f64::consts::PI),
        ),
    );
    let caps = Caps {
        endpoints: false,
        ..<Arc as GeometryCaps>::CAPS
    };
    let mut mesh = Mesh::default();
    append_curve_mesh(
        caps,
        &doc,
        doc.vector(),
        EPS,
        id,
        GeometryRole::Result,
        &mut mesh,
    );
    (doc, id, mesh)
}

#[test]
fn curve_without_endpoints_is_tessellated_from_on_curve_samples() {
    let (doc, id, mesh) = arc_mesh_without_endpoints();
    assert!(mesh.edges.len() > 1);
    let start = Arc::on_curve_expr(id, CurveParam::Normalized(0.0)).evaluate(&doc, doc.vector());
    let end = Arc::on_curve_expr(id, CurveParam::Normalized(1.0)).evaluate(&doc, doc.vector());
    assert_eq!(mesh.edges[0].a, start);
    assert_eq!(mesh.edges.last().unwrap().b, end);
    for e in &mesh.edges {
        assert_eq!(e.geometry, id);
        for v in [e.a, e.b] {
            let r = (v.x - 1.0).hypot(v.y - 1.0);
            assert!((r - 2.0).abs() < 1e-12, "vertex {v:?} not on the arc");
        }
    }
}

#[test]
fn curve_without_endpoints_yields_no_endpoint_hit() {
    let (_, id, mesh) = arc_mesh_without_endpoints();
    let t0 = mesh.edges[0].a;
    assert_eq!(mesh.endpoint_exprs().count(), 0);
    assert_eq!(mesh.hit_endpoint(t0, 0.5), None);
    assert_eq!(mesh.hit_geometry(t0, 0.5), Some(id));
}

#[test]
fn geometry_without_curve_capability_contributes_nothing() {
    let mut doc = Document::new();
    let id = doc.add_geometry::<Segment>(
        GeometryRole::Result,
        (Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)),
    );
    let caps = Caps {
        curve: false,
        ..<Segment as GeometryCaps>::CAPS
    };
    let mut mesh = Mesh::default();
    append_curve_mesh(
        caps,
        &doc,
        doc.vector(),
        EPS,
        id,
        GeometryRole::Result,
        &mut mesh,
    );
    assert!(mesh.edges.is_empty());
    assert_eq!(mesh.endpoint_exprs().count(), 0);
}

// --- place_samples (synthetic curves) ---

/// Largest turn per edge on a circle of curvature `k` within `eps`.
fn phi_max_acos(k: f64, eps: f64) -> f64 {
    2.0 * (1.0 - eps * k).clamp(-1.0, 1.0).acos()
}

type Kappa = Box<dyn Fn(f64) -> f64>;
type KappaBound = Box<dyn Fn(f64, f64) -> f64>;

/// A test curve in arc length `s ∈ [0, length]` (normalized `t = s / length`),
/// defined by its signed curvature, with an exact bound of |κ| over a piece
/// and positions from a fine cumulative table.
struct Synth {
    length: f64,
    bound: KappaBound,
    table: Vec<Point2>,
}

impl Synth {
    const STEPS: usize = 200_000;

    fn new(length: f64, kappa: Kappa, bound: KappaBound) -> Self {
        let h = length / Self::STEPS as f64;
        let mut theta = 0.0;
        let mut table = Vec::with_capacity(Self::STEPS + 1);
        let mut p = Point2::new(0.0, 0.0);
        table.push(p);
        for i in 0..Self::STEPS {
            let s = i as f64 * h;
            let mid = theta + 0.5 * h * kappa(s + 0.25 * h);
            p = Point2::new(p.x + h * mid.cos(), p.y + h * mid.sin());
            theta += h * kappa(s + 0.5 * h);
            table.push(p);
        }
        Self { length, bound, table }
    }

    /// Curve made of consecutive pieces `(length, curvature)`.
    fn piecewise(parts: &[(f64, f64)]) -> Self {
        let parts: Vec<(f64, f64, f64)> = parts
            .iter()
            .scan(0.0, |s, &(l, k)| {
                let start = *s;
                *s += l;
                Some((start, start + l, k))
            })
            .collect();
        let total = parts.last().unwrap().1;
        let p1 = parts.clone();
        let kappa = move |s: f64| p1.iter().find(|&&(a, b, _)| a <= s && s < b).map_or(0.0, |p| p.2);
        let bound = move |s0: f64, s1: f64| {
            parts
                .iter()
                .filter(|&&(a, b, _)| a <= s1 && s0 <= b)
                .map(|p| p.2.abs())
                .fold(0.0, f64::max)
        };
        Self::new(total, Box::new(kappa), Box::new(bound))
    }

    fn piece(&self, t0: f64, t1: f64) -> PieceBound {
        PieceBound {
            length: (t1 - t0) * self.length,
            curvature: (self.bound)(t0 * self.length, t1 * self.length),
        }
    }

    fn at(&self, t: f64) -> Point2 {
        let x = t.clamp(0.0, 1.0) * Self::STEPS as f64;
        let i = (x.floor() as usize).min(Self::STEPS - 1);
        let f = x - i as f64;
        let (a, b) = (self.table[i], self.table[i + 1]);
        Point2::new(a.x + f * (b.x - a.x), a.y + f * (b.y - a.y))
    }

    fn samples(&self, eps: f64, cap: usize) -> Vec<f64> {
        place_samples(|a, b| self.piece(a, b), eps, cap)
    }

    /// The sampler's demand integral ΣDᵢ over its 64 cells.
    fn demand(&self, eps: f64) -> f64 {
        (0..64)
            .map(|i| edge_demand(self.piece(i as f64 / 64.0, (i + 1) as f64 / 64.0), eps))
            .sum()
    }

    fn assert_within(&self, ts: &[f64], eps: f64, name: &str) {
        for w in ts.windows(2) {
            let (a, b) = (self.at(w[0]), self.at(w[1]));
            for k in 1..20 {
                let q = self.at(w[0] + (w[1] - w[0]) * k as f64 / 20.0);
                let d = distance_point_to_segment(q, a, b);
                assert!(d <= eps * (1.0 + 1e-6) + 1e-9, "{name}: deviation {d} on [{}, {}]", w[0], w[1]);
            }
        }
    }
}

#[test]
fn green_bound_of_constant_curvature_is_the_small_angle_sagitta() {
    let pieces = [PieceBound { length: 0.5, curvature: 2.0 }; 4];
    let l: f64 = 2.0;
    assert!((deviation_bound(&pieces) - 2.0 * l * l / 8.0).abs() < 1e-12);
    assert!((deviation_at(&pieces, 2.0, 1.0) - 1.0).abs() < 1e-12);
}

#[test]
fn green_bound_of_curvature_near_an_end_is_small() {
    let pieces = [
        PieceBound { length: 0.99, curvature: 0.0 },
        PieceBound { length: 0.01, curvature: 3.0 },
    ];
    assert!(deviation_bound(&pieces) < 0.02 * 3.0 / 8.0);
}

#[test]
fn green_bound_is_concave_and_matches_quadrature() {
    let pieces = [
        PieceBound { length: 0.3, curvature: 1.0 },
        PieceBound { length: 0.5, curvature: 0.2 },
        PieceBound { length: 0.4, curvature: 3.0 },
    ];
    let total = 1.2;
    let b = |s: f64| deviation_at(&pieces, total, s);
    for k in 1..119 {
        let (s0, s1, s2) = ((k - 1) as f64 / 100.0, k as f64 / 100.0, (k + 1) as f64 / 100.0);
        assert!(b(s0) - 2.0 * b(s1) + b(s2) <= 1e-12, "not concave at {s1}");
    }
    // Midpoint quadrature of ∫ G(s, u) κ(u) du on a grid (h = 1e-5) that has
    // the curvature jumps and every tested s on cell boundaries.
    let kappa = |u: f64| if u < 0.3 { 1.0 } else if u < 0.8 { 0.2 } else { 3.0 };
    for s in [0.1, 0.45, 0.9] {
        let n = 120_000;
        let h = total / n as f64;
        let q: f64 = (0..n)
            .map(|i| {
                let u = (i as f64 + 0.5) * h;
                let g = if u <= s { u * (total - s) / total } else { s * (total - u) / total };
                g * kappa(u) * h
            })
            .sum();
        assert!((b(s) - q).abs() < 1e-9, "s = {s}: {} vs {q}", b(s));
    }
}

/// Deterministic pseudo-random pieces (xorshift), some of them straight.
fn random_pieces(state: &mut u64) -> Vec<PieceBound> {
    let mut next = || {
        *state ^= *state << 13;
        *state ^= *state >> 7;
        *state ^= *state << 17;
        (*state >> 11) as f64 / (1u64 << 53) as f64
    };
    (0..4)
        .map(|_| PieceBound {
            length: 0.01 + next(),
            curvature: if next() < 0.3 { 0.0 } else { 10.0 * next() },
        })
        .collect()
}

#[test]
fn green_bound_is_the_exact_maximum() {
    let mut state = 0x9E37_79B9_7F4A_7C15;
    for _ in 0..500 {
        let pieces = random_pieces(&mut state);
        let total: f64 = pieces.iter().map(|b| b.length).sum();
        let bound = deviation_bound(&pieces);
        let sampled = (0..=4000)
            .map(|k| deviation_at(&pieces, total, total * k as f64 / 4000.0))
            .fold(0.0, f64::max);
        assert!(bound >= sampled * (1.0 - 1e-12), "{bound} < sampled {sampled}");
        assert!(bound <= sampled * (1.0 + 1e-5), "{bound} far above sampled {sampled}");
    }
}

/// Length upper bounds keep the check sound: lengthening a piece never lowers
/// the bound.
#[test]
fn green_bound_grows_with_piece_lengths() {
    let mut state = 0x2545_F491_4F6C_DD1D;
    for _ in 0..2000 {
        let pieces = random_pieces(&mut state);
        let before = deviation_bound(&pieces);
        for i in 0..pieces.len() {
            let mut longer = pieces.clone();
            longer[i].length *= 1.37;
            let after = deviation_bound(&longer);
            assert!(after >= before * (1.0 - 1e-12), "piece {i}: {after} < {before}");
        }
    }
}

#[test]
fn over_estimated_lengths_stay_within_tolerance() {
    use std::f64::consts::PI;
    let curve = Synth::piecewise(&[(PI, 0.0), (PI, 1.0)]);
    let eps = 1e-3;
    let padded = |a: f64, b: f64| {
        let p = curve.piece(a, b);
        PieceBound { length: p.length * 1.05, ..p }
    };
    let ts = place_samples(padded, eps, MAX_MESH_EDGES);
    curve.assert_within(&ts, eps, "lengths over-estimated by 5 %");
    assert!(ts.len() - 1 < MAX_MESH_EDGES / 4);
}

#[test]
fn zero_curvature_places_one_edge() {
    let piece = |a: f64, b: f64| PieceBound { length: 3.0 * (b - a), curvature: 0.0 };
    assert_eq!(place_samples(piece, 1e-3, 1024), vec![0.0, 1.0]);
}

#[test]
fn constant_curvature_places_equal_shares_without_halving() {
    let (r, sweep, eps) = (2.0, 4.0, 0.01);
    let piece = |a: f64, b: f64| PieceBound { length: r * sweep * (b - a), curvature: 1.0 / r };
    let ts = place_samples(piece, eps, 1024);
    let n = ts.len() - 1;
    let demand = r * sweep * (1.0 / (8.0 * eps * r)).sqrt();
    assert_eq!(n, demand.ceil() as usize);
    for (j, t) in ts.iter().enumerate() {
        assert!((t - j as f64 / n as f64).abs() < 1e-12, "t_{j} = {t}");
    }
}

#[test]
fn sine_like_wiggle_stays_within_tolerance() {
    use std::f64::consts::{FRAC_PI_2, TAU};
    // Curvature K sin(s): inflection points, equal end tangents over a period.
    let k = 1.5;
    let bound = move |s0: f64, s1: f64| {
        let peak = s1 - s0 >= std::f64::consts::PI
            || (0..4).any(|m| {
                let c = FRAC_PI_2 + f64::from(m) * std::f64::consts::PI;
                s0 <= c && c <= s1
            });
        if peak { k } else { k * s0.sin().abs().max(s1.sin().abs()) }
    };
    let curve = Synth::new(TAU, Box::new(move |s: f64| k * s.sin()), Box::new(bound));
    let eps = 0.01;
    let ts = curve.samples(eps, 1024);
    assert!(ts.len() > 2, "not a single chord");
    assert!(((ts.len() - 1) as f64) <= 2.0 * curve.demand(eps) + 12.0, "{} edges", ts.len() - 1);
    curve.assert_within(&ts, eps, "wiggle");
}

#[test]
fn curvature_spike_gets_denser_samples() {
    let kappa = |s: f64| 1.0 + 1000.0 * (-((s - 0.5) / 0.03).powi(2)).exp();
    let bound = move |s0: f64, s1: f64| kappa(0.5_f64.clamp(s0, s1));
    let curve = Synth::new(1.0, Box::new(kappa), Box::new(bound));
    let ts = curve.samples(1e-3, 1024);
    let edge_at = |t: f64| ts.windows(2).find(|w| w[0] <= t && t <= w[1]).map(|w| w[1] - w[0]).unwrap();
    assert!(edge_at(0.5) * 5.0 < edge_at(0.1), "{} vs {}", edge_at(0.5), edge_at(0.1));
    curve.assert_within(&ts, 1e-3, "gaussian spike");
}

#[test]
fn cap_bounds_the_edge_count() {
    let piece = |a: f64, b: f64| PieceBound { length: b - a, curvature: 1e6 };
    let ts = place_samples(piece, 1e-6, 8);
    assert_eq!(ts.len(), 9);
    assert_eq!((ts[0], ts[8]), (0.0, 1.0));
}

#[test]
fn zero_length_with_infinite_curvature_is_one_edge() {
    let piece = |_: f64, _: f64| PieceBound { length: 0.0, curvature: f64::INFINITY };
    assert_eq!(place_samples(piece, 1e-3, 1024), vec![0.0, 1.0]);
}

// --- #86: straight stretches, ramps and spikes stay cheap and within tolerance ---

fn assert_cheap_and_within(curve: &Synth, eps: f64, name: &str) -> Vec<f64> {
    let ts = curve.samples(eps, MAX_MESH_EDGES);
    let n = ts.len() - 1;
    let limit = 2.0 * curve.demand(eps) + 12.0;
    assert!((n as f64) <= limit, "{name}: {n} edges, limit {limit}");
    curve.assert_within(&ts, eps, name);
    ts
}

#[test]
fn line_then_circle_cases_from_86() {
    use std::f64::consts::{FRAC_PI_2, PI};
    let eps = 1e-3;
    assert_cheap_and_within(&Synth::piecewise(&[(PI, 0.0), (PI, 1.0)]), eps, "half line, half circle");
    for line in [2.0, 10.0, 50.0] {
        let name = format!("line {line}, quarter circle");
        assert_cheap_and_within(&Synth::piecewise(&[(line, 0.0), (FRAC_PI_2, 1.0)]), eps, &name);
    }
    assert_cheap_and_within(&Synth::piecewise(&[(5.0, 1e-4), (PI, 1.0)]), eps, "nearly straight, circle");
}

#[test]
fn curvature_ramp_from_86() {
    // κ rises linearly from 0 to 1 over s ∈ [0, 2], then stays 1 until s = 4.
    let kappa = |s: f64| (s / 2.0).min(1.0);
    let bound = move |_: f64, s1: f64| kappa(s1);
    let curve = Synth::new(4.0, Box::new(kappa), Box::new(bound));
    assert_cheap_and_within(&curve, 1e-3, "ramp");
}

#[test]
fn straight_stretch_beside_a_curved_part_has_fewer_edges() {
    use std::f64::consts::PI;
    let curve = Synth::piecewise(&[(PI, 0.0), (PI, 1.0)]);
    let ts = assert_cheap_and_within(&curve, 1e-3, "half line, half circle");
    let straight = ts.windows(2).filter(|w| 0.5 * (w[0] + w[1]) < 0.5).count();
    let curved = ts.len() - 1 - straight;
    assert!(straight < curved, "{straight} straight vs {curved} curved edges");
    assert!(ts.len() - 1 < MAX_MESH_EDGES / 4);
}

#[test]
fn narrow_spike_between_grid_nodes_is_not_missed() {
    // κ = 1 except 1000 on [0.505, 0.506]: inside the grid cell [0.5, 0.515625].
    let curve = Synth::piecewise(&[(0.505, 1.0), (0.001, 1000.0), (0.494, 1.0)]);
    let eps = 1e-3;
    let ts = curve.samples(eps, MAX_MESH_EDGES);
    curve.assert_within(&ts, eps, "narrow spike");
}

// --- display-mesh scenarios (chord tolerance, curvature-driven) ---

fn arc_doc(radius: f64, sweep: f64) -> (Document, GeometryId) {
    let mut doc = Document::new();
    let id = doc.add_geometry::<Arc>(
        GeometryRole::Result,
        (
            Point2::new(0.0, 0.0),
            Length::new(radius),
            HeadingAngle::new(0.0),
            TurnAngle::new(sweep),
        ),
    );
    (doc, id)
}

fn edges_of(mesh: &Mesh, id: GeometryId) -> Vec<&MeshEdge> {
    mesh.edges.iter().filter(|e| e.geometry == id).collect()
}

fn arc_edge_count(radius: f64, sweep: f64, eps: f64) -> usize {
    let (doc, id) = arc_doc(radius, sweep);
    edges_of(&Mesh::from_document(&doc, doc.vector(), MeshFilter::RESULT_ONLY, eps), id).len()
}

#[test]
fn arc_edges_are_equal_and_as_few_as_the_tolerance_allows() {
    for (r, sweep, eps) in [(2.0, 2.5, 0.01), (1.0, std::f64::consts::FRAC_PI_2, 0.001), (20.0, 3.0, 0.005)] {
        let (doc, id) = arc_doc(r, sweep);
        let mesh = Mesh::from_document(&doc, doc.vector(), MeshFilter::RESULT_ONLY, eps);
        let edges = edges_of(&mesh, id);
        let n = edges.len() as f64;
        let first = (edges[0].b - edges[0].a).length();
        for e in &edges {
            assert!(((e.b - e.a).length() - first).abs() < 1e-9 * r, "unequal edges");
        }
        let sagitta = |n: f64| r * (1.0 - (sweep / (2.0 * n)).cos());
        assert!(sagitta(n) <= eps * (1.0 + 1e-9), "R {r}: {n} edges exceed {eps}");
        assert!(sagitta(n - 1.0) > eps, "R {r}: {} edges would do", n - 1.0);
    }
}

#[test]
fn larger_radius_gives_more_edges() {
    let sweep = std::f64::consts::FRAC_PI_2;
    assert!(arc_edge_count(20.0, sweep, 0.01) > arc_edge_count(0.5, sweep, 0.01));
}

#[test]
fn edge_count_grows_in_single_steps() {
    let mut prev = arc_edge_count(5.0, 80f64.to_radians(), 0.005);
    for k in 1..=80 {
        let deg = 80.0 + 0.25 * k as f64;
        let n = arc_edge_count(5.0, deg.to_radians(), 0.005);
        assert!(n >= prev && n - prev <= 1, "{prev} → {n} at {deg}°");
        prev = n;
    }
}

#[test]
fn near_full_arc_is_not_a_single_chord() {
    let (sweep, eps) = (2.0 * std::f64::consts::PI - 0.05, 0.01);
    let n = arc_edge_count(1.0, sweep, eps);
    assert_eq!(n, (sweep / phi_max_acos(1.0, eps)).ceil() as usize);
    assert!(n > 1);
}

#[test]
fn large_arc_stays_within_tolerance() {
    let (r, eps) = (20.0, 0.005);
    let (doc, id) = arc_doc(r, std::f64::consts::PI);
    let mesh = Mesh::from_document(&doc, doc.vector(), MeshFilter::RESULT_ONLY, eps);
    for e in edges_of(&mesh, id) {
        let (ta, tb) = (e.a.y.atan2(e.a.x), e.b.y.atan2(e.b.x));
        for k in 0..=16 {
            let th = ta + (tb - ta) * k as f64 / 16.0;
            let q = Point2::new(r * th.cos(), r * th.sin());
            let d = distance_point_to_segment(q, e.a, e.b);
            assert!(d <= eps * (1.0 + 1e-9), "deviation {d}");
        }
    }
}

#[test]
fn finer_tolerance_gives_at_least_as_many_edges() {
    let mut doc = Document::new();
    let seg = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(0.0, 0.0), Point2::new(5.0, 1.0)));
    let mut ids = vec![seg];
    for (r, sweep) in [(0.5, 1.0), (3.0, 4.0), (20.0, -2.0)] {
        ids.push(doc.add_geometry::<Arc>(
            GeometryRole::Result,
            (Point2::new(1.0, 1.0), Length::new(r), HeadingAngle::new(0.4), TurnAngle::new(sweep)),
        ));
    }
    let eps = 0.01;
    let coarse = Mesh::from_document(&doc, doc.vector(), MeshFilter::RESULT_ONLY, eps);
    let fine = Mesh::from_document(&doc, doc.vector(), MeshFilter::RESULT_ONLY, eps / 4.0);
    for id in ids {
        assert!(edges_of(&fine, id).len() >= edges_of(&coarse, id).len());
    }
}

#[test]
fn tolerance_does_not_change_segments() {
    let mut doc = Document::new();
    let id = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(0.0, 0.0), Point2::new(3.0, 4.0)));
    for eps in [1e-9, 0.01, 1e3] {
        let mesh = Mesh::from_document(&doc, doc.vector(), MeshFilter::RESULT_ONLY, eps);
        let edges = edges_of(&mesh, id);
        assert_eq!(edges.len(), 1);
        assert_eq!((edges[0].a, edges[0].b), (Point2::new(0.0, 0.0), Point2::new(3.0, 4.0)));
    }
}

#[test]
fn same_options_give_the_same_mesh() {
    let (doc, _) = arc_doc(3.0, 2.0);
    let a = Mesh::from_document(&doc, doc.vector(), MeshFilter::RESULT_ONLY, 0.002);
    let b = Mesh::from_document(&doc, doc.vector(), MeshFilter::RESULT_ONLY, 0.002);
    assert_eq!(a, b);
}

#[test]
fn arc_reaching_the_cap_keeps_its_ends() {
    let (doc, id) = arc_doc(1e6, std::f64::consts::PI);
    let mesh = Mesh::from_document(&doc, doc.vector(), MeshFilter::RESULT_ONLY, 1e-6);
    let edges = edges_of(&mesh, id);
    assert_eq!(edges.len(), MAX_MESH_EDGES);
    assert_eq!(edges[0].a, Arc::start_expr(id).evaluate(&doc, doc.vector()));
    assert_eq!(edges.last().unwrap().b, Arc::end_expr(id).evaluate(&doc, doc.vector()));
}

#[test]
#[should_panic(expected = "chord tolerance")]
fn non_positive_tolerance_is_rejected() {
    let doc = Document::new();
    let _ = Mesh::from_document(&doc, doc.vector(), MeshFilter::RESULT_ONLY, 0.0);
}

// --- uniform kinds (closed form) and the half-turn cap ---

#[test]
fn uniform_kinds_need_no_halving_in_the_general_sampler() {
    let mut doc = Document::new();
    let mut ids = vec![doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(0.0, 0.0), Point2::new(5.0, 1.0)))];
    for (r, sweep) in [(0.5, 1.0), (3.0, 4.0), (20.0, -2.0), (1.0, 6.2), (0.0, 1.0)] {
        ids.push(doc.add_geometry::<Arc>(
            GeometryRole::Result,
            (Point2::new(1.0, 1.0), Length::new(r), HeadingAngle::new(0.4), TurnAngle::new(sweep)),
        ));
    }
    let p = doc.vector();
    for id in ids {
        for eps in [1e-4, 0.01, 0.3] {
            let piece = |a, b| doc.curve_piece_bound(id, p, a, b).unwrap();
            let whole = piece(0.0, 1.0);
            let closed = uniform_samples(whole.length, whole.curvature, eps, 1024);
            let general = place_samples(piece, eps, 1024);
            let (nc, ng) = (closed.len() - 1, general.len() - 1);
            assert!(nc <= ng && ng <= nc + 1, "{id:?}, eps {eps}: closed {nc}, general {ng}");
            for (j, t) in general.iter().enumerate() {
                assert!((t - j as f64 / ng as f64).abs() < 1e-12, "{id:?}, eps {eps}: halved");
            }
        }
    }
}

#[test]
fn chord_turn_is_capped_at_a_half_turn() {
    use std::f64::consts::PI;
    assert!((phi_max(1.0, 1.0) - PI).abs() < 1e-12);
    assert_eq!(phi_max(1.0, 3.0), phi_max(1.0, 1.0));
    // Tolerance above the radius: a near-full circle still gets two edges.
    assert_eq!(arc_edge_count(0.1, 2.0 * PI - 0.05, 0.5), 2);
}
