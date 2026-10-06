//! Interactive solve: soft drag, stay-near-previous, commit/fail, atomic drag.

use crate::assemble::{AssembledSystem, SoftContribution};
use crate::ceres_solver::CeresSolver;
use crate::document::Document;
use crate::geometry::PointExpr;
use crate::solver::{SolverOptions, SolverPort, SystemEval};
use crate::state::State;

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

fn soft_for_drag(
    doc: &Document,
    prev: &State,
    dragged: &PointExpr,
    mouse: (f64, f64),
    weights: &SoftTerms,
) -> Vec<SoftContribution> {
    // Indices pulled by the mouse must not also get stay-near — otherwise stay
    // fights the pointer and traps the drag in the previous nullspace (e.g. only
    // vertical motion on an axis-aligned right angle).
    let mut mouse_exprs = vec![dragged.clone()];
    mouse_exprs.extend(doc.points_identified_with(dragged));
    let mut mouse_indices = std::collections::BTreeSet::new();
    for expr in &mouse_exprs {
        for &i in &expr.param_indices(doc) {
            mouse_indices.insert(i);
        }
    }

    let mut soft = Vec::new();
    for (i, &v) in prev.iter().enumerate() {
        if mouse_indices.contains(&i) {
            continue;
        }
        soft.push(SoftContribution {
            index: i,
            target: v,
            weight: weights.stay_weight,
        });
    }
    for expr in &mouse_exprs {
        let ix = expr.param_indices(doc);
        // Soft mouse targets the expression's coordinate slots. Endpoint exprs
        // expose (x,y) as two indices; OnCurve is not a drag target in the UI yet.
        if ix.len() >= 2 {
            soft.push(SoftContribution {
                index: ix[0],
                target: mouse.0,
                weight: weights.mouse_weight,
            });
            soft.push(SoftContribution {
                index: ix[1],
                target: mouse.1,
                weight: weights.mouse_weight,
            });
        }
    }
    soft
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
    fn structural_norm(doc: &Document, p: &State) -> f64 {
        let soft0: [SoftContribution; 0] = [];
        let sys0 = AssembledSystem::new(doc, &soft0);
        let mut r = vec![0.0; sys0.n_residuals()];
        sys0.residual(p, &mut r);
        r.iter().map(|x| x * x).sum::<f64>().sqrt()
    }

    fn structural_ok(&self, doc: &Document, p: &State) -> bool {
        Self::structural_norm(doc, p) <= self.opts.tol * 10.0
    }

    /// Solve on a working copy of State; commit only if structural residuals pass.
    pub fn solve_commit(
        &mut self,
        doc: &mut Document,
        soft: &[SoftContribution],
        fallback: &State,
    ) -> Result<(), SolveError> {
        let mut working = doc.clone_state();

        // Phase 1: soft + hard (when soft is empty, this is structural-only).
        let sys = AssembledSystem::from_document(doc, soft);
        let _ = self.solver.solve(&sys, &mut working, &self.opts);

        if self.structural_ok(doc, &working) {
            doc.commit_state(&working);
            return Ok(());
        }

        // Phase 2: polish onto the constraint manifold from the soft attempt.
        if !soft.is_empty() {
            let soft0: [SoftContribution; 0] = [];
            let sys0 = AssembledSystem::new(doc, &soft0);
            let _ = self.solver.solve(&sys0, &mut working, &self.opts);
            if self.structural_ok(doc, &working) {
                doc.commit_state(&working);
                return Ok(());
            }
        }

        let s_norm = Self::structural_norm(doc, &working);
        doc.restore_state(fallback);
        Err(SolveError {
            message: format!(
                "structural residuals not satisfied after soft solve + polish (structural={s_norm})"
            ),
            residual_norm: s_norm,
        })
    }

    pub fn structural_only(&mut self, doc: &mut Document) -> Result<(), SolveError> {
        let fallback = doc.clone_state();
        let soft: [SoftContribution; 0] = [];
        self.solve_commit(doc, &soft, &fallback)
    }
}

#[derive(Clone, Debug)]
pub struct DragSession {
    snapshot: State,
    undo_stack: Vec<State>,
    active: Option<PointExpr>,
}

impl DragSession {
    pub fn new() -> Self {
        Self {
            snapshot: State::new(),
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
        let Some(point) = self.active.as_ref() else {
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
        let Some(point) = self.active.as_ref() else {
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
#[path = "tests_interact.rs"]
mod tests_interact;
