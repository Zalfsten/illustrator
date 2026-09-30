//! Ceres SolverPort adapter (dense NLS via C++ bridge).

use crate::solver::ffi::{jacobian_trampoline, residual_trampoline, EvalHandle};
use crate::solver::{SolveOutcome, SolverOptions, SolverPort, SystemEval};
use crate::state::State;
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

/// External Ceres backend. `SolverOptions::lm_lambda0` is ignored (LmSolver-only).
#[derive(Clone, Debug, Default)]
pub struct CeresSolver;

impl SolverPort for CeresSolver {
    fn solve(
        &mut self,
        eval: &dyn SystemEval,
        p: &mut State,
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

        let handle = EvalHandle {
            eval,
            state: p as *mut State,
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
                &handle as *const EvalHandle<'_> as *mut c_void,
                residual_trampoline,
                jacobian_trampoline,
                &c_opts,
                err.as_mut_ptr(),
                err.len(),
            )
        };

        let mut r = vec![0.0; m];
        eval.residual(p, &mut r);
        let norm = r.iter().map(|x| x * x).sum::<f64>().sqrt();
        let message = String::from_utf8_lossy(&err)
            .trim_end_matches('\0')
            .trim()
            .to_string();

        // Soft + hard residuals are often not driven to opts.tol together (soft is
        // best-effort). A usable Ceres step is Success; Interact gates on structural.
        if rc < 0 {
            SolveOutcome::Failure {
                residual_norm: norm,
                message: if message.is_empty() {
                    format!("Ceres solve failed (rc={rc}, norm={norm})")
                } else {
                    message
                },
            }
        } else {
            let _ = message;
            SolveOutcome::Success {
                residual_norm: norm,
            }
        }
    }
}

#[cfg(test)]
#[path = "tests_ceres_solver.rs"]
mod tests_ceres_solver;
