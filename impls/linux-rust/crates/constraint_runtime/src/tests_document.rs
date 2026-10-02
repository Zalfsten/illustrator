use super::*;
use crate::geom::{CurveParam, Point2};
use crate::geometries::{Chart, GeometryRole, Segment, CurveExprs, EndpointExprs};

#[test]
fn two_segment_join_connectivity_indices() {
    let mut doc = Document::new();
    let s0 = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)));
    let s1 = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(2.0, 0.0), Point2::new(3.0, 1.0)));
    doc.add_constraint(Constraint::Congruence {
        a: Segment::end_expr(s0),
        b: Segment::start_expr(s1),
    });
    let c = &doc.constraints()[0];
    let conn = c.param_connectivity(&doc.structure);
    assert_eq!(conn.global_indices, vec![2, 3, 4, 5]);
}

#[test]
fn congruence_between_endpoint_point_exprs() {
    let mut doc = Document::new();
    let s0 = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)));
    let s1 = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(2.0, 0.0), Point2::new(3.0, 1.0)));
    doc.add_constraint(Constraint::Congruence {
        a: Segment::end_expr(s0),
        b: Segment::start_expr(s1),
    });
    match &doc.constraints()[0] {
        Constraint::Congruence { a, b } => {
            assert_eq!(a, &Segment::end_expr(s0));
            assert_eq!(b, &Segment::start_expr(s1));
        }
        _ => panic!("expected Congruence"),
    }
}

#[test]
fn add_geometry_grows_state_and_param_block() {
    let mut doc = Document::new();
    assert_eq!(doc.state().len(), 0);
    let s0 = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)));
    assert_eq!(doc.state().len(), Segment::PARAMETER_COUNT);
    assert_eq!(doc.structure.parameter_count(), Segment::PARAMETER_COUNT);
    let s = doc.geometry(s0).unwrap();
    assert_eq!(s.param_block(), 0..4);
    assert_eq!(
        doc.structure.geometry(s0).unwrap().role(),
        GeometryRole::Result
    );
    let s1 = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(2.0, 0.0), Point2::new(3.0, 1.0)));
    assert_eq!(doc.state().len(), 2 * Segment::PARAMETER_COUNT);
    assert_eq!(doc.geometry(s1).unwrap().param_block(), 4..8);
}

#[test]
fn set_geometry_role_round_trips() {
    let mut doc = Document::new();
    let id = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)));
    assert_eq!(
        doc.structure.geometry(id).unwrap().role(),
        GeometryRole::Result
    );
    assert!(doc.set_geometry_role(id, GeometryRole::Construction));
    assert_eq!(
        doc.structure.geometry(id).unwrap().role(),
        GeometryRole::Construction
    );
    assert_eq!(
        doc.structure.geometry(id).unwrap().role(),
        GeometryRole::Construction
    );
    assert!(doc.set_geometry_role(id, GeometryRole::Result));
    assert_eq!(
        doc.structure.geometry(id).unwrap().role(),
        GeometryRole::Result
    );
}

#[test]
fn remove_geometry_compacts_state_and_drops_orphans() {
    let mut doc = Document::new();
    let s0 = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)));
    let s1 = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(1.0, 0.0), Point2::new(1.0, 1.0)));
    let s2 = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(5.0, 5.0), Point2::new(6.0, 5.0)));
    doc.add_constraint(Constraint::Congruence {
        a: Segment::end_expr(s0),
        b: Segment::start_expr(s1),
    });
    doc.add_constraint(Constraint::Orthogonal {
        a: Segment::tangent_expr(s0, CurveParam::Normalized(0.0)),
        b: Segment::tangent_expr(s1, CurveParam::Normalized(0.0)),
    });
    doc.add_constraint(Constraint::FixPoint {
        point: Segment::start_expr(s2),
        target: Point2::new(5.0, 5.0),
    });
    doc.add_constraint(Constraint::FixPoint {
        point: Segment::on_curve_expr(s1, crate::geom::CurveParam::Uniform(0.5)),
        target: Point2::new(1.0, 0.5),
    });
    assert_eq!(doc.constraints().len(), 4);
    assert!(doc.remove_geometry(s1));
    assert_eq!(doc.geometries().len(), 2);
    assert_eq!(doc.state().len(), 2 * Segment::PARAMETER_COUNT);
    // s0 still at 0..4; s2 remapped from 8..12 to 4..8
    assert_eq!(doc.geometry(s0).unwrap().param_block().start, 0);
    assert_eq!(
        doc.geometry(s2).unwrap().param_block().start,
        Segment::PARAMETER_COUNT
    );
    assert_eq!(doc.state()[4], 5.0);
    // congruence + orthogonal + OnCurve FixPoint referenced s1 → dropped; FixPoint on s2 kept
    assert_eq!(doc.constraints().len(), 1);
    assert!(matches!(
        &doc.constraints()[0],
        Constraint::FixPoint { point, .. }
            if point.referenced_geometry() == s2
    ));
}

#[test]
fn id_generator_allocate_and_observe() {
    use crate::geometries::{GeometryId, IdGenerator};

    let mut ids = IdGenerator::new();
    assert_eq!(ids.allocate(), GeometryId(0));
    assert_eq!(ids.allocate(), GeometryId(1));
    ids.observe(GeometryId(10));
    assert_eq!(ids.allocate(), GeometryId(11));
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
    assert_eq!(s0, crate::geometries::GeometryId(0));
    assert_eq!(s1, crate::geometries::GeometryId(1));
    assert_eq!(doc.geometry(s0).unwrap().id(), s0);
    assert_eq!(doc.geometry(s1).unwrap().id(), s1);
}
