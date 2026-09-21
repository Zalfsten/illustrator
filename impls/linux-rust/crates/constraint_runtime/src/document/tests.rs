use super::*;
use crate::geom::{EndpointKind, GeometryRole, PointExpr};

#[test]
fn two_segment_join_connectivity_indices() {
    let mut doc = Document::new();
    let s0 = doc.add_segment(Point2::new(0.0, 0.0), Point2::new(1.0, 0.0));
    let s1 = doc.add_segment(Point2::new(2.0, 0.0), Point2::new(3.0, 1.0));
    doc.join_endpoints(
        EndpointRef::new(s0, EndpointKind::End),
        EndpointRef::new(s1, EndpointKind::Start),
    );
    let c = &doc.constraints()[0];
    let conn = c.param_connectivity(&doc.structure);
    assert_eq!(conn.global_indices, vec![2, 3, 4, 5]);
}

#[test]
fn join_inserts_congruence_only() {
    let mut doc = Document::new();
    let s0 = doc.add_segment(Point2::new(0.0, 0.0), Point2::new(1.0, 0.0));
    let s1 = doc.add_segment(Point2::new(2.0, 0.0), Point2::new(3.0, 1.0));
    doc.join_endpoints(
        EndpointRef::new(s0, EndpointKind::End),
        EndpointRef::new(s1, EndpointKind::Start),
    );
    match &doc.constraints()[0] {
        Constraint::Congruence { a, b } => {
            assert!(matches!(
                a,
                crate::geom::PointExpr::Endpoint {
                    end: EndpointKind::End,
                    ..
                }
            ));
            assert!(matches!(
                b,
                crate::geom::PointExpr::Endpoint {
                    end: EndpointKind::Start,
                    ..
                }
            ));
        }
        _ => panic!("expected Congruence"),
    }
}

#[test]
fn add_segment_grows_state_and_connectivity() {
    let mut doc = Document::new();
    assert_eq!(doc.state().len(), 0);
    let s0 = doc.add_segment(Point2::new(0.0, 0.0), Point2::new(1.0, 0.0));
    assert_eq!(doc.state().len(), SEGMENT_STRIDE);
    assert_eq!(doc.structure.parameter_count(), SEGMENT_STRIDE);
    let s = doc.segment(s0).unwrap();
    assert_eq!(s.connectivity().global_indices, vec![0, 1, 2, 3]);
    assert_eq!(s.role, GeometryRole::Result);
    let s1 = doc.add_segment(Point2::new(2.0, 0.0), Point2::new(3.0, 1.0));
    assert_eq!(doc.state().len(), 2 * SEGMENT_STRIDE);
    assert_eq!(
        doc.segment(s1).unwrap().connectivity().global_indices,
        vec![4, 5, 6, 7]
    );
}

#[test]
fn set_geometry_role_round_trips() {
    let mut doc = Document::new();
    let id = doc.add_segment(Point2::new(0.0, 0.0), Point2::new(1.0, 0.0));
    assert_eq!(doc.segment(id).unwrap().role, GeometryRole::Result);
    assert!(doc.set_geometry_role(id, GeometryRole::Construction));
    assert_eq!(doc.segment(id).unwrap().role, GeometryRole::Construction);
    assert_eq!(
        doc.structure.geometry(id).unwrap().role(),
        GeometryRole::Construction
    );
    assert!(doc.set_geometry_role(id, GeometryRole::Result));
    assert_eq!(doc.segment(id).unwrap().role, GeometryRole::Result);
}

#[test]
fn remove_geometry_compacts_state_and_drops_orphans() {
    let mut doc = Document::new();
    let s0 = doc.add_segment(Point2::new(0.0, 0.0), Point2::new(1.0, 0.0));
    let s1 = doc.add_segment(Point2::new(1.0, 0.0), Point2::new(1.0, 1.0));
    let s2 = doc.add_segment(Point2::new(5.0, 5.0), Point2::new(6.0, 5.0));
    doc.join_endpoints(
        EndpointRef::new(s0, EndpointKind::End),
        EndpointRef::new(s1, EndpointKind::Start),
    );
    doc.add_constraint(Constraint::Orthogonal { a: s0, b: s1 });
    doc.add_constraint(Constraint::FixPoint {
        point: PointExpr::Endpoint {
            geometry: s2,
            end: EndpointKind::Start,
        },
        target: Point2::new(5.0, 5.0),
    });
    doc.add_constraint(Constraint::FixPoint {
        point: PointExpr::OnCurve {
            geometry: s1,
            param: crate::geom::CurveParam::Uniform(0.5),
        },
        target: Point2::new(1.0, 0.5),
    });
    assert_eq!(doc.constraints().len(), 4);
    assert!(doc.remove_geometry(s1));
    assert_eq!(doc.geometries().len(), 2);
    assert_eq!(doc.state().len(), 2 * SEGMENT_STRIDE);
    // s0 still at 0..4; s2 remapped from 8..12 to 4..8
    assert_eq!(doc.segment(s0).unwrap().param_offset, 0);
    assert_eq!(doc.segment(s2).unwrap().param_offset, SEGMENT_STRIDE);
    assert_eq!(doc.state()[4], 5.0);
    // congruence + orthogonal + OnCurve FixPoint referenced s1 → dropped; FixPoint on s2 kept
    assert_eq!(doc.constraints().len(), 1);
    assert!(matches!(
        &doc.constraints()[0],
        Constraint::FixPoint { point, .. }
            if matches!(point, PointExpr::Endpoint { geometry, .. } if *geometry == s2)
    ));
}
