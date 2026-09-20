# Tasks

## 1. Native dependency

- [ ] 1.1 Identify and install Ceres (and Eigen/glog as needed) in `.devcontainer/Dockerfile` for Debian trixie; verify `pkg-config` or compile smoke can find headers/libs after image rebuild instructions are documented
- [ ] 1.2 Document Ceres install/link and BSD-3 license note in `impls/linux-rust/README.md`; verify the README section exists and matches the Dockerfile packages

## 2. Ceres SolverPort adapter

- [ ] 2.1 Add C++/cxx (or cc) bridge that builds a Ceres problem whose cost Evaluate fills residual and dense Jacobian via the existing `SystemEval` / FFI trampoline contract; verify a unit test can call residual fill through the bridge
- [ ] 2.2 Implement `CeresSolver: SolverPort` mapping `SolverOptions` (tol, max_iters) and documenting `lm_lambda0` as LmSolver-only; verify `cargo test -p constraint_runtime` includes at least one successful Ceres solve (e.g. FixPoint or congruence fixture)
- [ ] 2.3 Keep `LmSolver` and `StubSolver` working; verify existing LM and stub tests still pass

## 3. Default interactive path

- [ ] 3.1 Make `InteractiveSolve` / `sketch_ui` default to `CeresSolver`; verify acceptance tests pass and `cargo build -p sketch_ui` succeeds

## 4. Hygiene

- [ ] 4.1 Run `openspec validate ceres-solver-backend` and fix any reported issues in change artifacts
