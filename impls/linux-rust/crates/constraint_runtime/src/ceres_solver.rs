//! Ceres SolverPort adapter (dense NLS via C++ bridge).

use crate::solver::ffi::{jacobian_trampoline, residual_trampoline, EvalHandle};
use crate::solver::{SolveOutcome, SolverOptions, SolverPort, SystemEval};
use crate::vector::Vector;
use std::os::raw::c_void;

#[repr(C)]
struct CrCeresOptions {
    function_tolerance: f64,
    max_iterations: i32,
}

extern "C" {
    fn cr_ceres_solve(
        n_params: usize,
        p: *mut f64,
        n_residuals: usize,
        user: *mut c_void,
        residual: unsafe extern "C" fn(
            *mut c_void,
            usize,
            *const f64,
            usize,
            *mut f64,
        ) -> i32,
        jacobian: unsafe extern "C" fn(
            *mut c_void,
            usize,
            *const f64,
            usize,
            *mut f64,
        ) -> i32,
        opts: *const CrCeresOptions,
        err_msg: *mut u8,
        err_msg_len: usize,
    ) -> i32;
}

/// External Ceres backend (dense reference). `SolverOptions::lm_lambda0` is
/// ignored (LmSolver-only). Success iff the final residual norm is within
/// `SolverOptions::tol`; otherwise Failure with Ceres' summary message.
#[derive(Clone, Debug, Default)]
pub struct CeresSolver;

impl SolverPort for CeresSolver {
    fn solve<E: SystemEval>(
        &mut self,
        eval: &E,
        p: &mut Vector,
        opts: &SolverOptions,
    ) -> SolveOutcome {
        let n = eval.n_params();
        let m = eval.n_residuals();
        if n == 0 || m == 0 {
            return SolveOutcome::Success {
                residual_norm: 0.0,
            };
        }
        assert_eq!(p.len(), n);

        let mut handle = EvalHandle {
            eval: eval as &dyn SystemEval,
            state: p as *mut Vector,
            residual_buf: Vector::zeros(m),
            jacobian_buf: eval.new_jacobian(),
        };
        let c_opts = CrCeresOptions {
            function_tolerance: opts.tol,
            max_iterations: opts.max_iters as i32,
        };
        let mut err = vec![0u8; 512];
        let rc = unsafe {
            cr_ceres_solve(
                n,
                p.as_mut_ptr(),
                m,
                &mut handle as *mut EvalHandle<'_> as *mut c_void,
                residual_trampoline,
                jacobian_trampoline,
                &c_opts,
                err.as_mut_ptr(),
                err.len(),
            )
        };

        let mut r = Vector::zeros(m);
        eval.residual(p, &mut r);
        let norm = r.iter().map(|x| x * x).sum::<f64>().sqrt();
        let message = String::from_utf8_lossy(&err)
            .trim_end_matches('\0')
            .trim()
            .to_string();

        // Success iff the configured tolerance is met (solver-port contract).
        // A usable but unconverged Ceres result is still written back to `p`;
        // Interact gates commits on structural residuals, not on this outcome.
        if rc >= 0 && norm <= opts.tol {
            SolveOutcome::Success {
                residual_norm: norm,
            }
        } else {
            let reason = if rc < 0 {
                "Ceres solve failed"
            } else {
                "Ceres did not reach tolerance"
            };
            SolveOutcome::Failure {
                residual_norm: norm,
                message: if message.is_empty() {
                    format!("{reason} (rc={rc}, norm={norm})")
                } else {
                    format!("{reason}: {message} (rc={rc}, norm={norm})")
                },
            }
        }
    }
}

#[cfg(test)]
#[path = "tests_ceres_solver.rs"]
mod tests_ceres_solver;
