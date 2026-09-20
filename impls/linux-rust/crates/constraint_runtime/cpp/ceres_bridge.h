#pragma once

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef int (*cr_residual_fn)(
    void* user, size_t n_params, const double* p, size_t n_res, double* r);

typedef int (*cr_jacobian_fn)(
    void* user, size_t n_params, const double* p, size_t n_res, double* j);

typedef struct cr_ceres_options {
  double function_tolerance;
  int max_iterations;
} cr_ceres_options;

/* Dense NLS via Ceres. Updates `p` in place.
 * Returns 0 on Ceres CONVERGENCE, 1 on non-convergence / soft failure, -1 on hard error.
 * `lm_lambda0` is not used (owned LmSolver only). */
int cr_ceres_solve(size_t n_params,
                   double* p,
                   size_t n_residuals,
                   void* user,
                   cr_residual_fn residual,
                   cr_jacobian_fn jacobian,
                   const cr_ceres_options* opts,
                   char* err_msg,
                   size_t err_msg_len);

#ifdef __cplusplus
}
#endif
