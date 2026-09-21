# Design

## Context

See proposal.md for motivation. Today `constraint_runtime` exposes `SolverPort` + `SystemEval` (dense residual and row-major Jacobian) and `solver::ffi` C trampolines. Default interactive path uses owned `LmSolver`. Dev Container is Rust-on-Debian-trixie with apt-managed deps in `.devcontainer/Dockerfile`.

## Goals / Non-Goals

**Goals:**

- Ship a `CeresSolver` implementing `SolverPort` for linux-rust.
- Wire interactive solve to Ceres by default; keep `LmSolver` compilable and tested.
- Document build/link steps in `impls/linux-rust/README.md`.

**Non-Goals:**

- User-facing or env-based backend switch (later change).
- Sparse Jacobian / Schur specializations beyond what Ceres needs for dense small problems.
- Changing soft/hard Interact commit semantics or residual weighting.
- Mandating Ceres in shared OpenSpec capabilities.

## Decisions

### 1. Binding style: `cxx` bridge + CostFunction callbacks

Use a small C++ translation unit (via `cxx` or `cc` crate) that:
- Builds a `ceres::Problem` with one dynamic cost function sized `(m, n)` from `SystemEval`
- On `Evaluate`, copies `p`, calls into Rust residual/Jacobian (reusing trampoline logic or a Rust-exported `extern "C"` that wraps `SystemEval`)
- Runs `ceres::Solve` with dense QR / dense Schur as appropriate for small \(n\)

**Alternatives:** Pure C wrapper around a hand-rolled C API — more code. Call Ceres only from a separate process — worse latency. Rust-only NLS crate — rejected by product choice (Ceres).

### 2. Package install

Prefer Debian packages on trixie (`libceres-dev` and transitive Eigen/glog as packaged). If versions are too old or missing, fall back to documenting a source build in the Dockerfile (pinned Ceres release).

### 3. Default wiring

- `InteractiveSolve::default()` (or a type alias / constructor used by `sketch_ui`) uses `CeresSolver`.
- Unit tests that intentionally need a controllable stub keep `StubSolver`; selected LM regression tests keep `LmSolver` explicitly.
- Feature flag `ceres` default-on for the crate so CI/devcontainer always build the default path; optional later to make it optional for hosts without Ceres.

### 4. Options mapping

Map `SolverOptions.tol` → Ceres function/parameter tolerance (document exact fields). Map `max_iters` → `max_num_iterations`. `lm_lambda0` is LM-specific: either map to Ceres initial trust-region radius if meaningful, or **ignore with observable log/debug note** and document fail-closed policy for “required” settings — do not silently claim LM damping was honored. Prefer rejecting unsupported *required* opts only if we introduce a stricter options API; for v1, document that `lm_lambda0` applies to `LmSolver` only.

### 5. License

Ceres is BSD-3-Clause; compatible with MIT/Apache-2.0 crate licensing. Note dependency in linux-rust README.

## Risks / Trade-offs

- [Heavy native dep / longer image build] → Prefer apt packages; cache Docker layers.
- [ABI / link errors on rebuild] → Pin package versions in Dockerfile comments; smoke `cargo test` in post-create optional.
- [Behavioral drift vs LM on soft-weighted systems] → Acceptance tests must pass on Ceres; keep LM tests for owned backend.
- [cxx compile times] → Isolate C++ in one small cc target.

## Migration Plan

1. Dockerfile: install Ceres dev packages.
2. Implement bridge + `CeresSolver`.
3. Flip interactive default; update README.
4. Run full `constraint_runtime` tests + acceptance; manual `sketch_ui` drag smoke.

## Open Questions

- Exact Debian package name/version on trixie at apply time — resolve during task 1 by `apt-cache` in the container.
