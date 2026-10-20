# Design

Target implementation: `impls/linux-rust` (`crates/constraint_runtime`, default backend wiring in `sketch_ui`).

## Context

See proposal.md (Why). Current state in linux-rust:

- `SystemEval::jacobian(&self, p, j: &mut Matrix)` fills a dense row-major \(m \times n\) `Matrix`. `AssembledSystem` zeroes it, then writes every contribution through `DenseMatrix::scatter_slice(row0, m, conn)` → `ScatterMatrix`, which maps local column `k` to `conn[k]`.
- Formula and constraint code writes through the generic `MatrixAccess` trait (`set_scaled` **assigns**, `VecJet::contract_into` **adds**). Joined contributions (`Congruence`, `EqualLength`, `Parallel`, `Orthogonal`) place the second operand with `block_slice`. If both operands touch the same geometry, `conn` contains duplicate global columns. With assignment, the second block then overwrites the first in the dense matrix. `Document::add_constraint` does not forbid this (for example a Congruence between the start and end of one arc).
- `InteractiveSolve` builds a fresh `AssembledSystem` for every solve (soft phase plus optional structural polish) and ignores `SolveOutcome`. Commit is gated only on the structural residual norm (`tol * 10`). The backend's Success/Failure distinction therefore does not change interaction behaviour.
- `CeresSolver` (default) calls `cpp/ceres_bridge.cpp`: one residual block, DENSE_QR, LM trust region. The dense Jacobian comes from `solver/ffi.rs` trampolines. `build.rs` always compiles and links Ceres, glog and gflags.
- `LmSolver` forms dense \(J^\top J + \lambda I\) and solves it with Gaussian elimination. Steps are accepted only if the residual norm drops.
- No hot-path benchmark exists.

## Goals / Non-Goals

**Goals:**

- Jacobian fill without scatter on write: each contribution writes its dense local block into a contiguous slice.
- Allocation-free, statically dispatched hot path. All per-solve buffers (Jacobian values, normal-equation values, factorisation scratch) are allocated once at solve start.
- A pure-Rust default backend whose quality is at least that of Ceres on the existing interact and acceptance tests.
- Ceres stays available as an opt-in reference backend.

**Non-Goals (design level):**

- Caching the symbolic factorisation across solves or drag frames. A fresh `AssembledSystem` per solve stays.
- A user-facing backend switch.
- Removing `ScatterVec` / `VecAccess` scatter in `vector.rs` (residual path, unaffected).

## Decisions

### 1. Contribution-blocked Jacobian (`BlockSparseMatrix`)

*Revised after review: the first implementation split structure (`BlockPattern`) and values (`BlockJacobian`), so neither looked like a matrix and every access needed both. Replaced by one self-contained type.*

`BlockSparseMatrix` in `matrix.rs`: an \(m \times n\) sparse matrix made of dense **row blocks**. Block `b` covers rows `row0..row0+m_b` and the columns listed in its own column map (`cols_b`, length \(k_b\)); its values are stored contiguously, row-major. Each row belongs to exactly one block. The global \(J\) is the **sum** of all blocks placed at their rows / mapped columns.

- Structure is built with `BlockSparseMatrix::new(ncols)` + `push_block(nrows, cols) -> b`; rows grow in push order. Structure is private; values start at zero.
- `block_mut(b) -> DenseMatrix` is the dense submatrix \(J[\text{rows}_b, \text{cols}_b]\) (packed slice, no scatter on write). `blocks()` iterates read-only `Block` views (`rows()`, `cols()`, `values()`, and matrix-style reads `blk[(i, k)]`). `DenseMatrix` is the writable local window (no knowledge of its place in \(J\)); `Block` is the read-only view that also knows its rows and column map — the two roles are kept separate on purpose.
- Matrix operations: `nrows()`, `ncols()`, `fill(v)`, `transpose_mul(&r, &mut g)` (\(g = J^\top r\)), `add_to_dense(&mut [f64])` (row-major \(m \times n\) buffer, duplicates summed; there is no owned dense matrix type).
- `SystemEval` becomes `n_params()`, `n_residuals()`, `residual(p, r)`, `new_jacobian() -> BlockSparseMatrix` (structure built, zero values) and `jacobian(p, &mut BlockSparseMatrix)`. The solver calls `new_jacobian()` once per solve.
- `AssembledSystem` no longer stores connectivity: the constructor only sums residual arities; `new_jacobian()` builds the structure (block order: structural, soft slots, soft point follows, soft length floors) — once per solve, as before, because Interact builds one `AssembledSystem` per solve. `residual()` uses a running row offset. `jacobian()` zeroes each block before its contribution writes; an inactive `SoftLengthFloor` stays zero.
- The column map is the **solver column mapping** (spec: assembly-owned column mapping); today it equals the State indices. A later Congruence elimination only rewrites the maps and `ncols`.

