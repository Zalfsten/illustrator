use super::*;
use crate::assemble::AssembledSystem;
use crate::constraint::Constraint;
use crate::document::Document;
use crate::geom::{Point2, PointExpr, SegmentEndKind};
use crate::SOLVE_TOL;

#[test]
fn ceres_solves_fixpoint() {
    let mut doc = Document::new();
    let s0 = doc.add_segment(Point2::new(3.0, 4.0), Point2::new(5.0, 6.0));
    doc.add_constraint(Constraint::FixPoint {
        point: PointExpr::SegmentEnd {
            segment: s0,
            end: SegmentEndKind::Start,
        },
        target: Point2::new(0.0, 0.0),
    });
    let soft: [crate::assemble::SoftContribution; 0] = [];
    let sys = AssembledSystem::from_document(&doc, &soft);
    let mut p = doc.state().to_vec();
    let mut solver = CeresSolver;
    let outcome = solver.solve(
        &sys,
        &mut p,
        &SolverOptions {
            tol: SOLVE_TOL,
            ..Default::default()
        },
    );
    match outcome {
        SolveOutcome::Success { residual_norm } => {
            assert!(residual_norm <= SOLVE_TOL * 10.0);
            assert!((p[0] - 0.0).abs() < 1e-4);
            assert!((p[1] - 0.0).abs() < 1e-4);
        }
        SolveOutcome::Failure {
            message,
            residual_norm,
        } => panic!("ceres failed: {message} norm={residual_norm}"),
    }
}
