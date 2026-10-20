# Tasks

Paths relative to `impls/linux-rust/crates/constraint_runtime` unless noted. Tests: `cargo test --workspace` from `impls/linux-rust`.

## 0. Precondition

- [ ] 0.1 Confirm `arc-radius-canonical` is applied: `LengthFormula::Radius` evaluates to `|R|` and committed arcs have `R ≥ 0`. Verify by its tests in `src/geometry_kinds/tests_arc.rs` passing. If it is not applied, stop and apply it first.

## 1. Invariant formulas and capability traits

- [ ] 1.1 Add `VecFormula::Direction` and `PointFormula::Center` to `src/geometry/capabilities.rs` (rustdoc: curve invariants, independent of `t`), and the traits `LineExprs` (`direction_expr`) and `CircularExprs` (`center_expr`, `radius_expr`) with default factories. Update the module doc list. Verify with `cargo check -p constraint_runtime` after 1.2.
- [ ] 1.2 Implement the formulas: `Segment` handles `Direction` (`b − a`, all 4 params; `Center` → `unreachable!`), `Arc` handles `Center` (`center.range()`, identity Jacobian; `Direction` → `unreachable!`). Add `impl LineExprs for Segment {}` and `impl CircularExprs for Arc {}`, remove the inherent `Arc::radius_expr`, and re-export both traits from `src/lib.rs`. Verify that `cargo test -p constraint_runtime` is green (existing `radius_expr` callers compile via the trait).
- [ ] 1.3 Unit tests (delegation candidate for the local model: fixed API, finite-difference oracle): in `src/geometry_kinds/tests_segment.rs`, direction equals `end − start` and equals the tangent at t = 0, 0.5, 1, and its Jacobian matches finite differences; in `src/geometry_kinds/tests_arc.rs`, center equals the chart center, its `param_indices` are the two center slots, its Jacobian matches finite differences, and every on-curve sample lies at `radius_expr` distance from `center_expr`. Verify that the tests pass.
- [ ] 1.4 Add `GeometryKind::has_line_exprs()` / `has_circular_exprs()` in `src/geometry_kinds/catalog.rs` with a unit test (Segment: line only; Arc: circular only). Verify that the test passes.

## 2. Rename the vector constraints

- [ ] 2.1 `git mv` `src/constraint_kinds/parallel.rs` → `vectors_parallel.rs` and `orthogonal.rs` → `vectors_orthogonal.rs`; rename the types to `VectorsParallel` / `VectorsOrthogonal` and update `constraint_kinds/mod.rs`, `catalog.rs`, `src/lib.rs`, and all uses in `src/`, `tests/`, `benches/`, `crates/sketch_ui`. Switch segment–segment uses in the bench scene and acceptance tests to `direction_expr` (keep one acceptance use of segment `tangent_expr` to document pointwise use). Verify `cargo test --workspace` is green and `grep -rnw "Parallel\|Orthogonal" impls/linux-rust/crates --include=*.rs` only hits UI tool labels / enum variants.

## 3. UI

- [ ] 3.1 `crates/sketch_ui/src/app.rs`: Parallel / Orthogonal tools accept only picks with `has_line_exprs()` (status message otherwise) and add `VectorsParallel` / `VectorsOrthogonal` on `Segment::direction_expr`; the demo scene's segment–segment constraint uses directions, and the fillet tangency stays on arc tangents. Verify `cargo build -p sketch_ui`, plus a manual smoke test by the user: picking an arc for Parallel shows the message and does not panic; parallel / orthogonal segments still behave as before.

## 4. Docs and spec Purpose

- [ ] 4.1 `docs/constraint-mathematics.md`: add "Constraint scope" (pointwise / existential / universal, with examples: arc tangent at t = pointwise; concentric = Congruence of centers = universal) and "Curve invariants" (line direction, circle center / radius), and rename the constraints. `openspec/config.yaml` context: new catalog names and capability traits. Main spec `openspec/specs/constraint-catalog/spec.md`: update the Purpose sentence to the new names. Verify `grep -rnw "Parallel\|Orthogonal" docs openspec/config.yaml openspec/specs` only hits intentional prose (e.g. "parallel lines").

## 5. Integration

- [ ] 5.1 Run `cargo test --workspace`, `cargo clippy --workspace --all-targets` (0 warnings), `cargo doc -p constraint_runtime --no-deps` (0 warnings), `cargo bench -p constraint_runtime --bench solve --no-run`, and `openspec validate curve-invariants-and-relations --strict`. Verify that all pass.

## Workflow follow-up

- Archive and sync the deltas (`constraint-catalog`, `geometry-ir`, `geometry-eval`, `solve-interaction`).
- Later: named relation tools in the UI (concentric, equal radius) built on these invariants; Collinear (needs a cross-geometry quantity); existential scope via the point-on-curve issue.