Because every local column has its own storage slot, duplicate columns cannot overwrite each other (system-assembly: "Duplicate connectivity columns accumulate").

*Alternatives:* keep the dense `Matrix` (rejected: \(O(mn)\) memory, \(O(mn^2)\) per step); standard CSR/CSC with deduplicated columns plus slot tables (rejected: scatter on write); factor-graph library (rejected: it would own the structure). `BlockSparseMatrix` deliberately does **not** offer mutable global element access `J[(i, j)]`: a structural zero has no storage and a duplicate column is a sum of two slots, so no `&mut f64` exists — writes go through the dense block submatrices.

### 2. `ScatterMatrix` and the `MatrixAccess` trait removed

After decision 1 every writer receives a dense block, so `ScatterMatrix` and `DenseMatrix::scatter_slice` are deleted. With `DenseMatrix` as the only implementor, the `MatrixAccess` trait (with its `BlockSlice<'a>` GAT) no longer abstracts anything: formula / constraint signatures take `&mut DenseMatrix<'_>` instead of `J: MatrixAccess`, and `nrows`, `ncols`, `block_slice`, `set_scaled`, `fill` become inherent `DenseMatrix` methods. Generated code is unchanged (the trait was only ever instantiated with `DenseMatrix`). Matrix types are then: `DenseMatrix` (borrowed dense window, also over plain `Vec<f64>` buffers) and `BlockSparseMatrix` (sparse; its blocks are `DenseMatrix`).

### 3. Normal-equation assembly (`NormalSystem`)

*(Uses `BlockSparseMatrix::blocks()` for \(J^\top J\) and `transpose_mul` for \(g = J^\top r\).)*

The solver forms \(G = J^\top J\) and \(g = J^\top r\) itself. It does not hand \(J\) to a library: the block columns are unsorted and may repeat.

- **Setup (once per solve):** collect every `(cols[a], cols[c])` pair with `cols[a] <= cols[c]` over all blocks (upper triangle). Add the full diagonal \(0..n\) so damping always has a slot, even for a parameter no contribution touches. Sort and deduplicate into a CSC pattern. For every block, store a `slots: Vec<usize>` with \(k_b^2\) entries (or upper-triangular entries only) pointing into `G.values`. Duplicate columns map to the same slot, so their contributions add: \((B_1+B_2)^2\) appears automatically. Store `diag_slot[i]`.
- **Per iteration:** zero `G.values` and `g`. For each block, \(G \mathrel{+}= B^\top B\) through `slots`, and \(g[\text{cols}[a]] \mathrel{+}= \sum_i B_{ia}\, r_{\text{row0}+i}\). Cost \(O(\sum_b m_b k_b^2)\), no allocation.

### 4. Owned sparse LM (`LmSolver`, replaces the dense one)

Algorithm: Levenberg–Marquardt with Marquardt scaling and Nielsen damping update (Madsen/Nielsen/Tingleff, "Methods for Non-Linear Least Squares Problems"):

- \(D = \mathrm{diag}(G)\), clamped to \([d_{\min}, d_{\max}]\) with \(d_{\min} = 10^{-6}\) and \(d_{\max} = 10^{32}\), as in Ceres. This keeps parameters that are almost untouched (underdetermined sketch) damped.
- Solve \((G + \lambda D)\,\delta = -g\) with sparse Cholesky \(LL^\top\).
- Gain ratio \(\rho = \dfrac{\tfrac12\|r(p)\|^2 - \tfrac12\|r(p+\delta)\|^2}{\tfrac12\,\delta^\top(\lambda D\delta - g)}\).
  - If \(\rho > 10^{-3}\): accept, \(\lambda \leftarrow \lambda \cdot \max(\tfrac13,\, 1-(2\rho-1)^3)\), \(\nu \leftarrow 2\).
  - Otherwise: reject, \(\lambda \leftarrow \lambda\nu\), \(\nu \leftarrow 2\nu\).
