#include "ceres_bridge.h"

#include <ceres/ceres.h>

#include <algorithm>
#include <cstring>
#include <string>

namespace {

class CallbackCostFunction final : public ceres::CostFunction {
 public:
  CallbackCostFunction(size_t n_params,
                       size_t n_residuals,
                       void* user,
                       cr_residual_fn residual,
                       cr_jacobian_fn jacobian)
      : n_params_(n_params),
        n_residuals_(n_residuals),
        user_(user),
        residual_(residual),
        jacobian_(jacobian) {
    set_num_residuals(static_cast<int>(n_residuals_));
    mutable_parameter_block_sizes()->push_back(static_cast<int>(n_params_));
  }

  bool Evaluate(double const* const* parameters,
                double* residuals,
                double** jacobians) const override {
    if (residual_(user_, n_params_, parameters[0], n_residuals_, residuals) != 0) {
      return false;
    }
    if (jacobians != nullptr && jacobians[0] != nullptr) {
      if (jacobian_(user_, n_params_, parameters[0], n_residuals_, jacobians[0]) !=
          0) {
        return false;
      }
    }
    return true;
  }

 private:
  size_t n_params_;
  size_t n_residuals_;
  void* user_;
  cr_residual_fn residual_;
  cr_jacobian_fn jacobian_;
};

void set_err(char* err_msg, size_t err_msg_len, const std::string& msg) {
  if (err_msg == nullptr || err_msg_len == 0) {
    return;
  }
  std::strncpy(err_msg, msg.c_str(), err_msg_len - 1);
  err_msg[err_msg_len - 1] = '\0';
}

}  // namespace

extern "C" int cr_ceres_solve(size_t n_params,
                              double* p,
                              size_t n_residuals,
                              void* user,
                              cr_residual_fn residual,
                              cr_jacobian_fn jacobian,
                              const cr_ceres_options* opts,
                              char* err_msg,
                              size_t err_msg_len) {
  if (p == nullptr || residual == nullptr || jacobian == nullptr || opts == nullptr) {
    set_err(err_msg, err_msg_len, "null argument to cr_ceres_solve");
    return -1;
  }
  if (n_params == 0 || n_residuals == 0) {
    return 0;
  }

  ceres::Problem problem;
  ceres::CostFunction* cost =
      new CallbackCostFunction(n_params, n_residuals, user, residual, jacobian);
  problem.AddResidualBlock(cost, nullptr, p);

  ceres::Solver::Options options;
  options.linear_solver_type = ceres::DENSE_QR;
  options.minimizer_progress_to_stdout = false;
  options.logging_type = ceres::SILENT;
  options.max_num_iterations = std::max(1, opts->max_iterations);
  // Function tolerance tracks residual reduction. Keep parameter/gradient
  // tolerances much tighter so tiny soft-residual steps do not stop early
  // while hard (structural) residuals are still above Interact's gate.
  options.function_tolerance = opts->function_tolerance;
  options.parameter_tolerance = 1e-16;
  options.gradient_tolerance = 1e-16;

  ceres::Solver::Summary summary;
  ceres::Solve(options, &problem, &summary);

  set_err(err_msg, err_msg_len, summary.message);
  if (!summary.IsSolutionUsable()) {
    return -1;
  }
  // 0 = Ceres CONVERGENCE, 1 = usable but stopped for another reason (still OK for Interact).
  return summary.termination_type == ceres::CONVERGENCE ? 0 : 1;
}
