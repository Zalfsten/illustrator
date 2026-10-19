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

### 1. Contribution-blocked Jacobian (`BlockPattern` + `BlockJacobian`)

New types in `matrix.rs`:

- `BlockPattern`: built once per `AssembledSystem`. For each block `b`: `row0`, `nrows`, `ncols`, `val_off` (offset into the values array) and `col_off` (offset into a flat `cols: Vec<usize>`). Plus `n_rows`, `n_cols` and the flat `cols` (the contribution connectivities, concatenated).
- `BlockJacobian`: `values: Vec<f64>` of length \(\sum_b m_b k_b\). `block_mut(&pattern, b) -> DenseMatrix<'_>` returns `DenseMatrix::packed(&mut values[off..off + m*k], m, k)`. It provides `to_dense(&pattern) -> Matrix`, which **adds** duplicates.
- `SystemEval` becomes:
  ```rust
  fn jacobian_pattern(&self) -> &BlockPattern;
  fn jacobian(&self, p: &Vector, j: &mut BlockJacobian);
  ```
  The caller allocates `BlockJacobian::zeros(eval.jacobian_pattern())` once per solve.
- `AssembledSystem` replaces `structural_conn` / `soft_point_conn` / `soft_length_conn` (`Vec<Vec<usize>>`) with one `BlockPattern`. Block order: structural constraints, then `SoftContribution`, `SoftPointFollow`, `SoftLengthFloor`. Each contribution, including a single-slot `SoftContribution`, is one block. Before calling the contribution, `jacobian()` zeroes the block (`fill(0.0)`). This keeps `+=` writers such as `contract_into` correct. An inactive `SoftLengthFloor` leaves its block zero but keeps it in the pattern.
- `cols` is the **solver column mapping** (spec: assembly-owned column mapping). Today it equals the State indices from connectivity. A later Congruence elimination only rewrites `cols` and `n_cols`.

Because every local column has its own storage slot, the duplicate-column overwrite cannot happen any more. The Jacobian is correct by construction, matching the system-assembly spec "Duplicate connectivity columns accumulate".

*Alternatives:*

- Keep the dense `Matrix`. Rejected: \(O(mn)\) memory and \(O(mn^2)\) per step.
- Standard CSR/CSC with sorted, deduplicated columns plus a per-block slot table (`SparseScatter`). Rejected: it keeps scatter on write, needs \(m \times k\) slot tables, and duplicates need explicit `+=` everywhere.
- Hand the blocks to a factor-graph library. Rejected: the library would own the structure, and our assembly would become redundant.

### 2. `ScatterMatrix` removed, `MatrixAccess` kept

After decision 1 every writer receives a `DenseMatrix`. Delete `ScatterMatrix`, `DenseMatrix::scatter_slice` and their tests. Keep the `MatrixAccess` trait so formula and constraint signatures stay unchanged. Collapsing it to `DenseMatrix` is a possible follow-up, not part of this change. Update the module docs in `matrix.rs` and the `lib.rs` re-exports.

### 3. Normal-equation assembly (`NormalSystem`)

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
- Start: \(\lambda_0 = \) `opts.lm_lambda0` \(\cdot \max_i D_{ii}\) (relative, Nielsen's \(\tau\)). Document this in the `SolverOptions` docs.
- Stop with **Success** when \(\|r\| \le\) `opts.tol`. Also Success (usable) when \(\|g\|_\infty \le 10^{-12}\) or \(\|\delta\| \le 10^{-12}(\|p\| + 10^{-12})\), so that soft and hard optima are not reported as failures. This matches the current Ceres rc≥0 semantics. **Failure** when `max_iters` is reached without a usable step, or \(\lambda > 10^{16}\). The best \(p\) seen is always written back, as before.
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
- Comparison test (feature `ceres`): the same scenarios through `LmSolver` and `CeresSolver` reach structural satisfaction, and their final structural norms are both ≤ `tol*10`.
- `InteractiveSolve<S: SolverPort = LmSolver>`; `Default`/`new()` use `LmSolver`. The doc comment on `lib.rs` is updated.
- The Dev Container keeps the Ceres packages. `impls/linux-rust/README.md` documents `cargo test --features ceres`.

### 7. Hot-path benchmark and allocation check

- `criterion` as a dev-dependency, `benches/solve.rs`. Scenarios: a closed polygon of `N ∈ {10, 100, 500}` segments with endpoint Congruence, alternating Orthogonal/Parallel, EqualLength, and one FixPoint; plus a drag step on one vertex (soft point follow + length floors).
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

- Exact faer version and API for the symbolic/numeric LLT and for scratch sizing. Resolve this when adding the dependency. It does not change the approach.
- Whether faer's numeric LLT can run allocation-free with a caller-provided stack. If not, document it as an exception in the allocation test.