- If the factorisation fails (not positive definite numerically), treat it as a rejected step: raise \(\lambda\).
- Start: \(\lambda_0 = \) `opts.lm_lambda0`. With Marquardt scaling \(D = \mathrm{diag}(G)\) the damping is already relative, so no extra \(\max_i D_{ii}\) factor (Ceres equivalently starts at \(\lambda = 1/\text{radius}_0\)). Document this in the `SolverOptions` docs.
- Stop early when \(\|r\| \le\) `opts.tol`, when \(\|g\|_\infty \le 10^{-12}\) (stationary), when \(\|\delta\| \le 10^{-12}(\|p\| + 10^{-12})\), when \(\lambda > 10^{16}\), or after `max_iters`.
- Outcome: **Success** iff the final \(\|r\| \le\) `opts.tol` (solver-port: a successful solve reports satisfaction within the configured tolerance). Otherwise **Failure** with the stop reason in the message (stationary least-squares optimum, step too small, damping overflow, iteration limit). Interact ignores the outcome and gates commits on the structural norm, so soft-phase "failures" do not change interaction. Only descending steps are accepted, so the written-back \(p\) is always the best iterate.
- Scratch allocated once per solve: `BlockJacobian`, `r`, `r_trial`, `p_trial`, \(G\), \(g\), \(\delta\), factorisation memory.

The name `LmSolver` stays. Its existing tests (`lm_solves_fixpoint_analytically`, `lm_solves_congruence`) remain as the minimal regression tests.

*Alternatives:*

- Sparse QR on \([J;\sqrt{\lambda D}]\). More robust (no squared condition number) but needs a full refactorisation per \(\lambda\) and more fill-in. Kept as fallback if conditioning problems show up in benchmarks.
- Dogleg. Fewer factorisations per accepted step, but more state. Not needed now.

### 5. Linear algebra: `faer` (sparse Cholesky)

- Add `faer` as a dependency, pinned to an exact minor version, with `default-features = false` plus `std` (no rayon, to keep the later WASM build open). Symbolic analysis (`SymbolicLlt` or equivalent, including fill-reducing ordering) runs **once per solve** on the CSC pattern of \(G\). The numeric factorisation and the solve run per iteration with a pre-allocated `MemBuffer`/`MemStack`, so there is no heap allocation in the loop.
- If the exact faer API needs a different call pattern (for example its own CSC type for the pattern), wrap it in a small private module `solver/sparse_chol.rs`. LM code does not depend on faer types.

*Alternatives:*

- Own sparse Cholesky. Rejected: ordering and supernodes are the hard part.
- `nalgebra-sparse`. Less mature Cholesky and no comparable symbolic/numeric split.
- Dense Cholesky on \(G\) for small \(n\). Rejected: one code path, decided by the benchmark.

### 6. Ceres as an opt-in reference backend

- `Cargo.toml`: `[features] ceres = []`, not in `default`. `cc` stays a build dependency (cheap). `build.rs` returns early unless `CARGO_FEATURE_CERES` is set.
- `ceres_solver.rs`, `tests_ceres_solver.rs`, the jacobian part of `solver/ffi.rs` and the `CeresSolver` re-export are under `#[cfg(feature = "ceres")]`. The residual trampoline stays unconditional. It is plain Rust and keeps the FFI contract tested.
- `EvalHandle` keeps a `BlockJacobian` and a dense `Matrix` scratch. The jacobian trampoline fills the blocks and writes them densely into the Ceres pointer (zero, then `+=` per block). There is no extra `to_dense` allocation per callback.
- Outcome semantics aligned with the solver-port spec (fixed after apply): `CeresSolver` reports Success iff the final \(\|r\| \le\) `tol`; a usable but unconverged Ceres result is still written back to \(p\) but reported as Failure with Ceres' summary message. Previously any usable result counted as Success.
- Comparison test (feature `ceres`): the same scenarios through `LmSolver` and `CeresSolver` reach structural satisfaction, and their final structural norms are both ≤ `tol*10`.
- `InteractiveSolve<S: SolverPort = LmSolver>`; `Default`/`new()` use `LmSolver`. The doc comment on `lib.rs` is updated.
- The Dev Container keeps the Ceres packages. `impls/linux-rust/README.md` documents `cargo test --features ceres`.

