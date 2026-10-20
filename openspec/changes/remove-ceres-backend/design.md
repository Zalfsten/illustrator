# Design

Target implementation: `impls/linux-rust` (crate `constraint_runtime`) plus the Dev Container.

## Context

See proposal.md (Why). Current state after `sparse-lm-solver`:

- `CeresSolver` (`src/ceres_solver.rs`, tests in `src/tests_ceres_solver.rs`) wraps `cpp/ceres_bridge.{h,cpp}`. It is compiled only with the non-default Cargo feature `ceres`; `build.rs` returns early otherwise; `cc` is a build dependency used only for this bridge.
- `src/solver/ffi.rs`: `EvalHandle` (`&dyn SystemEval`, state pointer, residual / Jacobian scratch) plus `extern "C"` `residual_trampoline` (always built) and `jacobian_trampoline` (feature `ceres`, densifies via `BlockSparseMatrix::add_to_dense`). `src/solver/tests_ffi.rs` tests the residual trampoline. Only `CeresSolver` uses the module.
- `tests/ceres_reference.rs` compares `LmSolver` and `CeresSolver` (feature-gated). `benches/solve.rs` has a feature-gated Ceres group with `DENSE_SIZES`.
- Dev Container: `libceres-dev`, `pkg-config` and `g++` are installed "for the C++ bridge". `pkg-config` is also needed by `sketch_ui` (`wayland-sys` build dependency via `eframe` → `glutin`); `cc` / C++ is not used by any other crate in the workspace.
- Docs mentioning Ceres / FFI: `impls/linux-rust/README.md`, `docs/constraint-mathematics.md`, `openspec/config.yaml` context, `src/lib.rs` and `src/solver.rs` doc comments, `.devcontainer/devcontainer.json` comment on `CARGO_BUILD_JOBS`.

## Goals / Non-Goals

**Goals:**

- No Ceres, no C++ and no `unsafe` FFI glue left in `constraint_runtime`; default and only build path is pure Rust.
- Keep all remaining behaviour and tests green; keep the benchmark and allocation checks.

**Non-Goals:**

- Removing `SolverPort`, `StubSolver`, `InteractiveSolve<S>` or `SystemEval` (user decision: keep).
- Any change to `BlockSparseMatrix`, `NormalSystem`, `SparseLlt` or the LM algorithm.

## Decisions

### 1. Delete the Ceres backend completely

Delete `src/ceres_solver.rs`, `src/tests_ceres_solver.rs`, `cpp/` and `build.rs`. Drop `[features] ceres`, `[build-dependencies] cc` and the `ceres_solver` module / `CeresSolver` re-export from `src/lib.rs`. Delete `tests/ceres_reference.rs`. In `benches/solve.rs` remove `bench_ceres`, `DENSE_SIZES` and the two `cfg`-alternative `criterion_group!` lines (one group: jacobian + lm).

*Alternative:* keep it behind the feature as a reference. Rejected: the reference test only checks that both backends satisfy the constraints (a weak oracle). Finite-difference, acceptance and allocation tests carry correctness. The code stays recoverable from git history and `BENCHMARKS.md` keeps the measured baseline.

### 2. Delete the FFI evaluation layer (spec requirement removed)

Delete `src/solver/ffi.rs` and `src/solver/tests_ffi.rs` and the `pub mod ffi` / `ffi` re-export. This removes the last `&dyn SystemEval` and all `unsafe` code in the solver path. `SystemEval` itself stays as the generic (static-dispatch) assembly/solver boundary. The solver-port delta removes the "FFI-friendly evaluation contract" requirement and the C-ABI wording.

*Alternatives:* keep `ffi.rs` (spec unchanged) or delete it but keep the spec (spec unverified in code). Both were offered; the user chose to delete it and relax the spec.

### 3. Keep `BlockSparseMatrix::add_to_dense`

After decision 2 its only callers are tests (dense oracle in `tests_assemble.rs`, `tests_constraint.rs`, `tests_matrix.rs`, `solver/tests_normal.rs`). It stays public: the solver-port spec explicitly allows a dense materialisation, and it is the reference that the block-sparse code is tested against. Only its doc wording changes from "backends that need a dense matrix" to "dense materialisation (tests, debugging)".

### 4. Dev Container

- `Dockerfile`: remove `libceres-dev` and `g++` together with the "Ceres NLS … C++ bridge" comment. Keep `pkg-config` with a comment naming its real user (`sketch_ui` / `wayland-sys`).
- `devcontainer.json`: keep `CARGO_BUILD_JOBS=6` and the memory/CPU caps (they also protect the host's Ollama). Replace the Ceres/Eigen justification with the actual one: keep parallel Rust compile jobs bounded inside the container caps.
- The image change takes effect on the next container rebuild. Verification inside the current container is the plain `cargo` build/test without the feature; the rebuild is a follow-up the user triggers.

### 5. Simplification review (beyond Ceres)

Checked for code that only existed for Ceres / FFI:

- `Vector`: no FFI-only methods (raw pointers came via `Deref<[f64]>`). Nothing to remove.
- `SolverOptions::lm_lambda0`: doc mentions "Ceres ignores it". Reword to "LM backends only".
- `src/solver.rs` / `src/lib.rs` crate docs: drop Ceres / FFI mentions.
- `SolverPort`, `StubSolver`, `InteractiveSolve<S>`, `SystemEval`: kept by user decision.

No further simplifications were found that do not remove an abstraction.

### 6. Documentation

- `README.md`: Requirements (pure Rust only), Solvers table (`LmSolver`, `StubSolver`), Build & test (drop `--features ceres` commands).
- `docs/constraint-mathematics.md`: drop "LM/FFI reuse … Ceres adapter densifies" from the assembly note.
- `openspec/config.yaml` context: drop `ffi.rs` / `CeresSolver` from the solver bullet.
- `BENCHMARKS.md`: keep historical Ceres rows and note that the backend was removed in this change. Drop the `--features ceres` bench command from the README.

## Risks / Trade-offs

- [Losing a cross-check against an established library] → The remaining tests check the Jacobian against finite differences and the solve against the acceptance scenes. Ceres can be restored from git if a future constraint type needs comparison.
- [Spec change affects other implementations] → Only removes an obligation; nothing that conformed before becomes non-conforming.
- [Dockerfile change untested until rebuild] → `g++` and `cc` are unused by any remaining crate (checked with `cargo tree -i`). If the rebuild fails, re-adding a package is a one-line change.

## Migration Plan

1. Delete code and tests (decisions 1–2), adjust `Cargo.toml`, `lib.rs`, bench.
2. Docs and Dev Container edits.
3. `cargo test --workspace`, `cargo clippy --workspace --all-targets`, bench build. Then the user rebuilds the container.

Rollback: revert the change commit.
