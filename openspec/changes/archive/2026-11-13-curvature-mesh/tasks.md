# Tasks

Implementation: impls/rust (see design.md). Run tests from `impls/rust` with `cargo test`.

## 1. Curvature query (geometry-ir)

- [x] 1.1 Add `Geometry::curvature(t, chart) -> f64` to `geometry/formula.rs` with the panicking default used for kinds without the curve capability (as `shape`); verify `cargo build -p constraint_runtime` succeeds.
- [x] 1.2 Implement it for `Segment` (0) and `Arc` (`1 / |R|`, `+∞` for `R = 0`); verify new tests in `geometry_kinds/tests_segment.rs` and `geometry_kinds/tests_arc.rs` (segment 0 at t = 0, 0.5, 1; arc 1/R at t = 0, 0.5, 1, also for a non-canonical `R < 0` iterate) pass.
- [x] 1.3 Add `Document::curve_curvature(id, p, t) -> Option<f64>` resolving by identity through `with_kind!`, `None` without the curve capability or for an unknown id; verify tests next to the `curve_shape` tests in `geometry/tests_shape.rs` (or a sibling `tests_*.rs` of the module that hosts it): segment, arc, label → `None`, unknown id → `None`, and the committed State unchanged after the query.

## 2. Sampler (display-mesh)

- [x] 2.1 Implement the private pure function `place_samples(speed, curvature, tolerance, cap)` in `mesh.rs` per design decisions 3–4 (osculating-circle density with the `|γ′| = 0` guard, 64-interval grid with bounded refinement, trapezoid integral, `n = clamp(ceil(I − 1e-9), 1, cap)`, equal-share placement); verify unit tests in `tests_mesh.rs` with synthetic closures: zero curvature → `[0, 1]`; constant curvature → `t_j = j/n` and `n = ceil(|Δθ| / φmax)`; a sine over a full period (equal end tangents, inflection points) → more than one edge and every midpoint-of-edge sample within tolerance of its chord; a curvature spike → denser samples near the spike; cap reached → exactly `cap` edges; zero speed with infinite curvature → one edge, no NaN.
- [x] 2.2 Replace `sample_normalized_params` / `refine_interval` by `place_samples` fed from the `TangentAt { Normalized }` expression (speed) and `Document::curve_curvature`; add the `tolerance` argument to `Mesh::from_document` (asserted finite and > 0); replace `MAX_MESH_SAMPLES` by a documented `MAX_MESH_EDGES = 1024`; remove `MAX_CHORD_TURN` from `mesh.rs` and from the `lib.rs` exports; update the module docs; verify `cargo build` of the workspace fails only at the callers handled in 2.3 and 3.1.
- [x] 2.3 Update `tests_mesh.rs` to pass a tolerance and to the new contract; verify the display-mesh scenarios as tests: equal arc edges with minimal count (one fewer would exceed the tolerance), radius 20 has more edges than radius 0.5, edge count non-decreasing and changing by at most one for sweeps 80° → 100° in small steps, near-full arc (360° minus a small angle) is not a single chord, large arc (R 20, 180°, ε 0.005) within tolerance at sampled points, finer tolerance (ε / 4) gives at least as many edges, segment stays one edge at any tolerance, same options give the same Mesh; the existing hit, filter and endpoint tests still pass.

## 3. App and documentation

- [x] 3.1 In `sketch_ui/src/app.rs` pass `viewport.px_to_world(MESH_TOLERANCE_PX)` with `MESH_TOLERANCE_PX = 0.25` to `Mesh::from_document`; pass a tolerance in `tests_app.rs` and `tests_labels.rs`; verify `cargo test -p sketch_ui` passes.
- [x] 3.2 Update the Mesh row in `docs/constraint-mathematics.md` (chord tolerance, curvature-driven placement, edge cap instead of the max chord turn) and the `mesh.rs` line in the project context of `openspec/config.yaml`; verify by reading that no reference to `MAX_CHORD_TURN` remains (`grep -rn MAX_CHORD_TURN` over the repository finds nothing outside archived changes).

## 4. Integration

- [x] 4.1 Verify the whole workspace: `cargo test` and `cargo clippy --all-targets` in `impls/rust` pass, and `impls/rust/scripts/check-wasm.sh` succeeds.
- [x] 4.2 Verify in the running app (native): a large arc looks smooth at the default view, a near-full arc is drawn as a curve, and dragging an arc's end across 90° sweep shows no jump between coarse and fine; picking an arc still works. *Manually confirmed: large arc smooth, no coarse/fine jump while dragging. Not checkable in the current GUI: near-full arc and arc picking; covered by `near_full_arc_is_not_a_single_chord` and the Mesh hit tests.*
- [x] 4.3 Validate the change: `openspec validate curvature-mesh --strict` passes.

## Workflow follow-up

- Archive the change after review (`/opsx:archive`), then close #75.