### 7. Hot-path benchmark and allocation check

- `criterion` as a dev-dependency, `benches/solve.rs`. Scenarios: an open staircase chain of `N` segments (Congruence joints, Orthogonal neighbours, Parallel every 5th pair, EqualLength every 3rd pair, one FixPoint, small deterministic perturbation so the start is near-consistent); plus a drag step on the last endpoint (soft point follow + length floors). `N ∈ {10, 30, 100}` for the dense baseline (dense \(O(mn^2)\) makes larger sizes impractical); `N = 500` is added once the sparse path exists. A closed all-right-angle polygon was dropped: from arbitrary start positions it does not reliably close, which would benchmark failures.
- Measured separately: `jacobian()`, normal-equation assembly, and a full `solve_commit_soft`.
- The baseline (current dense `LmSolver` and `CeresSolver`) is recorded **before** the refactor in `impls/linux-rust/BENCHMARKS.md`, and compared after it.
- Allocation freedom: an integration test `tests/alloc_free.rs` with a counting `#[global_allocator]` asserts zero allocations for repeated `jacobian()` and normal-equation assembly calls on a pre-sized system. If faer can run the numeric factorisation without allocation, include it as well; otherwise record that exception in this design.

## Risks / Trade-offs

- [Squared condition number of normal equations on nearly singular sketches] → Marquardt scaling plus a \(d_{\min}\) clamp. Compare against Ceres DENSE_QR in feature tests. Sparse QR is the documented fallback.
- [faer API churn between versions] → Pin the exact version and isolate it in `solver/sparse_chol.rs`.
- [Symbolic analysis per solve costs more than the iterations for tiny systems] → Measure with the benchmark at `N = 10`. Caching the symbolic analysis across drag frames is the follow-up if needed. The pattern only changes when soft terms or constraints change.
- [Behaviour drift during drag (different local minimum than Ceres)] → Acceptance tests plus a manual `sketch_ui` drag smoke test. Interact still gates commits structurally, so drift cannot commit invalid states.
- [Breaking `SystemEval` signature] → No external users besides this workspace. `sketch_ui` only uses `InteractiveSolve`.
- [Duplicate-column fix changes numbers for existing sketches with self-referencing constraints] → That is a bug fix. It is covered by the new finite-difference test.

## Migration Plan

1. Benchmark and baseline on the old code.
2. Introduce `BlockPattern`/`BlockJacobian`, switch `SystemEval` and `AssembledSystem`, and keep the old dense `LmSolver` and Ceres working via `to_dense`. All tests stay green at this step.
3. Remove `ScatterMatrix`.
4. Add normal-equation assembly and the sparse LM; switch the default.
5. Put Ceres behind the feature and add comparison tests.
6. Rerun the benchmark and update the docs.

Rollback: steps 2–3 are independent of the solver. Reverting steps 4–5 restores the dense LM / Ceres default.

## Open Questions

Resolved during apply:

- faer version: pinned `=0.24.4` (`default-features = false`, features `std`, `sparse-linalg`; no rayon). API used: `factorize_symbolic_cholesky` (AMD ordering, `Side::Upper`) once per solve; `SymbolicCholesky::factorize_numeric_llt` + `LltRef::solve_in_place_with_conj` per iteration with one `MemBuffer` sized from `factorize_numeric_llt_scratch` ∪ `solve_in_place_scratch`. Dynamic regularisation is disabled so non-positive-definite systems surface as errors (LM raises λ).
- Allocation freedom: verified by `solver/tests_normal.rs::assemble_and_factor_solve_do_not_allocate` — normal assembly, numeric LLT and solve perform zero heap allocations after setup. No exception needed.
