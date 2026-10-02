use super::*;
use crate::constraint::Constraint;
use crate::geom::{CurveParam, Point2};
use crate::geometry_kinds::Segment;
use crate::geometry::{CurveExprs, EndpointExprs, GeometryRole};
use crate::solver::StubSolver;

#[test]
fn soft_not_persisted_after_drag() {
    let mut doc = Document::new();
    let s0 = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)));
    doc.add_constraint(Constraint::FixPoint {
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
    doc.add_constraint(Constraint::FixPoint {
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
    doc.add_constraint(Constraint::FixPoint {
        point: Segment::start_expr(s0),
        target: Point2::new(10.0, 10.0),
    });
    let before = doc.clone_state();
    let soft: [SoftContribution; 0] = [];
    let err = interact
        .solve_commit(&mut doc, &soft, &before)
        .expect_err("must fail");
    assert!(!err.message.is_empty());
    assert_eq!(doc.state(), &before);
}

#[test]
fn atomic_drag_undo() {
    let mut doc = Document::new();
    let s0 = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)));
    let s1 = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(1.0, 0.0), Point2::new(1.0, 1.0)));
    doc.add_constraint(Constraint::Congruence {
        a: Segment::end_expr(s0),
        b: Segment::start_expr(s1),
    });
    doc.add_constraint(Constraint::Orthogonal {
        a: Segment::tangent_expr(s0, CurveParam::Normalized(0.0)),
        b: Segment::tangent_expr(s1, CurveParam::Normalized(0.0)),
    });
    doc.add_constraint(Constraint::FixPoint {
        point: Segment::start_expr(s0),
        target: Point2::new(0.0, 0.0),
    });
    let mut interact = InteractiveSolve::new();
    interact.structural_only(&mut doc).unwrap();
    let before = doc.clone_state();
    let mut drag = DragSession::new();
    let free = Segment::end_expr(s1);
    drag.pointer_down(&doc, free);
    drag.pointer_move(&mut interact, &mut doc, (2.0, 2.0))
        .unwrap();
    drag.pointer_up(&mut interact, &mut doc, (2.0, 2.0))
        .unwrap();
    assert!(drag.undo(&mut doc));
    assert_eq!(doc.state(), &before);
}
