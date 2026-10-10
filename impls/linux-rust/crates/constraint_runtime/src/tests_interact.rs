use super::*;
use crate::constraint_kinds::{Congruence, EqualLength, FixPoint, Orthogonal, Parallel};
use crate::geom::{CurveParam, HeadingAngle, Length, Point2, TurnAngle};
use crate::geometry_kinds::{Arc, Segment};
use crate::geometry::{CurveExprs, EndpointExprs, GeometryRole};
use crate::solver::StubSolver;

#[test]
fn soft_not_persisted_after_drag() {
    let mut doc = Document::new();
    let s0 = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)));
    doc.add_constraint(FixPoint {
        point: Segment::start_expr(s0),
        target: Point2::new(0.0, 0.0),
    });
    let n = doc.constraints().len();
    let mut interact = InteractiveSolve::new();
    let mut drag = DragSession::new();
    let expr = Segment::end_expr(s0);
    drag.pointer_down(&doc, expr);
    let _ = drag.pointer_move(&mut interact, &mut doc, (1.5, 0.2));
    let _ = drag.pointer_up(&mut interact, &mut doc, (1.5, 0.2));
    assert_eq!(doc.constraints().len(), n);
}

#[test]
fn failed_solve_keeps_state() {
    let mut doc = Document::new();
    let s0 = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)));
    doc.add_constraint(FixPoint {
        point: Segment::start_expr(s0),
        target: Point2::new(0.0, 0.0),
    });
    let mut interact = InteractiveSolve {
        solver: StubSolver { succeed: false },
        soft_weights: SoftTerms::default(),
        opts: SolverOptions {
            tol: crate::SOLVE_TOL,
            ..Default::default()
        },
    };
    // Conflicting FixPoints: start cannot be at two places.
    doc.add_constraint(FixPoint {
        point: Segment::start_expr(s0),
        target: Point2::new(10.0, 10.0),
    });
    let before = doc.clone_vector();
    let soft: [SoftContribution; 0] = [];
    let err = interact
        .solve_commit(&mut doc, &soft, &before)
        .expect_err("must fail");
    assert!(!err.message.is_empty());
    assert_eq!(doc.vector(), &before);
}

#[test]
fn atomic_drag_undo() {
    let mut doc = Document::new();
    let s0 = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)));
    let s1 = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(1.0, 0.0), Point2::new(1.0, 1.0)));
    doc.add_constraint(Congruence {
        a: Segment::end_expr(s0),
        b: Segment::start_expr(s1),
    });
    doc.add_constraint(Orthogonal {
        a: Segment::tangent_expr(s0, CurveParam::Normalized(0.0)),
        b: Segment::tangent_expr(s1, CurveParam::Normalized(0.0)),
    });
    doc.add_constraint(FixPoint {
        point: Segment::start_expr(s0),
        target: Point2::new(0.0, 0.0),
    });
    let mut interact = InteractiveSolve::new();
    interact.structural_only(&mut doc).unwrap();
    let before = doc.clone_vector();
    let mut drag = DragSession::new();
    let free = Segment::end_expr(s1);
    drag.pointer_down(&doc, free);
    drag.pointer_move(&mut interact, &mut doc, (2.0, 2.0))
        .unwrap();
    drag.pointer_up(&mut interact, &mut doc, (2.0, 2.0))
        .unwrap();
    assert!(drag.undo(&mut doc));
    assert_eq!(doc.vector(), &before);
}

