# Tasks

## 1. Geom API foundation (constraint_runtime)

- [ ] 1.1 Add `CurveParam::{Normalized, Uniform}` and document Normalized vs Uniform in `geom` module docs; verify `cargo test -p constraint_runtime` still passes
- [ ] 1.2 Implement curve position/tangent/normal for `Segment` at normalized \(t\) (`position_at` / `tangent_at` / `normal_at`) returning `Point2` / `Vec2` from state; verify unit tests for \(t \in \{0, 0.5, 1\}\), tangent length = segment length, and normal = leftward rotate of tangent with equal length
- [ ] 1.3 Expose endpoints via the curve/endpoints API (defaults aligned with \(t=0/1\)); verify existing endpoint tests and `Endpoints`-style callers still compile

## 2. Expressions

- [ ] 2.1 Extend `PointExpr` with on-curve samples (`OnCurve` + `CurveParam`, segment id for now); verify evaluation matches `position_at` for Normalized
- [ ] 2.2 Introduce `VecExpr` with at least `TangentAt` and `NormalAt` (plus segment direction as needed); verify evaluation returns free vectors with the specified length/orientation semantics
- [ ] 2.3 Implement Uniform evaluation for segments as identity with Normalized (value + analytical derivatives used by constraints); verify unit test `Uniform(u) == Normalized(u)` on a segment including a mid-parameter case
- [ ] 2.4 Wire `document` / constraint reference cleanup so new expression variants do not panic on segment remove; verify remove-segment / orphan constraint tests still pass

## 3. Constraints and acceptance

- [ ] 3.1 Ensure Congruence/FixPoint accept on-curve `PointExpr` without catalog changes; verify a unit or acceptance test that Congruence to `Uniform(0.5)` on a segment pins the midpoint
- [ ] 3.2 Run full `cargo test -p constraint_runtime` (lib + acceptance) and fix regressions

## 4. Docs and exports

- [ ] 4.1 Update `docs/constraint-mathematics.md` glossary for normalized vs uniform, tangent/normal length policy (not curvature), and vector expressions; verify terms match the delta specs
- [ ] 4.2 Re-export new public types from `constraint_runtime` `lib.rs` as needed; verify `sketch_ui` still builds (`cargo build -p sketch_ui`)
