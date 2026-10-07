use super::*;
use crate::assemble::AssembledSystem;
use crate::constraint_kinds::{Congruence, FixPoint};
use crate::document::Document;
use crate::geom::Point2;
use crate::geometry_kinds::Segment;
use crate::geometry::{EndpointExprs, GeometryId, GeometryRole};

#[test]
fn stub_backend_swaps() {
    let mut doc = Document::new();
    doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)));
    let soft: [crate::assemble::SoftContribution; 0] = [];
    let sys = AssembledSystem::from_document(&doc, &soft);
    let mut p = doc.clone_vector();
    let opts = SolverOptions::default();
    assert!(matches!(
        StubSolver { succeed: true }.solve(&sys, &mut p, &opts),
        SolveOutcome::Success { .. }
    ));
    assert!(matches!(
        StubSolver { succeed: false }.solve(&sys, &mut p, &opts),
        SolveOutcome::Failure { .. }
    ));
}

#[test]
fn lm_solves_fixpoint_analytically() {
    let mut doc = Document::new();
    let s0 = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(3.0, 4.0), Point2::new(5.0, 6.0)));
    doc.add_constraint(FixPoint {
        point: Segment::start_expr(s0),
        target: Point2::new(0.0, 0.0),
    });
    let soft: [crate::assemble::SoftContribution; 0] = [];
    let sys = AssembledSystem::from_document(&doc, &soft);
    let mut p = doc.clone_vector();
    let out = LmSolver.solve(&sys, &mut p, &SolverOptions::default());
    assert!(matches!(out, SolveOutcome::Success { .. }), "{out:?}");
    doc.restore_vector(&p);
    let q = Segment::start_expr(s0).evaluate(&doc, doc.vector());
    assert!(q.x.abs() < 1e-5 && q.y.abs() < 1e-5);
}

#[test]
fn lm_solves_congruence() {
    let mut doc = Document::new();
    let s0 = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)));
    let s1 = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(2.0, 1.0), Point2::new(3.0, 1.0)));
    doc.add_constraint(Congruence {
        a: Segment::end_expr(s0),
        b: Segment::start_expr(s1),
    });
    doc.add_constraint(FixPoint {
        point: Segment::start_expr(s0),
        target: Point2::new(0.0, 0.0),
    });
    doc.add_constraint(FixPoint {
        point: Segment::end_expr(s0),
        target: Point2::new(1.0, 0.0),
    });
    let soft: [crate::assemble::SoftContribution; 0] = [];
    let sys = AssembledSystem::from_document(&doc, &soft);
    let mut p = doc.clone_vector();
    let out = LmSolver.solve(&sys, &mut p, &SolverOptions::default());
    assert!(matches!(out, SolveOutcome::Success { .. }), "{out:?}");
    doc.restore_vector(&p);
    let a = Segment::end_expr(s0).evaluate(&doc, doc.vector());
    let b = Segment::start_expr(GeometryId(1)).evaluate(&doc, doc.vector());
    assert!((a.x - b.x).abs() < 1e-5 && (a.y - b.y).abs() < 1e-5);
}
