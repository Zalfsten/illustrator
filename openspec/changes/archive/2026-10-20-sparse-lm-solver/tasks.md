# Tasks

All paths are relative to `impls/linux-rust/crates/constraint_runtime` unless noted. Run tests with `cargo test -p constraint_runtime` from `impls/linux-rust`, and with the Ceres reference also `cargo test -p constraint_runtime --features ceres` once that feature exists (group 5).

## 1. Baseline benchmark (before any refactor)

- [x] 1.1 Add `criterion` as a dev-dependency and `benches/solve.rs` with the staircase-chain scenarios (N = 10, 30, 100 for the dense baseline; N = 500 added once the sparse path exists: open chain with Congruence joints, Orthogonal neighbours, Parallel every 5th pair, EqualLength every 3rd pair, one FixPoint, small deterministic perturbation) for `jacobian()`, the full `structural_only` solve, and one `DragSession::pointer_move` step. Verify that `cargo bench -p constraint_runtime --bench solve` runs for both the current `LmSolver` and `CeresSolver`.
- [x] 1.2 Record the baseline numbers (machine, commit, median times) in `impls/linux-rust/BENCHMARKS.md`. Verify that the file exists and lists all scenarios.
- [x] 1.3 Add a failing-first regression test in `src/tests_assemble.rs`: Congruence between the start and end point of the same `Arc` (duplicate connectivity columns). It compares the assembled Jacobian with central finite differences of the residual. Verify that it fails on the current code (documents the overwrite bug); mark it `#[ignore]` with a reference to task 2.4 until then.

## 2. Contribution-blocked Jacobian

- [x] 2.1 Add `BlockPattern` and `BlockJacobian` to `src/matrix.rs` (block meta: `row0`, `nrows`, `ncols`, `val_off`, `col_off`; flat `cols`; `block_mut` returning `DenseMatrix::packed`; `to_dense` with accumulating duplicates). Verify with unit tests in `src/tests_matrix.rs`: block write round-trip, `to_dense` sums duplicate columns, untouched entries are zero.
- [x] 2.2 Change `SystemEval` in `src/solver.rs` to `jacobian_pattern(&self) -> &BlockPattern` and `jacobian(&self, p, &mut BlockJacobian)`. Update the rustdoc to describe the contribution-blocked contract (spec solver-port). Verify that `cargo check -p constraint_runtime` reports only the expected call-site errors that 2.3–2.5 fix.
- [x] 2.3 Rework `AssembledSystem` in `src/assemble.rs`: build one `BlockPattern` in `with_soft` (structural, then soft slot, soft point, soft length blocks) in place of the three `Vec<Vec<usize>>` connectivity caches. `jacobian()` zeroes each block and writes through `block_mut`; an inactive `SoftLengthFloor` stays zero. Verify that existing `src/tests_assemble.rs` tests pass via `to_dense`.
- [x] 2.4 Remove `#[ignore]` from the duplicate-column test from 1.3 and add a dense-vs-blocked equivalence test (all constraint kinds plus all soft kinds, random `p`, compared against finite differences). Verify that both pass.
- [x] 2.5 Keep the old dense `LmSolver` and `CeresSolver` working on the new contract: dense scratch `Matrix` allocated once per solve and filled from the blocks; `solver/ffi.rs` `EvalHandle` holds a `BlockJacobian` and fills the Ceres buffer densely (zero, then `+=`). Verify that the full `cargo test -p constraint_runtime` (including acceptance and `tests_ffi.rs`) is green.
- [x] 2.6 Remove `ScatterMatrix`, `DenseMatrix::scatter_slice`, their tests in `src/tests_matrix.rs`, the scatter usage in `src/tests_constraint.rs`, and the `lib.rs` re-export; update the module docs in `src/matrix.rs`. Verify that `cargo test -p constraint_runtime` is green and `grep -rn ScatterMatrix src` is empty.
- [x] 2.7 Add `tests/alloc_free.rs` with a counting `#[global_allocator]`: after one warm-up call, repeated `jacobian()` and `residual()` calls on a pre-sized N = 100 staircase chain allocate zero times. Verify that the test passes.

## 3. Normal-equation assembly

- [x] 3.1 Add the `faer` dependency (exact version pin, `default-features = false` + `std`) and a private wrapper `src/solver/sparse_chol.rs`: symbolic LLT once from a CSC upper-triangular pattern, numeric factorisation and solve per call with pre-allocated scratch. Verify with unit tests in `src/solver/tests_sparse_chol.rs` against a dense SPD reference (including a pattern with an empty column that only has a diagonal entry).
- [x] 3.2 Implement `NormalSystem` (in `src/solver.rs` or `src/solver/normal.rs`): CSC pattern of \(J^\top J\) upper triangle plus full diagonal from `BlockPattern`, per-block slot tables, `diag_slot`; `assemble(&BlockJacobian, &Vector r)` fills \(G\) and \(g\). Verify with unit tests: \(G\), \(g\) equal dense \(J^\top J\), \(J^\top r\) from `to_dense`, including the duplicate-column case.
- [x] 3.3 Extend `tests/alloc_free.rs` to cover `NormalSystem::assemble` and, if faer allows it, the numeric factorisation plus solve. Verify that the test passes. If faer's numeric step must allocate, record the exception in `design.md` (Open Questions → resolved).

