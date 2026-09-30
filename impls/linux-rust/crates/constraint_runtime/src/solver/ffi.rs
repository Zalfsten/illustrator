//! C-ABI shaped trampolines for foreign solvers (opaque user data + fill buffers).
//!
//! Ceres may evaluate at a temporary parameter buffer that is not the caller's
//! [`State`] storage. Trampolines sync that buffer into [`EvalHandle::state`]
//! before calling [`SystemEval`] (O(n) only when the pointers differ).

use super::SystemEval;
use crate::state::State;
use std::os::raw::c_void;

pub struct EvalHandle<'a> {
    pub eval: &'a dyn SystemEval,
    /// Owned parameter State for this solve (same buffer passed to Ceres as
    /// the residual block). Used as the `&State` view for [`SystemEval`].
    pub state: *mut State,
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
    let r = std::slice::from_raw_parts_mut(r, n_res);
    handle.eval.residual(state, r);
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
    let j = std::slice::from_raw_parts_mut(j, n_res * n_params);
    handle.eval.jacobian(state, j);
    0
}

#[cfg(test)]
#[path = "tests_ffi.rs"]
mod tests_ffi;
