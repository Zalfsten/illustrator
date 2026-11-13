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

#[test]
fn zero_curvature_places_one_edge() {
    assert_eq!(place_samples(|_| 3.0, |_| 0.0, 1e-3, 1024), vec![0.0, 1.0]);
}

#[test]
fn constant_curvature_places_equal_shares_with_minimal_count() {
    let (r, sweep, eps) = (2.0, 4.0, 0.01);
    let ts = place_samples(|_| r * sweep, |_| 1.0 / r, eps, 1024);
    let n = ts.len() - 1;
    assert_eq!(n, (sweep / phi_max_acos(1.0 / r, eps)).ceil() as usize);
    for (j, t) in ts.iter().enumerate() {
        assert!((t - j as f64 / n as f64).abs() < 1e-12, "t_{j} = {t}");
    }
}

/// γ(t) = (2πt, sin 2πt): one full period, equal end tangents, inflections.
fn sine(t: f64) -> Point2 {
    let x = 2.0 * std::f64::consts::PI * t;
    Point2::new(x, x.sin())
}

#[test]
fn sine_period_stays_within_tolerance() {
    use std::f64::consts::PI;
    let eps = 0.01;
    let ts = place_samples(
        |t| {
            let c = (2.0 * PI * t).cos();
            2.0 * PI * (1.0 + c * c).sqrt()
        },
        |t| {
            let x = 2.0 * PI * t;
            x.sin().abs() / (1.0 + x.cos().powi(2)).powf(1.5)
        },
        eps,
        1024,
    );
    assert!(ts.len() > 2, "sine must not be a single chord");
    for w in ts.windows(2) {
        let (a, b) = (sine(w[0]), sine(w[1]));
        for k in 1..20 {
            let q = sine(w[0] + (w[1] - w[0]) * k as f64 / 20.0);
            let d = distance_point_to_segment(q, a, b);
            assert!(d <= eps, "deviation {d} on [{}, {}]", w[0], w[1]);
        }
    }
}

#[test]
fn curvature_spike_gets_denser_samples() {
    let ts = place_samples(
        |_| 1.0,
        |t| 1.0 + 1000.0 * (-((t - 0.5) / 0.03).powi(2)).exp(),
        1e-3,
        1024,
    );
    let edge_at = |t: f64| {
        ts.windows(2)
            .find(|w| w[0] <= t && t <= w[1])
            .map(|w| w[1] - w[0])
            .unwrap()
    };
    assert!(edge_at(0.5) * 5.0 < edge_at(0.1), "{} vs {}", edge_at(0.5), edge_at(0.1));
}

#[test]
fn cap_bounds_the_edge_count() {
    let ts = place_samples(|_| 1.0, |_| 1e6, 1e-6, 8);
    assert_eq!(ts.len(), 9);
    assert_eq!((ts[0], ts[8]), (0.0, 1.0));
}

#[test]
fn zero_speed_with_infinite_curvature_is_one_edge() {
    let ts = place_samples(|_| 0.0, |_| f64::INFINITY, 1e-3, 1024);
    assert_eq!(ts, vec![0.0, 1.0]);
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
