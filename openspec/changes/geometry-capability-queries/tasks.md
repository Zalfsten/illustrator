# Tasks

Paths relative to `impls/rust/crates/constraint_runtime` unless noted. Tests: `cargo test --workspace` from `impls/rust`.

## 1. Capability set and declaration macro

- [ ] 1.1 Add `Caps` (D1) to `src/geometry/capabilities.rs`: `Copy` struct with `curve`, `endpoints`, `line`, `circular`, `const` constructors/builders, and `Debug`/`PartialEq`. Add trait `GeometryCaps { const CAPS: Caps; }` and make it a supertrait of `Geometry` in `src/geometry/formula.rs`. Verify with `cargo check -p constraint_runtime` after 1.2.
- [ ] 1.2 Add `geometry_caps!` (D2) next to the capability traits. It expands to `impl CurveExprs` (always) plus one marker impl per listed capability (`endpoints`, `line`, `circular`) and `impl GeometryCaps`. Its rustdoc lists the expansion. Replace the hand-written marker impls in `src/geometry_kinds/segment.rs` (`geometry_caps!(Segment: endpoints, line)`) and `src/geometry_kinds/arc.rs` (`geometry_caps!(Arc: endpoints, circular)`), and update the "Adding a kind" docs in `geometry/formula.rs` and `geometry_kinds/catalog.rs`. Verify `cargo test -p constraint_runtime` is green with no other code changes.
- [ ] 1.3 Add `GeometryKind::caps()` via `with_kind!` (D3) and remove `has_line_exprs` / `has_circular_exprs`. Replace their test in `src/geometry_kinds/tests_catalog.rs` with one asserting `Segment` → curve+endpoints+line and `Arc` → curve+endpoints+circular (spec scenarios "Segment/Arc reports its capabilities"). Verify that the test passes. `sketch_ui` stops compiling until 3.1; check with `cargo test -p constraint_runtime` only.

## 2. Identity-based queries and factories

- [ ] 2.1 Add `Document::caps(id) -> Option<Caps>` and the option-returning factories `start_expr`, `end_expr`, `direction_expr`, `center_expr`, `radius_expr`, `on_curve_expr`, `tangent_expr`, `normal_expr`, `length_expr` (D4) in `src/document.rs`. Each checks caps and builds the expression via the same `Expr::new(id, tag)` as the static factory. Verify with `cargo doc -p constraint_runtime --no-deps` (0 warnings).
- [ ] 2.2 Tests in `src/tests_document.rs`: for a segment and an arc, each factory returns `Some(K::x_expr(id))` exactly when the kind has the capability and `None` otherwise. Unknown ids give `caps == None` and `None` from every factory. The direction of a segment evaluates to end − start, and the center of an arc evaluates to the chart center (spec scenarios "Direction from a picked segment / arc", "Center from a picked arc", "Reported set matches the factories"). Verify that the tests pass.

## 3. Callers use capabilities

- [ ] 3.1 `crates/sketch_ui/src/app.rs`: the Parallel/Orthogonal tool checks `doc.caps(id)` for `line` and builds `VectorsParallel` / `VectorsOrthogonal` from `doc.direction_expr(id)`, with no `Segment::` reference in tool code (D7). Feedback texts stay unchanged. Verify `cargo build -p sketch_ui` and `grep -n "Segment::\|Arc::" crates/sketch_ui/src/app.rs` hits only the demo scene and the Segment drawing tool. Manual smoke test by the user: picking an arc for Parallel shows the message, and two segments still become parallel/orthogonal.
- [ ] 3.2 `src/mesh.rs`: replace `append_open_curve_mesh<C: CurveExprs + EndpointExprs>` with `append_curve_mesh<C: CurveExprs>(caps, …)` (D5/D6), called as `append_curve_mesh::<K>(K::CAPS, …)` inside `with_kind!`. First and last vertices come from endpoint expressions when `caps.endpoints`, otherwise from on-curve samples at t = 0 / 1. Endpoint hit targets are pushed only when `caps.endpoints`. Verify existing tests in `src/tests_mesh.rs` pass unchanged.
- [ ] 3.3 Tests in `src/tests_mesh.rs`: building a segment's mesh through the helper with a caps value that has `endpoints` cleared yields edges tagged with its id, whose vertices are on-curve samples. An endpoint hit at its t = 0 location returns no hit, and a geometry hit there returns its id (spec scenarios "Curve without endpoints is tessellated", "Curve without endpoints yields no endpoint hit"). Verify that the tests pass.

## 4. Docs

- [ ] 4.1 Update `docs/constraint-mathematics.md` (capability table: mention the identity-based factories next to the trait names) and the `openspec/config.yaml` context, which still names `GeometryKind::has_line_exprs / has_circular_exprs` for dynamic ids: replace that with capability queries and identity-based factories. Verify `grep -rn "has_line_exprs\|has_circular_exprs" impls/rust docs openspec/config.yaml openspec/specs` returns no hits.

## 5. Integration

- [ ] 5.1 From `impls/rust` run `cargo test --workspace`, `cargo clippy --workspace --all-targets` (0 warnings), `cargo doc -p constraint_runtime --no-deps` (0 warnings), `cargo bench -p constraint_runtime --bench solve --no-run`, and `scripts/check-wasm.sh`. Then run `openspec validate geometry-capability-queries --strict`. Verify that all pass.

## Workflow follow-up

- Close GitHub #22 and tick it in #21 once the change is archived.
