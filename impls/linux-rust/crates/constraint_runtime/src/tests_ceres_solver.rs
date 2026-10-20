use super::*;
use crate::assemble::AssembledSystem;
use crate::constraint_kinds::FixPoint;
use crate::document::Document;
use crate::geom::Point2;
use crate::geometry_kinds::Segment;
use crate::geometry::{EndpointExprs, GeometryRole};
use crate::SOLVE_TOL;

#[test]
fn ceres_solves_fixpoint() {
    let mut doc = Document::new();
    let s0 = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(3.0, 4.0), Point2::new(5.0, 6.0)));
    doc.add_constraint(FixPoint {
        point: Segment::start_expr(s0),
        target: Point2::new(0.0, 0.0),
    });
    let soft: [crate::assemble::SoftContribution; 0] = [];
    let sys = AssembledSystem::from_document(&doc, &soft);
    let mut p = doc.clone_vector();
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

#[test]
fn ceres_reports_failure_when_tolerance_is_not_met() {
    // Two FixPoints on one point with different targets: Ceres converges to the
    // least-squares optimum, but the residual norm stays √2 > tol.
    let mut doc = Document::new();
    let s0 = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(5.0, 3.0), Point2::new(6.0, 3.0)));
    for x in [0.0, 2.0] {
        doc.add_constraint(FixPoint {
            point: Segment::start_expr(s0),
            target: Point2::new(x, 0.0),
        });
    }
    let soft: [crate::assemble::SoftContribution; 0] = [];
    let sys = AssembledSystem::from_document(&doc, &soft);
    let mut p = doc.clone_vector();
    let out = CeresSolver.solve(&sys, &mut p, &SolverOptions::default());
    let SolveOutcome::Failure { residual_norm, message } = out else {
        panic!("expected Failure, got {out:?}");
    };
    assert!((residual_norm - 2f64.sqrt()).abs() < 1e-6, "{message}");
    assert!(message.contains("did not reach tolerance"), "{message}");
    // The usable point near the least-squares optimum (1, 0) is still written
    // back (Ceres stops on its relative function tolerance).
    assert!((p[0] - 1.0).abs() < 1e-3 && p[1].abs() < 1e-3, "p={:?}", p.as_slice());
}
