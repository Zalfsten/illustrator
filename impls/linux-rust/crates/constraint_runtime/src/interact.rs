//! Interactive solve: soft drag, stay-near-previous, commit/fail, atomic drag.

use crate::assemble::{AssembledSystem, SoftContribution, SoftPointFollow};
use crate::ceres_solver::CeresSolver;
use crate::document::Document;
use crate::expr::PointExpr;
use crate::geom::Point2;
use crate::solver::{SolverOptions, SolverPort, SystemEval};
use crate::vector::Vector;

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

/// Soft least-squares weights for interactive drag (mouse follow / stay-near).
///
/// Residual scale is `sqrt(weight)`, relative to unweighted structural residuals.
/// Defaults fold in the soft-vs-hard tradeoff (formerly a separate ~1e-3 assemble
/// scale on top of mouse=40 / stay=0.02): too small vs canvas coords (~1e2) makes
/// the pointer numerically invisible; too large lets soft fight hard constraints.
#[derive(Clone, Debug)]
pub struct SoftTerms {
    pub mouse_weight: f64,
    pub stay_weight: f64,
}

impl Default for SoftTerms {
    fn default() -> Self {
        // 40 * (1e-3)^2 and 0.02 * (1e-3)^2 — same effective scale as before.
        Self {
            mouse_weight: 4e-5,
            stay_weight: 2e-8,
        }
    }
}

struct SoftDrag {
    params: Vec<SoftContribution>,
    points: Vec<SoftPointFollow>,
}

fn soft_for_drag(
    doc: &Document,
    prev: &Vector,
    dragged: &PointExpr,
    mouse: (f64, f64),
    weights: &SoftTerms,
) -> SoftDrag {
    // Indices pulled by the mouse must not also get stay-near — otherwise stay
    // fights the pointer and traps the drag in the previous nullspace (e.g. only
    // vertical motion on an axis-aligned right angle).
    let mut mouse_exprs = vec![*dragged];
    mouse_exprs.extend(doc.points_identified_with(dragged));
    let mut mouse_indices = std::collections::BTreeSet::new();
    for expr in &mouse_exprs {
        for &i in &expr.param_indices(doc) {
            mouse_indices.insert(i);
        }
    }

    let mut params = Vec::new();
    for (i, &v) in prev.iter().enumerate() {
        if mouse_indices.contains(&i) {
            continue;
        }
        params.push(SoftContribution {
            index: i,
            target: v,
            weight: weights.stay_weight,
        });
    }
    // Follow the opaque point map (not formula-local slots 0/1 — those are only
    // world xy for segment-style endpoint atoms, not for arc endpoints).
    let points = vec![SoftPointFollow {
        point: *dragged,
        target: Point2::new(mouse.0, mouse.1),
        weight: weights.mouse_weight,
    }];
    SoftDrag { params, points }
}

pub struct InteractiveSolve<S: SolverPort = CeresSolver> {
    pub solver: S,
    pub soft_weights: SoftTerms,
    pub opts: SolverOptions,
}

impl InteractiveSolve<CeresSolver> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for InteractiveSolve<CeresSolver> {
    fn default() -> Self {
        Self {
            solver: CeresSolver,
            soft_weights: SoftTerms::default(),
            opts: SolverOptions::default(),
        }
    }
}

impl<S: SolverPort> InteractiveSolve<S> {
    fn structural_norm(doc: &Document, p: &Vector) -> f64 {
        let soft0: [SoftContribution; 0] = [];
        let sys0 = AssembledSystem::new(doc, &soft0);
        let mut r = Vector::zeros(sys0.n_residuals());
        sys0.residual(p, &mut r);
        r.iter().map(|x| x * x).sum::<f64>().sqrt()
    }

    fn structural_ok(&self, doc: &Document, p: &Vector) -> bool {
        Self::structural_norm(doc, p) <= self.opts.tol * 10.0
    }

    /// Solve on a working copy of Vector; commit only if structural residuals pass.
    pub fn solve_commit(
        &mut self,
        doc: &mut Document,
        soft: &[SoftContribution],
        fallback: &Vector,
    ) -> Result<(), SolveError> {
        self.solve_commit_soft(doc, soft, &[], fallback)
    }

    pub fn solve_commit_soft(
        &mut self,
        doc: &mut Document,
        soft: &[SoftContribution],
        soft_points: &[SoftPointFollow],
        fallback: &Vector,
    ) -> Result<(), SolveError> {
        let mut working = doc.clone_vector();

        // Phase 1: soft + hard (when soft is empty, this is structural-only).
        let sys = AssembledSystem::with_soft(doc, soft, soft_points);
        let _ = self.solver.solve(&sys, &mut working, &self.opts);

        if self.structural_ok(doc, &working) {
            doc.commit_vector(&working);
            return Ok(());
        }

        // Phase 2: polish onto the constraint manifold from the soft attempt.
        if !soft.is_empty() || !soft_points.is_empty() {
            let soft0: [SoftContribution; 0] = [];
            let sys0 = AssembledSystem::new(doc, &soft0);
            let _ = self.solver.solve(&sys0, &mut working, &self.opts);
            if self.structural_ok(doc, &working) {
                doc.commit_vector(&working);
                return Ok(());
            }
        }

        let s_norm = Self::structural_norm(doc, &working);
        doc.restore_vector(fallback);
        Err(SolveError {
            message: format!(
                "structural residuals not satisfied after soft solve + polish (structural={s_norm})"
            ),
            residual_norm: s_norm,
        })
    }

    pub fn structural_only(&mut self, doc: &mut Document) -> Result<(), SolveError> {
        let fallback = doc.clone_vector();
        let soft: [SoftContribution; 0] = [];
        self.solve_commit(doc, &soft, &fallback)
    }
}

#[derive(Clone, Debug)]
pub struct DragSession {
    snapshot: Vector,
    undo_stack: Vec<Vector>,
    active: Option<PointExpr>,
}

impl DragSession {
    pub fn new() -> Self {
        Self {
            snapshot: Vector::new(),
            undo_stack: Vec::new(),
            active: None,
        }
    }

    pub fn pointer_down(&mut self, doc: &Document, point: PointExpr) {
        self.snapshot = doc.clone_vector();
        self.active = Some(point);
    }

    pub fn pointer_move<S: SolverPort>(
        &self,
        interact: &mut InteractiveSolve<S>,
        doc: &mut Document,
        mouse: (f64, f64),
    ) -> Result<(), SolveError> {
        let Some(point) = self.active.as_ref() else {
            return Ok(());
        };
        let soft = soft_for_drag(doc, &self.snapshot, point, mouse, &interact.soft_weights);
        let fallback = doc.clone_vector();
        interact.solve_commit_soft(doc, &soft.params, &soft.points, &fallback)
    }

    pub fn pointer_up<S: SolverPort>(
        &mut self,
        interact: &mut InteractiveSolve<S>,
        doc: &mut Document,
        mouse: (f64, f64),
    ) -> Result<(), SolveError> {
        let Some(point) = self.active.as_ref() else {
            return Ok(());
        };
        let soft = soft_for_drag(doc, &self.snapshot, point, mouse, &interact.soft_weights);
        let fallback = self.snapshot.clone();
        let result = interact.solve_commit_soft(doc, &soft.params, &soft.points, &fallback);
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
            doc.restore_vector(&prev);
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
            doc.restore_vector(&self.snapshot);
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
#[path = "tests_interact.rs"]
mod tests_interact;
