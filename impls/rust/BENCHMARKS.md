# Solver hot-path benchmarks

Bench target: `cargo bench -p constraint_runtime --bench solve` (criterion,
`crates/constraint_runtime/benches/solve.rs`).

Scene (`crates/constraint_runtime/tests/common/staircase.rs`): open staircase
chain of N segments with Congruence joints, Orthogonal between neighbours,
Parallel every 5th pair, EqualLength every 3rd pair, one FixPoint, and a small
deterministic perturbation. Parameters \(n = 4N\).

- `jacobian/N`: one Jacobian fill of the structural system.
- `structural/<backend>/N`: `InteractiveSolve::structural_only` from the
  perturbed start.
- `drag/<backend>/N`: one `DragSession::pointer_move` of the last endpoint by
  (+0.3, +0.2) after an initial structural solve (soft phase + optional polish).

Machine: AMD Ryzen 7 5800X (8 cores / 16 threads), rustc 1.99.0, Linux
Dev Container, release profile (criterion default). Median of criterion's
estimate.

> The Ceres reference backend was removed in change `remove-ceres-backend`.
> Its rows below are historical measurements; the current bench covers the
> sparse LM only.

## Baseline (before sparse-lm-solver)

Source state: commit `9c21447` (dense `Matrix` Jacobian through
`ScatterMatrix`, dense `LmSolver` with Gaussian elimination, Ceres DENSE_QR
default).

| Scenario | N = 10 | N = 30 | N = 100 |
|---|---:|---:|---:|
| `jacobian` (dense, incl. zero-fill) | 0.60 µs | 2.52 µs | 17.9 µs |
| `structural/lm` (dense LM) | 0.146 ms | 7.37 ms | 515 ms |
| `structural/ceres` | 0.292 ms | 4.29 ms | 112 ms |
| `drag/lm` (dense LM) | 5.95 ms | 155 ms | 8.46 s |
| `drag/ceres` | 1.73 ms | 11.6 ms | 335 ms |

Observations: both dense backends grow roughly with \(n^3\) per solve
(\(O(mn^2)\) normal equations / dense QR per iteration). At N = 100 a single
drag frame takes 0.34 s with Ceres and 8.5 s with the dense LM — far outside
interactive budgets. N = 500 was not measured on the dense path.

## After sparse-lm-solver

Source state: change `sparse-lm-solver` applied (contribution-blocked
Jacobian, normal equations assembled from blocks, sparse LM on faer sparse
Cholesky as default; Ceres via `--features ceres`). Ceres is only measured up
to N = 100 (still dense).

| Scenario | N = 10 | N = 30 | N = 100 | N = 500 |
|---|---:|---:|---:|---:|
| `jacobian` (blocked, before → after) | 0.60 → 0.53 µs | 2.52 → 1.67 µs | 17.9 → 5.72 µs | 28.7 µs |
| `structural/lm` (dense → sparse LM) | 0.146 → 0.049 ms | 7.37 → 0.163 ms | 515 → 0.593 ms | 3.49 ms |
| `structural/ceres` (reference) | 0.294 ms | 4.31 ms | 111 ms | — |
| `drag/lm` (dense → sparse LM) | 5.95 → 0.448 ms | 155 → 0.723 ms | 8460 → 1.48 ms | 7.07 ms |
| `drag/ceres` (reference) | 1.73 ms | 11.7 ms | 323 ms | — |

Every drag step in these scenes commits with the sparse LM (the dragged
endpoint follows the pointer exactly, final structural norm ≤ 3.2e-7 for
N = 10…500; checked with a one-off release run of the bench scene).

Assessment:

- Scaling is now roughly linear in N: from N = 100 to 500 (5× parameters) the
  structural solve grows 5.9× and the drag step 4.8×. The dense backends grew
  roughly with \(n^3\).
- At N = 100 a drag frame takes 1.5 ms instead of 323 ms (Ceres) or 8.5 s
  (dense LM). Even N = 500 (2000 parameters) stays at about 7 ms, inside a
  60 fps budget.
- At N = 10 the sparse LM is still about 4× faster than Ceres despite the
  per-solve symbolic factorisation, so caching the symbolic analysis across
  drag frames is not needed for now.
- Jacobian fill got faster because the dense \(m \times n\) zero-fill is
  gone; each block is written once into contiguous storage.

## After review refactor (`BlockSparseMatrix`, no `MatrixAccess` trait)

Same scenes, sparse LM only (2026-10-20). Structure and values in one
`BlockSparseMatrix`, structure built in `SystemEval::new_jacobian`; formula /
constraint code takes `&mut DenseMatrix` directly.

| Scenario | N = 10 | N = 30 | N = 100 | N = 500 |
|---|---:|---:|---:|---:|
| `jacobian` | 0.51 µs | 1.66 µs | 5.55 µs | 28.4 µs |
| `structural/lm` | 0.047 ms | 0.160 ms | 0.590 ms | 3.53 ms |
| `drag/lm` | 0.450 ms | 0.721 ms | 1.46 ms | 7.21 ms |

Unchanged within noise (±3 %) versus the previous section, as expected: the
loops are identical and the trait was only ever instantiated with
`DenseMatrix`. The allocation tests (`tests/alloc_free.rs`,
`solver/tests_normal.rs`) still pass.

Reading block entries as `blk[(i, k)]` instead of flat `values[i * k + a]`
(normal-equation assembly, `transpose_mul`, `add_to_dense`) was re-measured:
unchanged within noise (±2 %; e.g. `drag/lm/500` 7.19 ms).

Iterating block rows as 1-D slices (`Block::row_values`, `chunks_exact`) in
`transpose_mul` / `add_to_dense` instead of `blk[(i, a)]`: also within noise
(`structural/lm/100` 0.588 → 0.576 ms, `drag/lm/30` 0.735 → 0.748 ms,
`drag/lm/500` 7.19 → 7.12 ms). Kept for readability.

## Manual smoke test (`cargo run -p sketch_ui`)

Passed (2026-10-19, by the user): dragging chain vertices and arc endpoints
with the sparse LM default behaves like before — joints and right angles stay
intact, no visible difference or stutter.
