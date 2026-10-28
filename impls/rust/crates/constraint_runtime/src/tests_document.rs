use super::*;
use crate::constraint_kinds::{Congruence, FixPoint, VectorsOrthogonal, VectorsParallel};
use crate::geom::{CurveParam, HeadingAngle, Length, Point2, TurnAngle};
use crate::geometry_kinds::{Arc, Segment};
use crate::names::{NameError, ObjectId};
use crate::geometry::{
    Caps, Chart, CircularExprs, CurveExprs, EndpointExprs, GeometryId, GeometryRole, LineExprs,
};

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
    assert!(doc.remove_geometry(s1).is_some());
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
    assert!(doc.remove_constraint(cong).is_some());
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
    assert!(doc.remove_constraint(c0).is_some());
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

fn segment_and_arc(doc: &mut Document) -> (GeometryId, GeometryId) {
    let seg = doc.add_geometry::<Segment>(
        GeometryRole::Result,
        (Point2::new(1.0, 2.0), Point2::new(4.0, 6.0)),
    );
    let arc = doc.add_geometry::<Arc>(
        GeometryRole::Result,
        (
            Point2::new(-1.0, 3.0),
            Length::new(2.0),
            HeadingAngle::new(0.0),
            TurnAngle::new(1.0),
        ),
    );
    (seg, arc)
}

#[test]
fn caps_by_id_match_kind() {
    let mut doc = Document::new();
    let (seg, arc) = segment_and_arc(&mut doc);
    assert_eq!(
        doc.caps(seg),
        Some(Caps::CURVE.with_endpoints().with_line())
    );
    assert_eq!(
        doc.caps(arc),
        Some(Caps::CURVE.with_endpoints().with_circular())
    );
}

/// Curve and endpoint factories by id equal kind `K`'s static factories.
fn assert_curve_and_endpoint_factories<K: CurveExprs + EndpointExprs>(
    doc: &Document,
    id: GeometryId,
) {
    let t = CurveParam::Normalized(0.25);
    assert_eq!(doc.on_curve_expr(id, t), Some(K::on_curve_expr(id, t)));
    assert_eq!(doc.tangent_expr(id, t), Some(K::tangent_expr(id, t)));
    assert_eq!(doc.normal_expr(id, t), Some(K::normal_expr(id, t)));
    assert_eq!(doc.length_expr(id), Some(K::length_expr(id)));
    assert_eq!(doc.start_expr(id), Some(K::start_expr(id)));
    assert_eq!(doc.end_expr(id), Some(K::end_expr(id)));
}

#[test]
fn identity_factories_match_static_factories_by_capability() {
    let mut doc = Document::new();
    let (seg, arc) = segment_and_arc(&mut doc);
    assert_curve_and_endpoint_factories::<Segment>(&doc, seg);
    assert_curve_and_endpoint_factories::<Arc>(&doc, arc);

    assert_eq!(doc.direction_expr(seg), Some(Segment::direction_expr(seg)));
    assert_eq!(doc.direction_expr(arc), None);
    assert_eq!(doc.center_expr(arc), Some(Arc::center_expr(arc)));
    assert_eq!(doc.radius_expr(arc), Some(Arc::radius_expr(arc)));
    assert_eq!(doc.center_expr(seg), None);
    assert_eq!(doc.radius_expr(seg), None);
}

#[test]
fn identity_factories_evaluate_like_the_kind() {
    let mut doc = Document::new();
    let (seg, arc) = segment_and_arc(&mut doc);
    let p = doc.clone_vector();

    let d = doc.direction_expr(seg).unwrap().evaluate(&doc, &p);
    assert_eq!((d.x, d.y), (3.0, 4.0), "end − start");

    let c = doc.center_expr(arc).unwrap().evaluate(&doc, &p);
    assert_eq!((c.x, c.y), (-1.0, 3.0), "chart center");
}

#[test]
fn unknown_id_has_no_caps_and_no_expressions() {
    let mut doc = Document::new();
    let (seg, _) = segment_and_arc(&mut doc);
    assert!(doc.remove_geometry(seg).is_some());

    let t = CurveParam::Normalized(0.0);
    assert_eq!(doc.caps(seg), None);
    assert_eq!(doc.on_curve_expr(seg, t), None);
    assert_eq!(doc.tangent_expr(seg, t), None);
    assert_eq!(doc.normal_expr(seg, t), None);
    assert_eq!(doc.length_expr(seg), None);
    assert_eq!(doc.start_expr(seg), None);
    assert_eq!(doc.end_expr(seg), None);
    assert_eq!(doc.direction_expr(seg), None);
    assert_eq!(doc.center_expr(seg), None);
    assert_eq!(doc.radius_expr(seg), None);
}

fn segment(doc: &mut Document, x: f64) -> GeometryId {
    doc.add_geometry::<Segment>(
        GeometryRole::Result,
        (Point2::new(x, 0.0), Point2::new(x + 1.0, 1.0)),
    )
}

