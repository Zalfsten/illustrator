//! Swappable nonlinear solver port over residual/Jacobian evaluation.

use crate::matrix::BlockSparseMatrix;
use crate::vector::Vector;
use crate::SOLVE_TOL;

#[derive(Clone, Debug, PartialEq)]
pub enum SolveOutcome {
    Success { residual_norm: f64 },
    Failure { residual_norm: f64, message: String },
}

#[derive(Clone, Debug)]
pub struct SolverOptions {
    /// Residual-norm tolerance for Success.
    pub tol: f64,
    pub max_iters: usize,
    /// Initial LM damping, relative to the Marquardt scaling
    /// \(\mathrm{diag}(J^\top J)\) (LM backends only, e.g. [`LmSolver`]).
    pub lm_lambda0: f64,
}

impl Default for SolverOptions {
    fn default() -> Self {
        Self {
            tol: SOLVE_TOL,
            max_iters: 250,
            lm_lambda0: 1e-2,
        }
    }
}

/// Evaluation contract: residual and block-sparse Jacobian at `p`.
///
/// The Jacobian is a [`BlockSparseMatrix`] with one dense row block per
/// contribution. Its block structure is fixed for the evaluator (independent
/// of `p`): [`Self::new_jacobian`] builds it once per solve, then
/// [`Self::jacobian`] refills the values. A dense \(m \times n\) copy (tests,
/// debugging) is available via [`BlockSparseMatrix::add_to_dense`].
pub trait SystemEval {
    fn n_params(&self) -> usize;
    fn n_residuals(&self) -> usize;
    fn residual(&self, p: &Vector, r: &mut Vector);
    /// `n_residuals × n_params` Jacobian with this evaluator's block structure
    /// and zero values.
    fn new_jacobian(&self) -> BlockSparseMatrix;
    /// Overwrite every block of `j` (built by [`Self::new_jacobian`]).
    fn jacobian(&self, p: &Vector, j: &mut BlockSparseMatrix);
}

pub trait SolverPort {
    fn solve<E: SystemEval>(
        &mut self,
        eval: &E,
        p: &mut Vector,
        opts: &SolverOptions,
    ) -> SolveOutcome;
}

#[derive(Clone, Debug, Default)]
pub struct StubSolver {
    pub succeed: bool,
}

impl SolverPort for StubSolver {
    fn solve<E: SystemEval>(
        &mut self,
        eval: &E,
        p: &mut Vector,
        opts: &SolverOptions,
    ) -> SolveOutcome {
        let _ = opts;
        let mut r = Vector::zeros(eval.n_residuals());
        eval.residual(p, &mut r);
        let n = r.iter().map(|x| x * x).sum::<f64>().sqrt();
        if self.succeed {
            SolveOutcome::Success { residual_norm: n }
        } else {
            SolveOutcome::Failure {
                residual_norm: n,
                message: "stub forced failure".into(),
            }
        }
    }
}

/// Lower / upper clamp for the Marquardt scaling \(D = \mathrm{diag}(J^\top J)\).
const LM_DIAG_MIN: f64 = 1e-6;
const LM_DIAG_MAX: f64 = 1e32;
/// Minimum gain ratio for accepting a step.
const LM_ACCEPT_RHO: f64 = 1e-3;
/// Stationarity / step-size thresholds and damping overflow.
const LM_GRAD_TOL: f64 = 1e-12;
const LM_STEP_TOL: f64 = 1e-12;
const LM_LAMBDA_MAX: f64 = 1e16;

/// Owned sparse Levenberg–Marquardt backend (pure Rust).
///
/// Solves \((J^\top J + \lambda D)\,\delta = J^\top r\) with
/// \(D = \mathrm{diag}(J^\top J)\) (clamped) by sparse Cholesky; the normal
/// equations are assembled from the contribution blocks. Steps are judged by
/// the gain ratio and \(\lambda\) follows Nielsen's update. Symbolic
/// factorisation and all scratch are set up once per solve; iterations do not
/// allocate.
///
/// [`SolverOptions::lm_lambda0`] is the initial \(\lambda\) **relative** to
/// \(D\). Success iff the final \(\|r\| \le\) [`SolverOptions::tol`];
/// otherwise Failure with the stop reason. The written-back `p` is always the
/// best iterate (only descending steps are accepted).
#[derive(Clone, Debug, Default)]
pub struct LmSolver;