## 4. Sparse LM backend as default

- [x] 4.1 Replace the body of `LmSolver` with the sparse LM from design §4 (Marquardt scaling with clamp, gain ratio, Nielsen update, relative `lm_lambda0`, Success/Failure rules, best-`p` write-back) and remove `solve_linear_in`. Update the `SolverOptions` rustdoc to say how `lm_lambda0` is interpreted. Verify that `src/tests_solver.rs` passes, plus new tests there: underdetermined sketch (single free segment with one FixPoint) converges; an unsatisfiable system returns Failure with the best `p` restored.
- [x] 4.2 Switch `InteractiveSolve` default to `LmSolver` in `src/interact.rs` (type default, `new`, `Default`) and update the `lib.rs` crate doc. Verify that `src/tests_interact.rs`, `tests/acceptance.rs`, and `cargo build -p sketch_ui` pass.
- [x] 4.3 Manual smoke test: `cargo run -p sketch_ui`, drag chain vertices and arc endpoints; joints and right angles stay intact, and there are no visible stutters. Verify by noting the outcome in `impls/linux-rust/BENCHMARKS.md` (smoke section).

## 5. Ceres as opt-in reference

- [x] 5.1 Add `[features] ceres = []` (not default); gate `build.rs` on `CARGO_FEATURE_CERES`; put `ceres_solver.rs`, `tests_ceres_solver.rs`, the jacobian trampoline and the `CeresSolver` re-export under `#[cfg(feature = "ceres")]`. Verify that `cargo test -p constraint_runtime` passes **without** Ceres linked (check that `cargo tree` / the build log shows no `ceres_bridge` compile) and that `cargo test -p constraint_runtime --features ceres` passes.
- [x] 5.2 Add a comparison test (feature `ceres`; placed in `tests/ceres_reference.rs` instead of `src/tests_ceres_solver.rs` because it only needs the public API and the shared staircase scene): the acceptance-style right-angle drag and the N = 10 / 100 staircase chain drag solved by `LmSolver` and `CeresSolver` both reach structural norm ≤ `tol*10`. Verify with `cargo test -p constraint_runtime --features ceres`.
- [x] 5.3 Update `impls/linux-rust/README.md`: default backend is the owned sparse LM (faer), Ceres is optional via `--features ceres`, the Dev Container still ships the Ceres packages for reference tests. Verify that the documented commands run as written.

## 6. Integration check

- [x] 6.1 Rerun `cargo bench -p constraint_runtime --bench solve` and add the after-numbers next to the baseline in `impls/linux-rust/BENCHMARKS.md` with a short assessment (scaling with N, drag-step latency vs Ceres). Verify that the file contains both columns for every scenario.
- [x] 6.2 Run `openspec validate sparse-lm-solver --strict` and the full workspace test suite (`cargo test --workspace` and `cargo test -p constraint_runtime --features ceres`). Verify that all are green.

## 7. Review follow-up: self-contained block-sparse matrix

- [x] 7.1 Replace `BlockPattern` + `BlockJacobian` with one self-contained `BlockSparseMatrix` in `src/matrix.rs` (design §1: `new`, `push_block`, `nrows`, `ncols`, `block_mut`, `blocks`, `fill`, `transpose_mul`, `add_to_dense`). Verify with unit tests in `src/tests_matrix.rs` (round trip, duplicate columns summed in `add_to_dense` and `transpose_mul`).
- [x] 7.2 Switch `SystemEval` to `n_params` / `n_residuals` / `new_jacobian` / `jacobian(p, &mut BlockSparseMatrix)`; `AssembledSystem` builds the structure in `new_jacobian` only (no stored connectivity, running row offset in `residual`); update `NormalSystem` (uses `blocks()` and `transpose_mul`), `LmSolver`, FFI `EvalHandle`, `CeresSolver`, tests, bench and `tests/alloc_free.rs` (`src/solver/tests_normal.rs` was ported by the local model, reviewed). Verify `cargo test -p constraint_runtime` and `--features ceres` are green and `grep -rn "BlockPattern\|BlockJacobian" impls/linux-rust` is empty.
- [x] 7.3 Remove the `MatrixAccess` trait: formula / constraint / expression signatures take `&mut DenseMatrix<'_>`, trait methods become inherent `DenseMatrix` methods (design §2). Done as a scripted refactor (the local-coder skill excludes renames/refactors). Verify `cargo test --workspace --features constraint_runtime/ceres` green, `cargo clippy --workspace --all-targets --features constraint_runtime/ceres` warning-free, `grep -rn MatrixAccess impls/linux-rust/crates` empty.
- [x] 7.4 Update docs (`docs/constraint-mathematics.md`, `openspec/config.yaml` context, `matrix.rs` module docs, README if affected) and rerun the benchmark; record in `impls/linux-rust/BENCHMARKS.md` that times are unchanged within noise. Verify the allocation tests still pass.

## Workflow follow-up

- Archive the change once review is done; during archive, sync the `solver-port` and `system-assembly` deltas into the main specs.
- Follow-up changes (not part of this one): WASM build; solver-side Congruence elimination via the `cols` mapping; caching the symbolic factorisation across drag frames.
