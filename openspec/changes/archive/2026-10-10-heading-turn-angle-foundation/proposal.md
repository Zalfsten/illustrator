# Proposal

## Why

The next geometry kind (circular arc / Kreissegment) needs chart parameters that are absolute orientations versus free angular displacements — the same absolute/free split already enforced for `Point2` / `Vec2`. Today linux-rust has a single undifferentiated `Angle` DOF type and no typed angle expressions; introducing one generic `AngleExpr` would erase that distinction just as an overloaded “Point2” would. Foundation types and expression families must land before the arc kind.

## What Changes

- **BREAKING (linux-rust API):** Replace undifferentiated `Angle` with two scalar DOF types: `HeadingAngle` (absolute orientation from a fixed null ray) and `TurnAngle` (free angular displacement).
- Add opaque expression families `HeadingAngleExpr` and `TurnAngleExpr` (parallel to `LengthExpr`), with evaluate / connectivity / analytical \(1 \times n\) Jacobian surface — no public formula-kind matching.
- Require meaningful algebra: heading − heading → turn; heading + turn → heading; turn + turn → turn; forbid heading + heading at the type level.
- Document the Point/Vec analogy for angles in shared docs / `openspec/config.yaml` context (typed angle families instead of a single future `AngleExpr`).
- Wire both types as chart `StateAtom`s so a later arc chart can pack `(center, radius, start, sweep)`.

## Non-goals

- Circular-arc / Kreissegment geometry kind, Mesh tessellation, or angle-valued constraint catalog entries (Equal heading, fixed turn, …).
- `mod 2π` wrapping or branch-cut policy inside the DOF types (radians as plain DOFs; arc conventions come with that kind).
- Untyped `ScalarExpr`; length stays `LengthExpr`.
- UI, styles, LaTeX, export.

## Capabilities

### New Capabilities

_(none)_

### Modified Capabilities

- `geometry-ir`: Distinguish absolute heading angles from free turn angles in the domain model; require opaque typed expression families for each (alongside length scalars); retire undifferentiated angle-as-one-kind language where it would allow conflating the two.
- `geometry-eval`: Evaluation and analytical \(1 \times n\) Jacobians for heading-angle and turn-angle expressions under the same production rules as other opaque expressions.

## Impact

- Shared specs: `geometry-ir`, `geometry-eval` deltas; docs / config vocabulary.
- linux-rust `constraint_runtime`: `geom.rs`, `geometry/chart.rs` (`StateAtom`), `expr.rs`, `geometry/expr.rs` dispatch hooks, `lib.rs` re-exports, chart unit tests; no live geometry currently stores `Angle`, so migration is localized.
- Specs remain shared across implementations; this change does not mandate Rust.
- Follow-on change: admit arc kind with chart atoms `start: HeadingAngle`, `sweep: TurnAngle` (plus center/radius) and kind-owned formula factories.
