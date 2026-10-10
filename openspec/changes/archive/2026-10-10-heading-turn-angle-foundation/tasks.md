# Tasks

## 1. Domain types (`geom`)

- [x] 1.1 Replace `Angle` with `HeadingAngle` and `TurnAngle` in `geom.rs` (radians accessors; algebra: heading−heading→turn, heading±turn→heading, turn±turn→turn, neg turn; no heading+heading) and update `lib.rs` re-exports; verify `cargo test -p constraint_runtime --lib geom::` (or full lib) compiles and geom unit coverage for the algebra holds
- [x] 1.2 Update `docs/constraint-mathematics.md` glossary + Point/Vec analogy for heading/turn; verify the glossary names `HeadingAngle` / `TurnAngle` and no longer documents undifferentiated `Angle` as the production DOF

## 2. Chart atoms

- [x] 2.1 Implement `StateAtom` for `HeadingAngle` and `TurnAngle` in `geometry/chart.rs`; remove `StateAtom for Angle`; update `geometry/tests_chart.rs` pack/unpack tests for both types; verify `cargo test -p constraint_runtime --lib geometry::chart::`

## 3. Opaque expressions

- [x] 3.1 Add `HeadingAngleExpr` and `TurnAngleExpr` in `expr.rs` (geometry id + keyed formula fields, `new`, `referenced_geometry`) and re-export from `lib.rs`; verify the crate builds with the new public types
- [x] 3.2 Extend `geometry/expr.rs` dispatch so both families support `evaluate` / `param_indices` / `param_count` / `accumulate_jacobian` (\(1 \times n\)) via kind-keyed enums (empty catalog until a kind registers); verify compile and that Segment registry is unchanged (`cargo test -p constraint_runtime --lib`)

## 4. Project context alignment

- [x] 4.1 Update `openspec/config.yaml` context: Point/Vec-style heading/turn vocabulary; `HeadingAngleExpr` / `TurnAngleExpr` instead of a single `AngleExpr`; geom module list; verify wording matches the implemented names

## 5. Verification

- [x] 5.1 Run `cargo test -p constraint_runtime --lib` and confirm all tests pass after the foundation land
