//! Owned sparse LM vs the Ceres reference backend on the same scenarios
//! (only with `--features ceres`).
#![cfg(feature = "ceres")]

mod common;

use common::staircase::staircase;
use constraint_runtime::{
    structural_residual_norm, CeresSolver, Congruence, CurveExprs, CurveParam, Document,
    DragSession, EndpointExprs, FixPoint, GeometryRole, InteractiveSolve, LmSolver, Orthogonal,
    Point2, Segment, SoftTerms, SolverOptions, SolverPort, SOLVE_TOL,
};

fn interactive<S: SolverPort>(solver: S) -> InteractiveSolve<S> {
    InteractiveSolve {
        solver,
        soft_weights: SoftTerms::default(),
        opts: SolverOptions::default(),
    }
}

fn assert_structural(doc: &Document, what: &str) {
    let norm = structural_residual_norm(doc);
    assert!(norm <= SOLVE_TOL * 10.0, "{what}: structural norm {norm}");
}

fn right_angle() -> (Document, constraint_runtime::PointExpr) {
    let mut doc = Document::new();
    let s0 = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)));
    let s1 = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(1.0, 0.0), Point2::new(1.0, 1.0)));
    doc.add_constraint(Congruence { a: Segment::end_expr(s0), b: Segment::start_expr(s1) });
    doc.add_constraint(Orthogonal {
        a: Segment::tangent_expr(s0, CurveParam::Normalized(0.0)),
        b: Segment::tangent_expr(s1, CurveParam::Normalized(0.0)),
    });
    doc.add_constraint(FixPoint { point: Segment::start_expr(s0), target: Point2::new(0.0, 0.0) });
    (doc, Segment::end_expr(s1))
}

/// Structural solve, then one drag step of `free` towards `mouse`.
fn run<S: SolverPort>(solver: S, mut doc: Document, free: constraint_runtime::PointExpr, mouse: (f64, f64), what: &str) {
    let mut interact = interactive(solver);
    interact.structural_only(&mut doc).unwrap_or_else(|e| panic!("{what}: {e:?}"));
    assert_structural(&doc, what);
    let mut drag = DragSession::new();
    drag.pointer_down(&doc, free);
    drag.pointer_move(&mut interact, &mut doc, mouse)
        .unwrap_or_else(|e| panic!("{what} drag: {e:?}"));
    drag.pointer_up(&mut interact, &mut doc, mouse)
        .unwrap_or_else(|e| panic!("{what} drag up: {e:?}"));
    assert_structural(&doc, what);
}

#[test]
fn right_angle_drag_matches_reference() {
    let (doc, free) = right_angle();
    run(LmSolver, doc, free, (0.2, 1.5), "lm");
    let (doc, free) = right_angle();
    run(CeresSolver, doc, free, (0.2, 1.5), "ceres");
}

#[test]
fn staircase_drag_matches_reference() {
    for n in [10, 100] {
        let make = || {
            let (doc, ids) = staircase(n);
            let free = Segment::end_expr(*ids.last().unwrap());
            let q = free.evaluate(&doc, doc.vector());
            (doc, free, (q.x + 0.3, q.y + 0.2))
        };
        let (doc, free, mouse) = make();
        run(LmSolver, doc, free, mouse, &format!("lm/{n}"));
        let (doc, free, mouse) = make();
        run(CeresSolver, doc, free, mouse, &format!("ceres/{n}"));
    }
}
