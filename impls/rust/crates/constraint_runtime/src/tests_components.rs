use super::*;
use crate::assemble::structural_residual_norm;
use crate::constraint_kinds::{Congruence, FixPoint, VectorsParallel};
use crate::geom::Point2;
use crate::geometry::{EndpointExprs, GeometryRole, LineExprs};
use crate::geometry_kinds::Segment;

fn seg(doc: &mut Document, a: (f64, f64), b: (f64, f64)) -> GeometryId {
    doc.add_geometry::<Segment>(
        GeometryRole::Result,
        (Point2::new(a.0, a.1), Point2::new(b.0, b.1)),
    )
}

fn parallel(a: GeometryId, b: GeometryId) -> VectorsParallel {
    VectorsParallel {
        a: Segment::direction_expr(a),
        b: Segment::direction_expr(b),
    }
}

#[test]
fn joined_segments_form_one_component() {
    let mut doc = Document::new();
    let s1 = seg(&mut doc, (0.0, 0.0), (1.0, 0.0));
    let s2 = seg(&mut doc, (1.0, 0.0), (1.0, 1.0));
    let s3 = seg(&mut doc, (5.0, 5.0), (6.0, 5.0));
    let j = doc.add_constraint(Congruence {
        a: Segment::end_expr(s1),
        b: Segment::start_expr(s2),
    });
    assert_eq!(
        doc.components(),
        vec![
            Component {
                geometries: vec![s1, s2],
                constraints: vec![j],
            },
            Component {
                geometries: vec![s3],
                constraints: vec![],
            },
        ]
    );
    assert_eq!(doc.component_of(s2), doc.component_of(s1));
}

#[test]
fn removing_the_only_link_splits_a_component() {
    let mut doc = Document::new();
    let s1 = seg(&mut doc, (0.0, 0.0), (1.0, 0.0));
    let s2 = seg(&mut doc, (0.0, 1.0), (1.0, 1.0));
    let par = doc.add_constraint(parallel(s1, s2));
    assert_eq!(doc.components().len(), 1);
    doc.remove_constraint(par);
    assert_eq!(doc.components().len(), 2);
    assert!(!doc.component_of(s1).contains(s2));
}

#[test]
fn coincident_unconstrained_segments_stay_separate() {
    let mut doc = Document::new();
    let s1 = seg(&mut doc, (0.0, 0.0), (1.0, 0.0));
    let s2 = seg(&mut doc, (1.0, 0.0), (2.0, 0.0));
    assert_eq!(doc.component_of(s1).geometries, vec![s1]);
    assert_eq!(doc.component_of(s2).geometries, vec![s2]);
}

#[test]
fn chain_through_several_constraints_is_one_component() {
    let mut doc = Document::new();
    let s1 = seg(&mut doc, (0.0, 0.0), (1.0, 0.0));
    let s2 = seg(&mut doc, (0.0, 1.0), (1.0, 1.0));
    let s3 = seg(&mut doc, (0.0, 2.0), (1.0, 2.0));
    let p1 = doc.add_constraint(parallel(s1, s2));
    let p2 = doc.add_constraint(parallel(s3, s2));
    let c = doc.component_of(s3);
    assert_eq!(c.geometries, vec![s1, s2, s3]);
    assert_eq!(c.constraints, vec![p1, p2]);
}

#[test]
fn unsatisfied_fix_makes_its_component_inconsistent() {
    let mut doc = Document::new();
    let s1 = seg(&mut doc, (0.0, 0.0), (1.0, 0.0));
    let s2 = seg(&mut doc, (5.0, 0.0), (6.0, 0.0));
    doc.add_constraint(FixPoint {
        point: Segment::start_expr(s1),
        target: Point2::new(0.0, 10.0 * COMMIT_TOL),
    });
    doc.add_constraint(FixPoint {
        point: Segment::start_expr(s2),
        target: Point2::new(5.0, 0.0),
    });
    assert!(!doc.is_consistent(&doc.component_of(s1)));
    assert!(doc.is_consistent(&doc.component_of(s2)));
}

#[test]
fn residual_within_commit_tolerance_is_consistent() {
    let mut doc = Document::new();
    let s = seg(&mut doc, (0.0, 0.0), (1.0, 0.0));
    doc.add_constraint(FixPoint {
        point: Segment::start_expr(s),
        target: Point2::new(0.0, 0.5 * COMMIT_TOL),
    });
    assert!(doc.is_consistent(&doc.component_of(s)));
}

#[test]
fn single_component_residual_is_the_structural_norm() {
    let mut doc = Document::new();
    let s1 = seg(&mut doc, (0.0, 0.0), (1.0, 0.0));
    let s2 = seg(&mut doc, (0.0, 1.0), (1.0, 2.0));
    doc.add_constraint(parallel(s1, s2));
    doc.add_constraint(FixPoint {
        point: Segment::end_expr(s1),
        target: Point2::new(2.0, 0.5),
    });
    let c = doc.component_of(s1);
    let r = doc.component_residual(&c, doc.vector());
    assert!(r > 0.0);
    assert!((r - structural_residual_norm(&doc)).abs() <= 1e-15 * r.max(1.0));
}
