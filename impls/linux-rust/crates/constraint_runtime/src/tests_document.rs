use super::*;
use crate::constraint_kinds::{Congruence, FixPoint, VectorsOrthogonal};
use crate::geom::{CurveParam, HeadingAngle, Length, Point2, TurnAngle};
use crate::geometry_kinds::{Arc, Segment};
use crate::geometry::{Chart, CurveExprs, EndpointExprs, GeometryRole};

#[test]
fn two_segment_join_connectivity_indices() {
    let mut doc = Document::new();
    let s0 = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)));
    let s1 = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(2.0, 0.0), Point2::new(3.0, 1.0)));
    doc.add_constraint(Congruence {
        a: Segment::end_expr(s0),
        b: Segment::start_expr(s1),
    });
    let c = doc.constraints().values().next().unwrap().body();
    let conn = c.param_connectivity(&doc);
    assert_eq!(conn, vec![2, 3, 4, 5]);
}

#[test]
fn congruence_between_endpoint_point_exprs() {
    let mut doc = Document::new();
    let s0 = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)));
    let s1 = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(2.0, 0.0), Point2::new(3.0, 1.0)));
    let a = Segment::end_expr(s0);
    let b = Segment::start_expr(s1);
    let cid = doc.add_constraint(Congruence { a, b });
    let c = doc.constraint(cid).unwrap();
    assert_eq!(c.residual_arity(), 2);
    assert_eq!(doc.constraints().values().next().unwrap().id, cid);
    assert_eq!(
        doc.points_identified_with(&Segment::end_expr(s0)),
        vec![Segment::start_expr(s1)]
    );
}

#[test]
fn add_geometry_grows_state_and_param_block() {
    let mut doc = Document::new();
    assert_eq!(doc.vector().len(), 0);
    let s0 = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)));
    assert_eq!(doc.vector().len(), Segment::PARAMETER_COUNT);
    assert_eq!(doc.parameter_count(), Segment::PARAMETER_COUNT);
    let s = doc.geometry(s0).unwrap();
    assert_eq!(s.param_block(), 0..4);
    assert_eq!(
        doc.geometry(s0).unwrap().role(),
        GeometryRole::Result
    );
    let s1 = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(2.0, 0.0), Point2::new(3.0, 1.0)));
    assert_eq!(doc.vector().len(), 2 * Segment::PARAMETER_COUNT);
    assert_eq!(doc.geometry(s1).unwrap().param_block(), 4..8);
}

#[test]
fn set_geometry_role_round_trips() {
    let mut doc = Document::new();
    let id = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)));
    assert_eq!(
        doc.geometry(id).unwrap().role(),
        GeometryRole::Result
    );
    assert!(doc.set_geometry_role(id, GeometryRole::Construction));
    assert_eq!(
        doc.geometry(id).unwrap().role(),
        GeometryRole::Construction
    );
    assert_eq!(
        doc.geometry(id).unwrap().role(),
        GeometryRole::Construction
    );
    assert!(doc.set_geometry_role(id, GeometryRole::Result));
    assert_eq!(
        doc.geometry(id).unwrap().role(),
        GeometryRole::Result
    );
}

#[test]
fn remove_geometry_compacts_state_and_drops_orphans() {
    let mut doc = Document::new();
    let s0 = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)));
    let s1 = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(1.0, 0.0), Point2::new(1.0, 1.0)));
    let s2 = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(5.0, 5.0), Point2::new(6.0, 5.0)));
    doc.add_constraint(Congruence {
        a: Segment::end_expr(s0),
        b: Segment::start_expr(s1),
    });
    doc.add_constraint(VectorsOrthogonal {
        a: Segment::tangent_expr(s0, CurveParam::Normalized(0.0)),
        b: Segment::tangent_expr(s1, CurveParam::Normalized(0.0)),
    });
    doc.add_constraint(FixPoint {
        point: Segment::start_expr(s2),
        target: Point2::new(5.0, 5.0),
    });
    doc.add_constraint(FixPoint {
        point: Segment::on_curve_expr(s1, crate::geom::CurveParam::Uniform(0.5)),
        target: Point2::new(1.0, 0.5),
    });
    assert_eq!(doc.constraints().len(), 4);
    assert!(doc.remove_geometry(s1));
    assert_eq!(doc.geometries().len(), 2);
    assert_eq!(doc.vector().len(), 2 * Segment::PARAMETER_COUNT);
    // s0 still at 0..4; s2 remapped from 8..12 to 4..8
    assert_eq!(doc.geometry(s0).unwrap().param_block().start, 0);
    assert_eq!(
        doc.geometry(s2).unwrap().param_block().start,
        Segment::PARAMETER_COUNT
    );
    assert_eq!(doc.vector()[4], 5.0);
    // congruence + orthogonal + OnCurve FixPoint referenced s1 → dropped; FixPoint on s2 kept
    assert_eq!(doc.constraints().len(), 1);
    let kept = doc.constraints().values().next().unwrap().body();
    assert!(kept.references_geometry(s2));
    assert!(!kept.references_geometry(s0));
    assert!(doc.constraints_on(s0).is_empty());
    assert!(doc.constraints_on(s1).is_empty());
    assert_eq!(doc.constraints_on(s2).len(), 1);
}

