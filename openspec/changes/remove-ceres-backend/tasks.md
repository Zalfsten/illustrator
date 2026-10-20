# Tasks

Paths relative to `impls/linux-rust/crates/constraint_runtime` unless noted. Tests: `cargo test --workspace` from `impls/linux-rust`.

## 1. Remove the Ceres backend

- [x] 1.1 Delete `src/ceres_solver.rs`, `src/tests_ceres_solver.rs`, `cpp/`, `build.rs` and `tests/ceres_reference.rs`; remove `[features] ceres` and `[build-dependencies] cc` from `Cargo.toml`, and the `ceres_solver` module / `CeresSolver` re-export from `src/lib.rs`. Verify `cargo test -p constraint_runtime` is green and `grep -rni ceres src tests Cargo.toml` is empty.
- [x] 1.2 Simplify `benches/solve.rs`: remove `bench_ceres`, `DENSE_SIZES` and the feature-gated `criterion_group!` variants (single group: jacobian + lm). Verify `cargo bench -p constraint_runtime --bench solve --no-run` builds without warnings.

## 2. Remove the FFI evaluation layer

- [x] 2.1 Delete `src/solver/ffi.rs` and `src/solver/tests_ffi.rs`; remove `pub mod ffi` from `src/solver.rs` and `ffi` from the `src/lib.rs` re-exports. Verify `cargo test -p constraint_runtime` is green and `grep -rn "unsafe\|extern \"C\"\|EvalHandle\|trampoline\|dyn SystemEval" src` returns nothing (except the test-only counting allocator in `src/test_alloc.rs`, whose `unsafe impl GlobalAlloc` is required).
- [ ] 2.2 Reword docs left over from Ceres/FFI: `SolverOptions::lm_lambda0` ("LM backends only"), `SystemEval` and `BlockSparseMatrix::add_to_dense` docs (dense materialisation for tests/debugging), `src/lib.rs` crate doc. Verify `cargo doc -p constraint_runtime --no-deps` has no broken intra-doc links (no warnings).

## 3. Dev Container and documentation

- [x] 3.1 `.devcontainer/Dockerfile`: remove `libceres-dev` and `g++` and the Ceres comment; keep `pkg-config` with a comment naming `sketch_ui` / `wayland-sys`. `.devcontainer/devcontainer.json`: keep `CARGO_BUILD_JOBS=6`, replace the Ceres/Eigen comment. Verify with `cargo tree -i cc --workspace` / `cargo tree -i pkg-config --workspace` that the remaining users are `wayland-backend` (C only) and `wayland-sys`, and that `build-essential` (gcc/g++) is installed in the base image (`dpkg -l build-essential`).
- [x] 3.2 Update `impls/linux-rust/README.md` (requirements, solver table, build/test and bench commands without `--features ceres`), `docs/constraint-mathematics.md` (assembly note without FFI/Ceres), `openspec/config.yaml` context (solver bullet without `ffi.rs` / `CeresSolver`), and add a note in `impls/linux-rust/BENCHMARKS.md` that Ceres was removed (historical rows kept). Verify `grep -rniE "ceres|ffi|trampoline" impls/linux-rust/README.md docs openspec/config.yaml` only hits intentional historical mentions (none expected), and the README commands run as written.

## 4. Integration check

- [x] 4.1 Run `cargo test --workspace`, `cargo clippy --workspace --all-targets` (0 warnings), `cargo bench -p constraint_runtime --bench solve --no-run`, and `openspec validate remove-ceres-backend --strict`. Verify all pass.

## Workflow follow-up

- Rebuild the Dev Container so the slimmer image (no `libceres-dev`, no `g++`) is used; confirm `cargo test --workspace` in the rebuilt container.
- Archive the change; sync the `solver-port` delta (requirement removed, C-ABI wording dropped) into the main spec.
- Update the auto-memory note on the WASM goal (Ceres removed; next step WASM build).