impl SolverPort for LmSolver {
    fn solve<E: SystemEval>(
        &mut self,
        eval: &E,
        p: &mut Vector,
        opts: &SolverOptions,
    ) -> SolveOutcome {
        let n = eval.n_params();
        let m = eval.n_residuals();
        if n == 0 || m == 0 {
            return SolveOutcome::Success { residual_norm: 0.0 };
        }
        assert_eq!(p.len(), n);

        let mut j = eval.new_jacobian();
        let mut normal = NormalSystem::new(&j);
        let mut llt = SparseLlt::new(
            normal.n(),
            normal.col_ptr().to_vec(),
            normal.row_idx().to_vec(),
        );
        let mut damped = vec![0.0; normal.values().len()];
        let mut scale = vec![0.0; n];
        let mut delta = vec![0.0; n];
        let mut r = Vector::zeros(m);
        let mut r_trial = Vector::zeros(m);
        let mut p_trial = p.clone();

        eval.residual(p, &mut r);
        let mut cost = 0.5 * norm_sq(&r);
        let mut lambda = opts.lm_lambda0;
        let mut nu = 2.0;
        let mut fresh_jacobian = false;
        let mut stop = "iteration limit reached";

        for _ in 0..opts.max_iters {
            if (2.0 * cost).sqrt() <= opts.tol {
                break;
            }
            if !fresh_jacobian {
                eval.jacobian(p, &mut j);
                normal.assemble(&j, &r);
                for (i, d) in scale.iter_mut().enumerate() {
                    *d = normal.values()[normal.diag_slot(i)].clamp(LM_DIAG_MIN, LM_DIAG_MAX);
                }
                fresh_jacobian = true;
                if normal.rhs().iter().all(|g| g.abs() <= LM_GRAD_TOL) {
                    stop = "stationary point (gradient below threshold)";
                    break;
                }
            }

            damped.copy_from_slice(normal.values());
            for (i, d) in scale.iter().enumerate() {
                damped[normal.diag_slot(i)] += lambda * d;
            }
            delta.copy_from_slice(normal.rhs());
            if !llt.factor_solve(&damped, &mut delta) {
                lambda *= nu;
                nu *= 2.0;
                if lambda > LM_LAMBDA_MAX {
                    stop = "damping overflow (factorisation failed)";
                    break;
                }
                continue;
            }

            let step_sq: f64 = delta.iter().map(|d| d * d).sum();
            if step_sq.sqrt() <= LM_STEP_TOL * (norm_sq(p).sqrt() + LM_STEP_TOL) {
                stop = "step below threshold";
                break;
            }
            for i in 0..n {
                p_trial[i] = p[i] - delta[i];
            }
            eval.residual(&p_trial, &mut r_trial);
            let trial_cost = 0.5 * norm_sq(&r_trial);

            // Model decrease for step h = −δ with (G + λD)δ = g:
            // L(0) − L(h) = ½ δᵀ(g + λDδ).
            let predicted: f64 = 0.5
                * (0..n)
                    .map(|i| delta[i] * (normal.rhs()[i] + lambda * scale[i] * delta[i]))
                    .sum::<f64>();
            let rho = (cost - trial_cost) / predicted;
            if predicted > 0.0 && rho > LM_ACCEPT_RHO {
                p.copy_from(&p_trial);
                std::mem::swap(&mut r, &mut r_trial);
                cost = trial_cost;
                lambda *= (1.0 - (2.0 * rho - 1.0).powi(3)).max(1.0 / 3.0);
                nu = 2.0;
                fresh_jacobian = false;
            } else {
                lambda *= nu;
                nu *= 2.0;
                if lambda > LM_LAMBDA_MAX {
                    stop = "damping overflow (no descending step)";
                    break;
                }
            }
        }

        let norm = (2.0 * cost).sqrt();
        if norm <= opts.tol {
            SolveOutcome::Success {
                residual_norm: norm,
            }
        } else {
            SolveOutcome::Failure {
                residual_norm: norm,
                message: format!("LM did not reach tolerance: {stop} (norm={norm})"),
            }
        }
    }
}

fn norm_sq(v: &Vector) -> f64 {
    v.iter().map(|x| x * x).sum()
}

mod normal;
mod sparse_chol;

use normal::NormalSystem;
use sparse_chol::SparseLlt;

#[cfg(test)]
#[path = "tests_solver.rs"]
mod tests_solver;
