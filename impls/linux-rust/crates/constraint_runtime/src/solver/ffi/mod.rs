//! C-ABI shaped trampolines for foreign solvers (opaque user data + fill buffers).


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
mod tests;
