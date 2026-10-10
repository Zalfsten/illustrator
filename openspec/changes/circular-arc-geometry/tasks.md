# Tasks

## 1. Kind skeleton

- [x] 1.1 Add `geometry_kinds/arc.rs` with `Arc` shell + `chart_atoms!` (`center: Point2`, `radius: Length`, `start: HeadingAngle`, `sweep: TurnAngle`) and register in `geometry_kinds/mod.rs` + `geometry_catalog!`; verify `PARAMETER_COUNT == 5` and pack/unpack round-trip in a unit test
- [x] 1.2 Implement `eval_*` / `accumulate_*_jacobian` for curve point, endpoints, tangent/normal, length, start heading, and sweep (analytical); verify FD-matching tests for representative formulas (`cargo test -p constraint_runtime --lib geometry_kinds::arc`)

## 2. Expression registry

- [x] 2.1 Register `Arc` in `register_expr_kinds!` (curve/endpoint/vec/length) and replace empty `HeadingAngleFormulaKeyed` / `TurnAngleFormulaKeyed` with Arc variants + dispatch; verify Segment tests still pass and heading/turn Exprs evaluate for an Arc
- [x] 2.2 Expose `Arc::start_heading_expr` / `sweep_expr` (and rely on `CurveExprs`/`EndpointExprs` blankets); verify evaluate values match chart atoms at a known State

## 3. Mesh

- [x] 3.1 Tessellate `GeometryInst::Arc` into a polyline in `mesh.rs` (segment path unchanged); verify Mesh tests: arc yields multiple edges with shared identity, segment still one edge, endpoint hits use arc start/end

## 4. Docs and export

- [x] 4.1 Update `openspec/config.yaml` / `docs/constraint-mathematics.md` for Arc chart and Mesh multi-edge; re-export `Arc` from `lib.rs`; verify `cargo test -p constraint_runtime --lib` passes fully
