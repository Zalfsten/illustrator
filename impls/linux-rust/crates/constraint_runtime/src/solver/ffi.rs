//! C-ABI shaped trampolines for foreign solvers (opaque user data + fill buffers).
//!
//! Ceres may evaluate at a temporary parameter buffer that is not the caller's
//! [`Vector`] storage. Trampolines sync that buffer into [`EvalHandle::state`]
//! before calling [`SystemEval`] (O(n) only when the pointers differ).
//! Residual / Jacobian scratch buffers live on the handle and are reused across
//! callbacks; results are copied into the Ceres-provided output pointers (the
//! contribution-blocked Jacobian is densified row-major, duplicates summed).

use super::SystemEval;
use crate::matrix::BlockSparseMatrix;
use crate::vector::Vector;
use std::os::raw::c_void;

pub struct EvalHandle<'a> {
    pub eval: &'a dyn SystemEval,
    /// Working parameter state for this solve (same buffer passed to Ceres as
    /// the residual block). Used as the `&Vector` view for [`SystemEval`].
    pub state: *mut Vector,
    /// Reused across residual trampoline calls (sized at solve start).
    pub residual_buf: Vector,
    /// Reused across jacobian trampoline calls (sized at solve start from
    /// [`SystemEval::new_jacobian`]); densified into the foreign buffer.
    pub jacobian_buf: BlockSparseMatrix,
}

/// Fill the foreign residual buffer `r` at `p`.
///
/// # Safety
///
/// `user` must point to a live [`EvalHandle`]; `p` / `r` must be valid for
/// `n_params` / `n_res` elements.
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
    let handle = &mut *(user as *mut EvalHandle<'_>);
    if handle.state.is_null() {
        return -1;
    }
    if handle.eval.n_params() != n_params || handle.eval.n_residuals() != n_res {
        return -2;
    }
    let state = &mut *handle.state;
    debug_assert_eq!(state.len(), n_params);
    let incoming = std::slice::from_raw_parts(p, n_params);
    if state.as_ptr() != p {
        state.copy_from_slice(incoming);
    }
    debug_assert_eq!(handle.residual_buf.len(), n_res);
    handle.eval.residual(state, &mut handle.residual_buf);
    std::slice::from_raw_parts_mut(r, n_res).copy_from_slice(handle.residual_buf.as_slice());
    0
}

/// Fill the foreign row-major dense Jacobian buffer `j` at `p`.
///
/// Only built with the `ceres` feature (the sole dense foreign consumer).
///
/// # Safety
///
/// `user` must point to a live [`EvalHandle`]; `p` / `j` must be valid for
/// `n_params` / `n_res * n_params` elements.
#[cfg(feature = "ceres")]
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
    let handle = &mut *(user as *mut EvalHandle<'_>);
    if handle.state.is_null() {
        return -1;
    }
    if handle.eval.n_params() != n_params || handle.eval.n_residuals() != n_res {
        return -2;
    }
    let state = &mut *handle.state;
    debug_assert_eq!(state.len(), n_params);
    let incoming = std::slice::from_raw_parts(p, n_params);
    if state.as_ptr() != p {
        state.copy_from_slice(incoming);
    }
    handle.eval.jacobian(state, &mut handle.jacobian_buf);
    let out = std::slice::from_raw_parts_mut(j, n_res * n_params);
    out.fill(0.0);
    handle.jacobian_buf.add_to_dense(out);
    0
}

#[cfg(test)]
#[path = "tests_ffi.rs"]
mod tests_ffi;
