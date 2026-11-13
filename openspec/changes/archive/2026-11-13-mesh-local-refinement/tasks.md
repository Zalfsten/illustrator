# Tasks

Implementation: impls/rust, crate `constraint_runtime` (see design.md). Run tests from `impls/rust` with `cargo test -p constraint_runtime`.

## 1. Piece bound query (geometry-ir)

- [x] 1.1 Add `PieceBound { length, curvature }` and `Geometry::piece_bound(t0, t1, c, a)` with a panicking default, documented as: exact length (constant-speed chart) and a curvature upper bound on the whole piece. Implement it for `Segment` and `Arc` (design decision 1). Verify with tests in `geometry_kinds/tests_segment.rs` and `tests_arc.rs`:
  - the spec scenarios (segment of length 4 on [0.25, 0.75]; arc R 2, sweep π on [0, 0.5]);
  - an arc with a non-canonical `R < 0` iterate;
  - an arc with R = 0 (length 0, curvature ∞).
- [x] 1.2 Add `Document::curve_piece_bound(id, p, t0, t1) -> Option<PieceBound>` next to `curve_curvature`, and export `PieceBound` from `lib.rs`. Verify with tests in `geometry/tests_shape.rs`: segment, arc, label → `None`, unknown id → `None`, committed State unchanged.
- [x] 1.3 Extend `geometry_kinds/tests_catalog.rs`: for every curve kind, `piece_bound` does not panic, its curvature bound is ≥ `curvature(t)` at the piece ends and the midpoint, and the 64 piece lengths sum to the kind's length expression within 1e-12 relative. Verify that the test passes.

## 2. Sampler (display-mesh)

- [x] 2.1 Rewrite `place_samples(piece, tolerance, cap)` in `mesh.rs` per design decisions 2–4, where `piece: impl Fn(f64, f64) -> PieceBound`:
  - piece-bound grid with the small-angle / quarter-turn demand;
  - equal shares;
  - the Green bound check with M = 4 sub-pieces, closed-form integrals and ternary search;
  - local halving with the cap and the width floor.

  Remove `Node`, `refine_grid`, `edges_within_tolerance`, `MAX_REFINE_DEPTH`, `REFINE_RATIO` and `REFINE_MIN_EDGES`. Feed `uniform_samples` from `piece_bound(0, 1)`. Update the module and function docs, including dropping the "Limitation" paragraph. Verify that `cargo build -p constraint_runtime` succeeds.
- [x] 2.2 Unit-test the Green bound helper in `tests_mesh.rs`:
  - constant κ gives `κL²/8` at the midpoint;
  - curvature only on the last 1 % of an edge gives a bound below 2 % of `κL²/8`;
  - `B` is concave;
  - each sub-piece integral equals a numeric quadrature.

  Verify that the tests pass.
- [x] 2.3 Move the existing `place_samples` tests in `tests_mesh.rs` (zero curvature, constant curvature, sine period, spike, cap, zero speed) to the piece-closure signature. Synthetic pieces get an exact length and a curvature maximum computed over the piece (analytic where possible). Replace `uniform_kinds_mesh_like_the_general_sampler` by: for segments and arcs, the general sampler gives at least the closed-form count and at most one edge more, without splits. Verify that all mesh tests pass.
- [x] 2.4 Add the #86 cases as `place_samples` tests, each checking an edge-count upper bound of `2 × ΣDᵢ + 12` and that every edge stays within ε at 20 points per edge. Positions come from an exact piecewise-arc evaluator in the test for piecewise-constant curvature, and from numeric integration for the ramp. The cases:
  - half line, half circle (κ = 1, ε = 1e-3);
  - lines of length 2, 10 and 50 followed by a quarter circle;
  - nearly straight (κ = 1e-4) followed by a circle;
  - a curvature ramp from 0 to 1, then constant;
  - the spec scenario "straight stretch beside a curved part" (fewer edges on the straight part than on the circular part);
  - the spec scenario "narrow spike": κ = 1 except κ = 1000 on a piece of width 1e-3 placed between grid nodes, all points within ε.

  Verify that the tests pass.

## 3. Documentation

- [x] 3.1 Update the Mesh row in `docs/constraint-mathematics.md` (piece bounds, Green-function deviation bound, local halving) and the `mesh.rs` / `formula.rs` lines in the project context of `openspec/config.yaml`. Verify by reading against design decisions 1–4.

## 4. Integration

- [x] 4.1 Verify the whole workspace: `cargo test`, `cargo clippy --all-targets` and `scripts/check-wasm.sh` pass in `impls/rust`.
- [x] 4.2 Validate the change: `openspec validate mesh-local-refinement --strict` passes.

## Review follow-ups (PR #89)

- [x] 5.1 Piece length is an upper bound (exact for segment and arc): trait, `Document`, `mesh.rs`, geometry-ir requirement, `config.yaml`, `docs/constraint-mathematics.md`, design decision 1; catalog test checks `Σ ℓ ≥ length` (equality for uniform kinds); tests `green_bound_grows_with_piece_lengths`, `over_estimated_lengths_stay_within_tolerance`.
- [x] 5.2 `deviation_bound` maximizes exactly per piece (ends and vertex) instead of ternary search; test `green_bound_is_the_exact_maximum`.
- [x] 5.3 `debug_assert` on the piece range in `Document::curve_piece_bound`; the unbounded-cell fallback is commented.
- [x] 5.4 Priority refinement when the cap is reached is recorded as a requirement for #81 (design risks).

## Workflow follow-up

- Archive the change after review (`/opsx:archive`), then close #86.