fn fillet_doc_without_length_pin(r: f64) -> (Document, crate::geometry::GeometryId) {
    let mut doc = Document::new();
    let s0 = doc.add_geometry::<Segment>(
        GeometryRole::Result,
        (Point2::new(0.0, 0.0), Point2::new(2.0 - r, 0.0)),
    );
    let fillet = doc.add_geometry::<Arc>(
        GeometryRole::Result,
        (
            Point2::new(2.0 - r, r),
            Length::new(r),
            HeadingAngle::new(-std::f64::consts::FRAC_PI_2),
            TurnAngle::new(std::f64::consts::FRAC_PI_2),
        ),
    );
    let s1 = doc.add_geometry::<Segment>(
        GeometryRole::Result,
        (Point2::new(2.0, r), Point2::new(2.0, 2.0)),
    );
    doc.add_constraint(Congruence {
        a: Segment::end_expr(s0),
        b: Arc::start_expr(fillet),
    });
    doc.add_constraint(Congruence {
        a: Arc::end_expr(fillet),
        b: Segment::start_expr(s1),
    });
    doc.add_constraint(Parallel {
        a: Segment::tangent_expr(s0, CurveParam::Normalized(0.0)),
        b: Arc::tangent_expr(fillet, CurveParam::Normalized(0.0)),
    });
    doc.add_constraint(Parallel {
        a: Segment::tangent_expr(s1, CurveParam::Normalized(0.0)),
        b: Arc::tangent_expr(fillet, CurveParam::Normalized(1.0)),
    });
    doc.add_constraint(Orthogonal {
        a: Segment::tangent_expr(s0, CurveParam::Normalized(0.0)),
        b: Segment::tangent_expr(s1, CurveParam::Normalized(0.0)),
    });
    doc.add_constraint(FixPoint {
        point: Segment::start_expr(s0),
        target: Point2::new(0.0, 0.0),
    });
    (doc, fillet)
}

#[test]
fn drag_arc_soft_length_floor_resists_collapse() {
    const R: f64 = 0.5;
    let (mut doc, fillet) = fillet_doc_without_length_pin(R);
    let mut interact = InteractiveSolve::new();
    interact.structural_only(&mut doc).unwrap();

    let mut drag = DragSession::new();
    // Collapse attractor: pull the arc start toward the corner (2,0).
    drag.pointer_down(&doc, Arc::start_expr(fillet));
    drag.pointer_move(&mut interact, &mut doc, (1.95, 0.0))
        .unwrap();
    drag.pointer_up(&mut interact, &mut doc, (1.95, 0.0))
        .unwrap();

    let len = Arc::length_expr(fillet)
        .evaluate(&doc, doc.vector())
        .value();
    let l0 = R * std::f64::consts::FRAC_PI_2;
    let floor = interact.soft_weights.min_curve_length_fraction * l0;
    assert!(
        len >= floor * 0.5,
        "soft length floor failed: L={len}, expected ≥ ~{floor}"
    );
    let start = Arc::start_expr(fillet).evaluate(&doc, doc.vector());
    let end = Arc::end_expr(fillet).evaluate(&doc, doc.vector());
    let join_dist = (start.x - end.x).hypot(start.y - end.y);
    assert!(
        join_dist > 1e-2,
        "arc endpoints collapsed (dist={join_dist})"
    );
}

#[test]
fn drag_arc_endpoint_keeps_fillet_radius() {
    const R: f64 = 0.5;
    let (mut doc, fillet) = fillet_doc_without_length_pin(R);
    let arc_len = R * std::f64::consts::FRAC_PI_2;
    let length_gauge = doc.add_geometry::<Segment>(
        GeometryRole::Construction,
        (Point2::new(-1.0, -1.0), Point2::new(-1.0 + arc_len, -1.0)),
    );
    doc.add_constraint(FixPoint {
        point: Segment::start_expr(length_gauge),
        target: Point2::new(-1.0, -1.0),
    });
    doc.add_constraint(FixPoint {
        point: Segment::end_expr(length_gauge),
        target: Point2::new(-1.0 + arc_len, -1.0),
    });
    doc.add_constraint(EqualLength {
        a: Arc::length_expr(fillet),
        b: Segment::length_expr(length_gauge),
    });
    let mut interact = InteractiveSolve::new();
    interact.structural_only(&mut doc).unwrap();

    let mut drag = DragSession::new();
    // Drag the arc start join slightly along the horizontal leg.
    drag.pointer_down(&doc, Arc::start_expr(fillet));
    drag.pointer_move(&mut interact, &mut doc, (1.3, 0.0))
        .unwrap();
    drag.pointer_up(&mut interact, &mut doc, (1.3, 0.0))
        .unwrap();

    let len = Arc::length_expr(fillet)
        .evaluate(&doc, doc.vector())
        .value();
    assert!(
        (len - arc_len).abs() < 1e-3,
        "arc length not fixed: L={len}"
    );
    let start = Arc::start_expr(fillet).evaluate(&doc, doc.vector());
    let end = Arc::end_expr(fillet).evaluate(&doc, doc.vector());
    let join_dist = (start.x - end.x).hypot(start.y - end.y);
    assert!(
        join_dist > 0.3,
        "arc endpoints merged into a corner (dist={join_dist})"
    );
}
