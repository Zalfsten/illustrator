use super::*;
use crate::constraint::{Constraint, EndpointRef, SegmentEnd};
use crate::geom::{Point2, SegmentEndKind, SegmentId};
use crate::solver::StubSolver;

#[test]
fn soft_not_persisted_after_drag() {
    let mut doc = Document::new();
    let s0 = doc.add_segment(Point2::new(0.0, 0.0), Point2::new(1.0, 0.0));
    doc.add_constraint(Constraint::FixPoint {
        point: PointExpr::SegmentEnd {
            segment: s0,
            end: SegmentEndKind::Start,
        },
        target: Point2::new(0.0, 0.0),
    });
    let n = doc.constraints().len();
    let mut interact = InteractiveSolve::default();
    let mut drag = DragSession::new();
    let expr = PointExpr::SegmentEnd {
        segment: s0,
        end: SegmentEndKind::End,
    };
    drag.pointer_down(&doc, expr);
    let _ = drag.pointer_move(&mut interact, &mut doc, (1.5, 0.2));
    let _ = drag.pointer_up(&mut interact, &mut doc, (1.5, 0.2));
    assert_eq!(doc.constraints().len(), n);
}

#[test]
fn failed_solve_keeps_state() {
    let mut doc = Document::new();
    let s0 = doc.add_segment(Point2::new(0.0, 0.0), Point2::new(1.0, 0.0));
    doc.add_constraint(Constraint::FixPoint {
        point: PointExpr::SegmentEnd {
            segment: s0,
            end: SegmentEndKind::Start,
        },
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
        point: PointExpr::SegmentEnd {
            segment: s0,
            end: SegmentEndKind::Start,
        },
        target: Point2::new(10.0, 10.0),
    });
    let before = doc.clone_state();
    let soft: [SoftContribution; 0] = [];
    let err = interact
        .solve_commit(&mut doc, &soft, &before)
        .expect_err("must fail");
    assert!(!err.message.is_empty());
    assert_eq!(doc.state(), before.as_slice());
}

#[test]
fn atomic_drag_undo() {
    let mut doc = Document::new();
    let s0 = doc.add_segment(Point2::new(0.0, 0.0), Point2::new(1.0, 0.0));
    let s1 = doc.add_segment(Point2::new(1.0, 0.0), Point2::new(1.0, 1.0));
    doc.join_endpoints(
        EndpointRef::new(s0, SegmentEnd::End),
        EndpointRef::new(s1, SegmentEnd::Start),
    );
    doc.add_constraint(Constraint::Orthogonal {
        a: SegmentId(0),
        b: SegmentId(1),
    });
    doc.add_constraint(Constraint::FixPoint {
        point: PointExpr::SegmentEnd {
            segment: s0,
            end: SegmentEndKind::Start,
        },
        target: Point2::new(0.0, 0.0),
    });
    let mut interact = InteractiveSolve::default();
    interact.structural_only(&mut doc).unwrap();
    let before = doc.clone_state();
    let mut drag = DragSession::new();
    let free = PointExpr::SegmentEnd {
        segment: s1,
        end: SegmentEndKind::End,
    };
    drag.pointer_down(&doc, free);
    drag.pointer_move(&mut interact, &mut doc, (2.0, 2.0))
        .unwrap();
    drag.pointer_up(&mut interact, &mut doc, (2.0, 2.0))
        .unwrap();
    assert!(drag.undo(&mut doc));
    assert_eq!(doc.state(), before.as_slice());
}