#[test]
fn geometry_and_constraint_names_do_not_collide() {
    let mut doc = Document::new();
    let s = segment(&mut doc, 0.0);
    let c = doc.add_constraint(FixPoint {
        point: Segment::start_expr(s),
        target: Point2::new(0.0, 0.0),
    });
    assert_eq!(doc.geometry_name(s), Some("seg1"));
    assert_eq!(doc.constraint_name(c), Some("fix1"));
    assert_eq!(doc.lookup("seg1"), Some(ObjectId::Geometry(s)));
    assert_eq!(doc.lookup("fix1"), Some(ObjectId::Constraint(c)));
    assert_eq!(doc.name(ObjectId::Constraint(c)), Some("fix1"));
}

#[test]
fn constraint_kinds_have_their_own_prefixes() {
    let mut doc = Document::new();
    let (a, b) = (segment(&mut doc, 0.0), segment(&mut doc, 2.0));
    let par = doc.add_constraint(VectorsParallel {
        a: Segment::direction_expr(a),
        b: Segment::direction_expr(b),
    });
    let orth = doc.add_constraint(VectorsOrthogonal {
        a: Segment::direction_expr(a),
        b: Segment::direction_expr(b),
    });
    assert_eq!(doc.constraint_name(par), Some("par1"));
    assert_eq!(doc.constraint_name(orth), Some("orth1"));
}

#[test]
fn other_edits_keep_names() {
    let mut doc = Document::new();
    let ids: Vec<_> = (0..3).map(|i| segment(&mut doc, i as f64)).collect();
    doc.remove_geometry(ids[1]).unwrap();
    assert_eq!(doc.geometry_name(ids[0]), Some("seg1"));
    assert_eq!(doc.geometry_name(ids[2]), Some("seg3"));
    assert_eq!(doc.lookup("seg2"), None);
}

#[test]
fn geometry_removal_reports_and_drops_cascaded_names() {
    let mut doc = Document::new();
    let (s1, s2) = (segment(&mut doc, 0.0), segment(&mut doc, 2.0));
    doc.add_constraint(VectorsParallel {
        a: Segment::direction_expr(s1),
        b: Segment::direction_expr(s2),
    });
    doc.add_constraint(Congruence {
        a: Segment::end_expr(s1),
        b: Segment::start_expr(s2),
    });

    let removal = doc.remove_geometry(s1).unwrap();
    let mut names: Vec<_> = removal.names().collect();
    names.sort();
    assert_eq!(names, ["join1", "par1", "seg1"]);
    for n in ["seg1", "par1", "join1"] {
        assert_eq!(doc.lookup(n), None, "{n}");
    }
    assert_eq!(doc.lookup("seg2"), Some(ObjectId::Geometry(s2)));
}

#[test]
fn constraint_removal_reports_its_name() {
    let mut doc = Document::new();
    let s = segment(&mut doc, 0.0);
    let c = doc.add_constraint(FixPoint {
        point: Segment::start_expr(s),
        target: Point2::new(0.0, 0.0),
    });
    let removal = doc.remove_constraint(c).unwrap();
    assert_eq!(removal.constraints, [(c, "fix1".to_owned())]);
    assert!(removal.geometries.is_empty());
    assert_eq!(doc.lookup("fix1"), None);
}

#[test]
fn rename_keeps_the_drawing() {
    let mut doc = Document::new();
    let (s1, s2) = (segment(&mut doc, 0.0), segment(&mut doc, 2.0));
    let par = doc.add_constraint(VectorsParallel {
        a: Segment::direction_expr(s1),
        b: Segment::direction_expr(s2),
    });
    let values = doc.clone_vector();

    doc.rename(ObjectId::Geometry(s1), "x_axis").unwrap();
    assert_eq!(doc.lookup("x_axis"), Some(ObjectId::Geometry(s1)));
    assert_eq!(doc.lookup("seg1"), None);
    assert_eq!(doc.clone_vector(), values);
    assert_eq!(doc.constraints_on(s1), [par]);
    assert_eq!(
        doc.constraint(par).unwrap().referenced_geometries(),
        vec![s1, s2]
    );
    assert_eq!(
        doc.rename(ObjectId::Geometry(s2), "x_axis"),
        Err(NameError::Taken)
    );
}

#[test]
fn restoring_a_snapshot_restores_names() {
    let mut doc = Document::new();
    let s = segment(&mut doc, 0.0);
    let snapshot = doc.clone();
    doc.rename(ObjectId::Geometry(s), "x_axis").unwrap();
    segment(&mut doc, 2.0);
    doc = snapshot;
    assert_eq!(doc.geometry_name(s), Some("seg1"));
    assert_eq!(doc.lookup("x_axis"), None);
    assert_eq!(doc.lookup("seg2"), None);
}
