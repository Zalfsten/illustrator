//! Swappable nonlinear solver port over residual/Jacobian evaluation.

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

/// Evaluation contract: residual and dense row-major Jacobian at `p`.
pub trait SystemEval {
    fn n_params(&self) -> usize;
    fn n_residuals(&self) -> usize;
    fn residual(&self, p: &[f64], r: &mut [f64]);
    fn jacobian(&self, p: &[f64], j: &mut [f64]);
}

pub trait SolverPort {
    fn solve(
        &mut self,
        eval: &dyn SystemEval,
        p: &mut [f64],
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
        p: &mut [f64],
        opts: &SolverOptions,
    ) -> SolveOutcome {
        let _ = opts;
        let mut r = vec![0.0; eval.n_residuals()];
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
        p: &mut [f64],
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
        let mut best_p = p.to_vec();
        let mut best_norm = {
            let mut r = vec![0.0; m];
            eval.residual(p, &mut r);
            r.iter().map(|x| x * x).sum::<f64>().sqrt()
        };
        let mut stagnant = 0usize;

        for _ in 0..opts.max_iters {
            let mut r = vec![0.0; m];
            eval.residual(p, &mut r);
            let norm = r.iter().map(|x| x * x).sum::<f64>().sqrt();
            if norm < best_norm {
                best_norm = norm;
                best_p.copy_from_slice(p);
            }
            if norm <= opts.tol {
                return SolveOutcome::Success {
                    residual_norm: norm,
                };
            }

            let mut j = vec![0.0; m * n];
            eval.jacobian(p, &mut j);

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

            let mut p_trial = p.to_vec();
            for i in 0..n {
                p_trial[i] -= dp[i];
            }
            let mut r_trial = vec![0.0; m];
            eval.residual(&p_trial, &mut r_trial);
            let trial_norm = r_trial.iter().map(|x| x * x).sum::<f64>().sqrt();

            if trial_norm < norm {
                p.copy_from_slice(&p_trial);
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

        p.copy_from_slice(&best_p);
        let mut r = vec![0.0; m];
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

/// C-ABI shaped trampolines for foreign solvers (opaque user data + fill buffers).
pub mod ffi {
    use super::SystemEval;
    use std::os::raw::c_void;

    pub struct EvalHandle<'a> {
        pub eval: &'a dyn SystemEval,
    }

    /// `# Safety`: `user` must point to a live [`EvalHandle`].
    pub unsafe extern "C" fn residual_trampoline(
        user: *mut c_void,
        n_params: usize,
        p: *const f64,
        n_res: usize,
        r: *mut f64,
    ) -> i32 {
        if user.is_null() || p.is_null() || r.is_null() {
            return -1;
        }
        let handle = &*(user as *const EvalHandle<'_>);
        if handle.eval.n_params() != n_params || handle.eval.n_residuals() != n_res {
            return -2;
        }
        let p = std::slice::from_raw_parts(p, n_params);
        let r = std::slice::from_raw_parts_mut(r, n_res);
        handle.eval.residual(p, r);
        0
    }

    /// `# Safety`: `user` must point to a live [`EvalHandle`].
    pub unsafe extern "C" fn jacobian_trampoline(
        user: *mut c_void,
        n_params: usize,
        p: *const f64,
        n_res: usize,
        j: *mut f64,
    ) -> i32 {
        if user.is_null() || p.is_null() || j.is_null() {
            return -1;
        }
        let handle = &*(user as *const EvalHandle<'_>);
        if handle.eval.n_params() != n_params || handle.eval.n_residuals() != n_res {
            return -2;
        }
        let p = std::slice::from_raw_parts(p, n_params);
        let j = std::slice::from_raw_parts_mut(j, n_res * n_params);
        handle.eval.jacobian(p, j);
        0
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::assemble::AssembledSystem;
        use crate::constraint::Constraint;
        use crate::document::Document;
        use crate::geom::{Point2, PointExpr, SegmentEndKind};

        #[test]
        fn c_calling_pattern_fills_residual() {
            let mut doc = Document::new();
            let s0 = doc.add_segment(Point2::new(0.0, 0.0), Point2::new(1.0, 0.0));
            doc.add_constraint(Constraint::FixPoint {
                point: PointExpr::SegmentEnd {
                    segment: s0,
                    end: SegmentEndKind::Start,
                },
                target: Point2::new(0.0, 0.0),
            });
            let soft: [crate::assemble::SoftContribution; 0] = [];
            let sys = AssembledSystem::from_document(&doc, &soft);
            let handle = EvalHandle { eval: &sys };
            let p = doc.state().to_vec();
            let mut r = vec![0.0; sys.n_residuals()];
            let rc = unsafe {
                residual_trampoline(
                    &handle as *const _ as *mut c_void,
                    p.len(),
                    p.as_ptr(),
                    r.len(),
                    r.as_mut_ptr(),
                )
            };
            assert_eq!(rc, 0);
            assert!(r.iter().all(|x| x.abs() < 1e-12));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assemble::AssembledSystem;
    use crate::constraint::{Constraint, EndpointRef, SegmentEnd};
    use crate::document::Document;
    use crate::geom::{Point2, PointExpr, SegmentEndKind, SegmentId};

    #[test]
    fn stub_backend_swaps() {
        let mut doc = Document::new();
        doc.add_segment(Point2::new(0.0, 0.0), Point2::new(1.0, 0.0));
        let soft: [crate::assemble::SoftContribution; 0] = [];
        let sys = AssembledSystem::from_document(&doc, &soft);
        let mut p = doc.state().to_vec();
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
        let out = LmSolver.solve(&sys, &mut p, &SolverOptions::default());
        assert!(matches!(out, SolveOutcome::Success { .. }), "{out:?}");
        doc.restore_state(&p);
        let q = doc.segment(s0).unwrap().start(doc.state());
        assert!(q.x.abs() < 1e-5 && q.y.abs() < 1e-5);
    }

    #[test]
    fn lm_solves_congruence() {
        let mut doc = Document::new();
        let s0 = doc.add_segment(Point2::new(0.0, 0.0), Point2::new(1.0, 0.0));
        let s1 = doc.add_segment(Point2::new(2.0, 1.0), Point2::new(3.0, 1.0));
        doc.join_endpoints(
            EndpointRef::new(s0, SegmentEnd::End),
            EndpointRef::new(s1, SegmentEnd::Start),
        );
        doc.add_constraint(Constraint::FixPoint {
            point: PointExpr::SegmentEnd {
                segment: s0,
                end: SegmentEndKind::Start,
            },
            target: Point2::new(0.0, 0.0),
        });
        doc.add_constraint(Constraint::FixPoint {
            point: PointExpr::SegmentEnd {
                segment: s0,
                end: SegmentEndKind::End,
            },
            target: Point2::new(1.0, 0.0),
        });
        let soft: [crate::assemble::SoftContribution; 0] = [];
        let sys = AssembledSystem::from_document(&doc, &soft);
        let mut p = doc.state().to_vec();
        let out = LmSolver.solve(&sys, &mut p, &SolverOptions::default());
        assert!(matches!(out, SolveOutcome::Success { .. }), "{out:?}");
        doc.restore_state(&p);
        let a = doc.segment(s0).unwrap().end(doc.state());
        let b = doc.segment(SegmentId(1)).unwrap().start(doc.state());
        assert!((a.x - b.x).abs() < 1e-5 && (a.y - b.y).abs() < 1e-5);
    }
}
