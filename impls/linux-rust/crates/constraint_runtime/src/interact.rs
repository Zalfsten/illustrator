//! Interactive solve: soft drag, stay-near-previous, commit/fail, atomic drag.

use crate::assemble::{AssembledSystem, SoftContribution};
use crate::document::Document;
use crate::geom::PointExpr;
use crate::solver::{LmSolver, SolveOutcome, SolverOptions, SolverPort, SystemEval};

#[derive(Clone, Debug, PartialEq)]
pub struct SolveError {
    pub message: String,
    pub residual_norm: f64,
}

impl std::fmt::Display for SolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "solve failed (norm={}): {}",
            self.residual_norm, self.message
        )
    }
}

impl std::error::Error for SolveError {}

#[derive(Clone, Debug)]
pub struct SoftTerms {
    pub mouse_weight: f64,
    pub stay_weight: f64,
}

impl Default for SoftTerms {
    fn default() -> Self {
        Self {
            mouse_weight: 40.0,
            stay_weight: 0.05,
        }
    }
}

fn soft_for_drag(
    doc: &Document,
    prev: &[f64],
    dragged: PointExpr,
    mouse: (f64, f64),
    weights: &SoftTerms,
) -> Vec<SoftContribution> {
    let mut soft = Vec::new();
    for (i, &v) in prev.iter().enumerate() {
        soft.push(SoftContribution {
            index: i,
            target: v,
            weight: weights.stay_weight,
        });
    }
    let idxs = dragged.param_indices(doc.segments());
    soft.push(SoftContribution {
        index: idxs[0],
        target: mouse.0,
        weight: weights.mouse_weight,
    });
    soft.push(SoftContribution {
        index: idxs[1],
        target: mouse.1,
        weight: weights.mouse_weight,
    });
    for c in doc.constraints() {
        if let crate::constraint::Constraint::Congruence { a, b } = c {
            let other = if *a == dragged {
                *b
            } else if *b == dragged {
                *a
            } else {
                continue;
            };
            let oi = other.param_indices(doc.segments());
            soft.push(SoftContribution {
                index: oi[0],
                target: mouse.0,
                weight: weights.mouse_weight,
            });
            soft.push(SoftContribution {
                index: oi[1],
                target: mouse.1,
                weight: weights.mouse_weight,
            });
        }
    }
    soft
}

/// Structural residual rows only (exclude soft) for success checks.
#[allow(dead_code)]
fn structural_norm_at(doc: &Document, p: &[f64]) -> f64 {
    let soft: [SoftContribution; 0] = [];
    let sys = AssembledSystem::from_document(doc, &soft);
    let mut r = vec![0.0; sys.n_residuals()];
    sys.residual(p, &mut r);
    r.iter().map(|x| x * x).sum::<f64>().sqrt()
}

pub struct InteractiveSolve<S: SolverPort = LmSolver> {
    pub solver: S,
    pub soft_weights: SoftTerms,
    pub opts: SolverOptions,
}

impl Default for InteractiveSolve<LmSolver> {
    fn default() -> Self {
        Self {
            solver: LmSolver,
            soft_weights: SoftTerms::default(),
            opts: SolverOptions::default(),
        }
    }
}

impl<S: SolverPort> InteractiveSolve<S> {
    pub fn solve_commit(
        &mut self,
        doc: &mut Document,
        soft: &[SoftContribution],
        fallback: &[f64],
    ) -> Result<(), SolveError> {
        let sys = AssembledSystem::from_document(doc, soft);
        let mut p = doc.state().to_vec();
        let n_struct: usize = doc.constraints().iter().map(|c| c.residual_arity()).sum();

        // Custom eval that reports structural success using full system for steps
        // but we check structural norm after.
        let outcome = self.solver.solve(&sys, &mut p, &self.opts);
        match outcome {
            SolveOutcome::Success { .. } | SolveOutcome::Failure { .. } => {
                // Prefer accepting when structural residuals are satisfied even if soft remains.
                let s_norm = {
                    let soft0: [SoftContribution; 0] = [];
                    let sys0 = AssembledSystem::from_document(doc, &soft0);
                    let mut r = vec![0.0; sys0.n_residuals()];
                    sys0.residual(&p, &mut r);
                    r[..n_struct.min(r.len())]
                        .iter()
                        .map(|x| x * x)
                        .sum::<f64>()
                        .sqrt()
                };
                if s_norm <= self.opts.tol {
                    doc.restore_state(&p);
                    Ok(())
                } else {
                    doc.restore_state(fallback);
                    let message = match outcome {
                        SolveOutcome::Failure { message, .. } => message,
                        SolveOutcome::Success { .. } => {
                            "structural residuals not satisfied".into()
                        }
                    };
                    Err(SolveError {
                        message,
                        residual_norm: s_norm,
                    })
                }
            }
        }
    }

    pub fn structural_only(&mut self, doc: &mut Document) -> Result<(), SolveError> {
        let fallback = doc.clone_state();
        let soft: [SoftContribution; 0] = [];
        self.solve_commit(doc, &soft, &fallback)
    }
}

#[derive(Clone, Debug)]
pub struct DragSession {
    snapshot: Vec<f64>,
    undo_stack: Vec<Vec<f64>>,
    active: Option<PointExpr>,
}

impl DragSession {
    pub fn new() -> Self {
        Self {
            snapshot: Vec::new(),
            undo_stack: Vec::new(),
            active: None,
        }
    }

    pub fn pointer_down(&mut self, doc: &Document, point: PointExpr) {
        self.snapshot = doc.clone_state();
        self.active = Some(point);
    }

    pub fn pointer_move<S: SolverPort>(
        &self,
        interact: &mut InteractiveSolve<S>,
        doc: &mut Document,
        mouse: (f64, f64),
    ) -> Result<(), SolveError> {
        let Some(point) = self.active else {
            return Ok(());
        };
        let soft = soft_for_drag(doc, &self.snapshot, point, mouse, &interact.soft_weights);
        let fallback = doc.clone_state();
        interact.solve_commit(doc, &soft, &fallback)
    }

    pub fn pointer_up<S: SolverPort>(
        &mut self,
        interact: &mut InteractiveSolve<S>,
        doc: &mut Document,
        mouse: (f64, f64),
    ) -> Result<(), SolveError> {
        let Some(point) = self.active else {
            return Ok(());
        };
        let soft = soft_for_drag(doc, &self.snapshot, point, mouse, &interact.soft_weights);
        let fallback = self.snapshot.clone();
        let result = interact.solve_commit(doc, &soft, &fallback);
        self.active = None;
        match result {
            Ok(()) => {
                self.undo_stack.push(self.snapshot.clone());
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    pub fn undo(&mut self, doc: &mut Document) -> bool {
        if let Some(prev) = self.undo_stack.pop() {
            doc.restore_state(&prev);
            true
        } else {
            false
        }
    }

    pub fn is_dragging(&self) -> bool {
        self.active.is_some()
    }

    pub fn cancel(&mut self, doc: &mut Document) {
        if self.active.is_some() {
            doc.restore_state(&self.snapshot);
            self.active = None;
        }
    }
}

impl Default for DragSession {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
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
}
