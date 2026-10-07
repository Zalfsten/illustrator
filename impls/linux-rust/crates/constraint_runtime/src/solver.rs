//! Swappable nonlinear solver port over residual/Jacobian evaluation.

use crate::matrix::Matrix;
use crate::vector::Vector;
use crate::SOLVE_TOL;

#[derive(Clone, Debug, PartialEq)]
pub enum SolveOutcome {
    Success { residual_norm: f64 },
    Failure { residual_norm: f64, message: String },
}

#[derive(Clone, Debug)]
pub struct SolverOptions {
    pub tol: f64,
    pub max_iters: usize,
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

/// Evaluation contract: residual and dense Jacobian at `p`.
pub trait SystemEval {
    fn n_params(&self) -> usize;
    fn n_residuals(&self) -> usize;
    fn residual(&self, p: &Vector, r: &mut Vector);
    fn jacobian(&self, p: &Vector, j: &mut Matrix);
}

pub trait SolverPort {
    fn solve(
        &mut self,
        eval: &dyn SystemEval,
        p: &mut Vector,
        opts: &SolverOptions,
    ) -> SolveOutcome;
}

#[derive(Clone, Debug, Default)]
pub struct StubSolver {
    pub succeed: bool,
}

impl SolverPort for StubSolver {
    fn solve(
        &mut self,
        eval: &dyn SystemEval,
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

#[derive(Clone, Debug, Default)]
pub struct LmSolver;

impl SolverPort for LmSolver {
    fn solve(
        &mut self,
        eval: &dyn SystemEval,
        p: &mut Vector,
        opts: &SolverOptions,
    ) -> SolveOutcome {
        let n = eval.n_params();
        let m = eval.n_residuals();
        if n == 0 {
            return SolveOutcome::Success {
                residual_norm: 0.0,
            };
        }
        if m == 0 {
            return SolveOutcome::Success {
                residual_norm: 0.0,
            };
        }

        let mut lambda = opts.lm_lambda0;
        let mut best_p = p.clone();
        let mut best_norm = {
            let mut r = Vector::zeros(m);
            eval.residual(p, &mut r);
            r.iter().map(|x| x * x).sum::<f64>().sqrt()
        };
        let mut stagnant = 0usize;

        for _ in 0..opts.max_iters {
            let mut r = Vector::zeros(m);
            eval.residual(p, &mut r);
            let norm = r.iter().map(|x| x * x).sum::<f64>().sqrt();
            if norm < best_norm {
                best_norm = norm;
                best_p.copy_from(p);
            }
            if norm <= opts.tol {
                return SolveOutcome::Success {
                    residual_norm: norm,
                };
            }

            let mut j = Matrix::zeros(m, n);
            eval.jacobian(p, &mut j);
            let j = j.as_slice();

            // (J^T J + lambda I) dp = J^T r
            let mut jtj = vec![0.0; n * n];
            let mut jtr = vec![0.0; n];
            for i in 0..n {
                for k in 0..n {
                    let mut s = 0.0;
                    for row in 0..m {
                        s += j[row * n + i] * j[row * n + k];
                    }
                    jtj[i * n + k] = s;
                }
                jtj[i * n + i] += lambda;
                let mut s = 0.0;
                for row in 0..m {
                    s += j[row * n + i] * r[row];
                }
                jtr[i] = s;
            }

            let Some(dp) = solve_linear(&jtj, &jtr, n) else {
                lambda *= 10.0;
                stagnant += 1;
                continue;
            };

            let mut p_trial = p.clone();
            for i in 0..n {
                p_trial[i] -= dp[i];
            }
            let mut r_trial = Vector::zeros(m);
            eval.residual(&p_trial, &mut r_trial);
            let trial_norm = r_trial.iter().map(|x| x * x).sum::<f64>().sqrt();

            if trial_norm < norm {
                p.copy_from(&p_trial);
                lambda = (lambda * 0.5).max(1e-12);
                stagnant = 0;
            } else {
                lambda *= 4.0;
                stagnant += 1;
                if lambda > 1e12 || stagnant > 40 {
                    break;
                }
            }
        }

        p.copy_from(&best_p);
        let mut r = Vector::zeros(m);
        eval.residual(p, &mut r);
        let norm = r.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm <= opts.tol {
            SolveOutcome::Success {
                residual_norm: norm,
            }
        } else {
            SolveOutcome::Failure {
                residual_norm: norm,
                message: format!("LM did not reach tolerance (norm={norm})"),
            }
        }
    }
}

fn solve_linear(a_in: &[f64], b_in: &[f64], n: usize) -> Option<Vec<f64>> {
    let mut a = a_in.to_vec();
    let mut b = b_in.to_vec();
    for i in 0..n {
        let mut piv = i;
        for r in i..n {
            if a[r * n + i].abs() > a[piv * n + i].abs() {
                piv = r;
            }
        }
        if a[piv * n + i].abs() < 1e-14 {
            return None;
        }
        if piv != i {
            for c in 0..n {
                a.swap(i * n + c, piv * n + c);
            }
            b.swap(i, piv);
        }
        let diag = a[i * n + i];
        for r in (i + 1)..n {
            let f = a[r * n + i] / diag;
            for c in i..n {
                a[r * n + c] -= f * a[i * n + c];
            }
            b[r] -= f * b[i];
        }
    }
    let mut x = vec![0.0; n];
    for i in (0..n).rev() {
        let mut s = b[i];
        for c in (i + 1)..n {
            s -= a[i * n + c] * x[c];
        }
        x[i] = s / a[i * n + i];
    }
    Some(x)
}

pub mod ffi;

#[cfg(test)]
#[path = "tests_solver.rs"]
mod tests_solver;