#[test]
fn constraints_on_tracks_incidence() {
    let mut doc = Document::new();
    let s0 = doc.add_geometry::<Segment>(
        GeometryRole::Result,
        (Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)),
    );
    let s1 = doc.add_geometry::<Segment>(
        GeometryRole::Result,
        (Point2::new(1.0, 0.0), Point2::new(1.0, 1.0)),
    );
    let cong = doc.add_constraint(Congruence {
        a: Segment::end_expr(s0),
        b: Segment::start_expr(s1),
    });
    let fix = doc.add_constraint(FixPoint {
        point: Segment::start_expr(s0),
        target: Point2::new(0.0, 0.0),
    });
    assert_eq!(doc.constraints_on(s0), &[cong, fix]);
    assert_eq!(doc.constraints_on(s1), &[cong]);
    assert!(doc.remove_constraint(cong));
    assert_eq!(doc.constraints_on(s0), &[fix]);
    assert!(doc.constraints_on(s1).is_empty());
}

#[test]
fn id_map_next_id_skips_explicit_and_removed_ids() {
    use crate::geometry::GeometryId;
    use crate::id::IdMap;

    let mut ids = IdMap::<GeometryId, ()>::new();
    assert_eq!(ids.next_id(), GeometryId(0));
    ids.insert(GeometryId(0), ());
    ids.insert(GeometryId(10), ());
    assert_eq!(ids.next_id(), GeometryId(11));
    assert_eq!(ids.remove(GeometryId(10)), Some(()));
    assert_eq!(ids.next_id(), GeometryId(11));
    assert_eq!(ids.len(), 1);
}

#[test]
fn constraint_id_allocate_lookup_and_remove() {
    use crate::constraint::ConstraintId;
    let mut doc = Document::new();
    let s0 = doc.add_geometry::<Segment>(
        GeometryRole::Result,
        (Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)),
    );
    let c0 = doc.add_constraint(FixPoint {
        point: Segment::start_expr(s0),
        target: Point2::new(0.0, 0.0),
    });
    let c1 = doc.add_constraint(FixPoint {
        point: Segment::end_expr(s0),
        target: Point2::new(1.0, 0.0),
    });
    assert_eq!(c0, ConstraintId(0));
    assert_eq!(c1, ConstraintId(1));
    assert!(doc.constraint(c0).is_some());
    assert!(doc.remove_constraint(c0));
    assert!(doc.constraint(c0).is_none());
    assert!(doc.constraint(c1).is_some());
    assert_eq!(doc.constraints().len(), 1);
    assert_eq!(doc.constraints().values().next().unwrap().id, c1);
}

#[test]
fn add_geometry_assigns_sequential_ids_without_chart_id() {
    let mut doc = Document::new();
    let s0 = doc.add_geometry::<Segment>(
        GeometryRole::Result,
        (Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)),
    );
    let s1 = doc.add_geometry::<Segment>(
        GeometryRole::Result,
        (Point2::new(1.0, 0.0), Point2::new(1.0, 1.0)),
    );
    assert_eq!(s0, crate::geometry::GeometryId(0));
    assert_eq!(s1, crate::geometry::GeometryId(1));
    assert_eq!(doc.geometry(s0).unwrap().id(), s0);
    assert_eq!(doc.geometry(s1).unwrap().id(), s1);
}

fn flipped_arc_init() -> (Point2, Length, HeadingAngle, TurnAngle) {
    (Point2::new(0.5, 1.0), Length::new(-2.0), HeadingAngle::new(0.3), TurnAngle::new(1.2))
}

#[test]
fn add_geometry_stores_canonical_arc() {
    let init = flipped_arc_init();
    let mut doc = Document::new();
    let id = doc.add_geometry::<Arc>(GeometryRole::Result, init);
    let p = doc.vector();
    assert_eq!(p[2], 2.0, "radius stored non-negative");
    assert!((p[3] - (0.3 + std::f64::consts::PI)).abs() < 1e-12);
    // Same curve as the given (non-canonical) values.
    let start = Arc::start_expr(id).evaluate(&doc, p);
    let want = <Arc as Geometry>::point(crate::geometry::PointFormula::Start, init);
    assert!((start.x - want.x).abs() < 1e-12 && (start.y - want.y).abs() < 1e-12);
    let end = Arc::end_expr(id).evaluate(&doc, p);
    let want = <Arc as Geometry>::point(crate::geometry::PointFormula::End, init);
    assert!((end.x - want.x).abs() < 1e-12 && (end.y - want.y).abs() < 1e-12);
}

#[test]
fn canonicalize_rewrites_only_negative_radius_arc() {
    let mut doc = Document::new();
    doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)));
    let arc = doc.add_geometry::<Arc>(
        GeometryRole::Result,
        (Point2::new(0.0, 0.0), Length::new(1.0), HeadingAngle::new(0.0), TurnAngle::new(1.0)),
    );
    let mut p = doc.clone_vector();
    // Arc block starts after the 4 segment params: negate R, shift heading by π (same curve).
    p[6] = -1.0;
    p[7] = std::f64::consts::PI;
    let before = p.clone();
    let end_before = Arc::end_expr(arc).evaluate(&doc, &p);
    doc.canonicalize(&mut p);
    assert_eq!(&p[0..4], &before[0..4], "segment block untouched");
    assert_eq!(p[6], 1.0);
    assert!((p[7] - 2.0 * std::f64::consts::PI).abs() < 1e-12);
    let end_after = Arc::end_expr(arc).evaluate(&doc, &p);
    assert!((end_after.x - end_before.x).abs() < 1e-12 && (end_after.y - end_before.y).abs() < 1e-12);
}
