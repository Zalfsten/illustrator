# Proposal

## Why

Expression factories on `Segment` mix three concerns (curve frame, endpoints, segment-only direction) and do not scale to closed curves, which must omit join endpoints. The shared specs already separate Curve vs Endpoints capabilities; the linux-rust API should match. Segment-direction vector expressions are redundant with constant curve tangent on a segment and should be removed so Parallel/Orthogonal use tangent expressions.

## What Changes

- Introduce **capability factory contracts**: Curve expression factories (`on_curve` / `tangent` / `normal`) and Endpoints factories (`start` / `end`). Closed/periodic curves implement Curve only.
- **BREAKING:** Remove segment-direction vector expressions (`direction_expr` / direction formula). Tools and tests use curve tangent expressions instead (any chart parameter on a segment; tangent is constant).
- Align Parallel/Orthogonal catalog wording with free-vector expressions typically supplied as curve tangents, not a privileged direction API.
- linux-rust: traits for the factories; `Segment` implements Curve + Endpoints; drop `VecFormula::Direction`.

## Non-goals

- Implementing circle/arc kinds in this change.
- Changing residual math for Parallel/Orthogonal.
- Making expression factories object-safe on `dyn Geometry`.
- ECS/layout mapper work.

## Capabilities

### New Capabilities

_(none)_

### Modified Capabilities

- `geometry-ir`: Expression factories follow Curve vs Endpoints capabilities; remove segment-direction as a required/privileged expression kind; closed curves omit endpoints factories.
- `constraint-catalog`: Parallel/Orthogonal examples use curve tangent (free-vector) expressions rather than segment-direction factories.

## Impact

- All call sites of `Segment::direction_expr` (UI, acceptance, unit tests) switch to `tangent_expr`.
- Docs/`openspec/config.yaml` factory lists updated.
