use super::*;
use crate::constraint_kinds::Congruence;
use crate::geom::Point2;
use crate::geometry::{EndpointExprs, GeometryId};
use crate::geometry_kinds::Segment;

fn seg(doc: &mut Document, role: GeometryRole, a: (f64, f64), b: (f64, f64)) -> GeometryId {
    doc.add_geometry::<Segment>(role, (Point2::new(a.0, a.1), Point2::new(b.0, b.1)))
}

fn same_set(a: &[PointExpr], b: &[PointExpr]) -> bool {
    a.len() == b.len() && a.iter().all(|x| b.contains(x))
}

/// Three segments meeting at (1, 0), joined as a chain s0.end = s1.start = s2.start.
fn chain() -> (Document, [GeometryId; 3]) {
    let mut doc = Document::new();
    let s0 = seg(&mut doc, GeometryRole::Result, (0.0, 0.0), (1.0, 0.0));
    let s1 = seg(&mut doc, GeometryRole::Result, (1.0, 0.0), (2.0, 0.0));
    let s2 = seg(&mut doc, GeometryRole::Result, (1.0, 0.0), (1.0, 1.0));
    doc.add_constraint(Congruence {
        a: Segment::end_expr(s0),
        b: Segment::start_expr(s1),
    });
    doc.add_constraint(Congruence {
        a: Segment::start_expr(s1),
        b: Segment::start_expr(s2),
    });
    (doc, [s0, s1, s2])
}

#[test]
fn unjoined_point_is_its_own_class() {
    let mut doc = Document::new();
    let s = seg(&mut doc, GeometryRole::Result, (0.0, 0.0), (1.0, 0.0));
    assert_eq!(
        doc.point_class(&Segment::end_expr(s)),
        vec![Segment::end_expr(s)]
    );
}

#[test]
fn direct_join_is_one_class() {
    let mut doc = Document::new();
    let s0 = seg(&mut doc, GeometryRole::Result, (0.0, 0.0), (1.0, 0.0));
    let s1 = seg(&mut doc, GeometryRole::Result, (1.0, 0.0), (2.0, 0.0));
    doc.add_constraint(Congruence {
        a: Segment::end_expr(s0),
        b: Segment::start_expr(s1),
    });
    assert!(same_set(
        &doc.point_class(&Segment::start_expr(s1)),
        &[Segment::end_expr(s0), Segment::start_expr(s1)]
    ));
}

#[test]
fn join_chain_is_one_class_from_every_member() {
    let (doc, [s0, s1, s2]) = chain();
    let all = [
        Segment::end_expr(s0),
        Segment::start_expr(s1),
        Segment::start_expr(s2),
    ];
    for p in &all {
        assert!(same_set(&doc.point_class(p), &all), "class of {p:?}");
    }
}

#[test]
fn coincident_unjoined_points_are_separate_classes() {
    let mut doc = Document::new();
    let s0 = seg(&mut doc, GeometryRole::Result, (0.0, 0.0), (1.0, 0.0));
    let s1 = seg(&mut doc, GeometryRole::Result, (1.0, 0.0), (2.0, 0.0));
    assert!(!doc
        .point_class(&Segment::end_expr(s0))
        .contains(&Segment::start_expr(s1)));
}

#[test]
fn removing_the_middle_link_splits_the_class() {
    let (mut doc, [s0, s1, s2]) = chain();
    let middle = doc.constraints().iter().map(|(id, _)| id).last().unwrap();
    doc.remove_constraint(middle).unwrap();
    assert!(same_set(
        &doc.point_class(&Segment::end_expr(s0)),
        &[Segment::end_expr(s0), Segment::start_expr(s1)]
    ));
    assert_eq!(
        doc.point_class(&Segment::start_expr(s2)),
        vec![Segment::start_expr(s2)]
    );
}

#[test]
fn coincident_unjoined_endpoints_are_two_logical_points() {
    let mut doc = Document::new();
    let s0 = seg(&mut doc, GeometryRole::Result, (0.0, 0.0), (1.0, 0.0));
    let s1 = seg(&mut doc, GeometryRole::Result, (1.0, 0.0), (2.0, 0.0));
    let reps: Vec<_> = doc
        .logical_points(MeshFilter::RESULT_ONLY)
        .into_iter()
        .map(|lp| lp.representative)
        .collect();
    assert_eq!(
        reps,
        vec![
            Segment::start_expr(s0),
            Segment::end_expr(s0),
            Segment::start_expr(s1),
            Segment::end_expr(s1),
        ]
    );
}

#[test]
fn join_chain_is_one_logical_point_with_lowest_id_representative() {
    let (doc, [s0, s1, s2]) = chain();
    let points = doc.logical_points(MeshFilter::RESULT_ONLY);
    // s0.start, corner, s1.end, s2.end
    assert_eq!(points.len(), 4);
    let corner = &points[1];
    assert_eq!(corner.representative, Segment::end_expr(s0));
    assert!(same_set(
        &corner.members,
        &[
            Segment::end_expr(s0),
            Segment::start_expr(s1),
            Segment::start_expr(s2),
        ]
    ));
}

#[test]
fn construction_endpoints_follow_the_filter() {
    let mut doc = Document::new();
    seg(&mut doc, GeometryRole::Result, (0.0, 0.0), (1.0, 0.0));
    let c = seg(&mut doc, GeometryRole::Construction, (0.0, 1.0), (1.0, 1.0));
    let has_c = |filter| {
        doc.logical_points(filter)
            .iter()
            .any(|lp| lp.members.contains(&Segment::start_expr(c)))
    };
    assert!(!has_c(MeshFilter::RESULT_ONLY));
    assert!(has_c(MeshFilter::INCLUDE_CONSTRUCTION));
}

#[test]
fn hidden_member_does_not_become_representative() {
    let mut doc = Document::new();
    let c = seg(&mut doc, GeometryRole::Construction, (0.0, 0.0), (1.0, 0.0));
    let r = seg(&mut doc, GeometryRole::Result, (1.0, 0.0), (2.0, 0.0));
    doc.add_constraint(Congruence {
        a: Segment::end_expr(c),
        b: Segment::start_expr(r),
    });
    let points = doc.logical_points(MeshFilter::RESULT_ONLY);
    let corner = points
        .iter()
        .find(|lp| lp.members.contains(&Segment::end_expr(c)))
        .unwrap();
    assert_eq!(corner.representative, Segment::start_expr(r));
}

#[test]
fn logical_points_do_not_depend_on_values() {
    let mut doc = Document::new();
    seg(&mut doc, GeometryRole::Result, (0.0, 0.0), (1.0, 0.0));
    seg(&mut doc, GeometryRole::Result, (5.0, 0.0), (6.0, 0.0));
    let before = doc.logical_points(MeshFilter::RESULT_ONLY);
    // Move the second segment's start onto the first segment's end.
    let v = doc.vector_mut();
    for i in 0..v.len() {
        v[i] = 1.0;
    }
    assert_eq!(doc.logical_points(MeshFilter::RESULT_ONLY), before);
}
