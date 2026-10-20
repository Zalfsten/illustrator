# Design

Target implementation: `impls/linux-rust` (crates `constraint_runtime`, `sketch_ui`). Requires `arc-radius-canonical` applied first.

## Context

See proposal.md (Why). Current state:

- `geometry/capabilities.rs`: kind-agnostic formula tags (`PointFormula::{OnCurve, Start, End}`, `VecFormula::{TangentAt, NormalAt}`, `LengthFormula::{Length, Radius}`, `HeadingFormula::StartHeading`, `TurnFormula::Sweep`) and opt-in factory traits `CurveExprs` / `EndpointExprs`. A kind that does not support a tag handles it with an `unreachable!` arm (e.g. `Segment` for `LengthFormula::Radius`). The factory traits make such expressions unbuildable *for statically known kinds*.
- `Arc` has inherent factories `start_heading_expr`, `sweep_expr`, `radius_expr`. There is no center expression.
- `constraint_kinds/parallel.rs`, `orthogonal.rs`: `Parallel` / `Orthogonal { a: VecExpr, b: VecExpr }`, residual cross / dot product, listed in `constraint_kinds/catalog.rs`.
- `sketch_ui/src/app.rs`: the Parallel / Orthogonal tools build `Segment::tangent_expr(id, Normalized(0.0))` for **any** picked edge id. Picking an arc silently constrains the arc tangent at `t = 0`. The demo scene uses arc tangents at `t = 0 / 1` for a fillet (intended pointwise tangency) and a segment–segment Orthogonal.
- Bench scene (`tests/common/staircase.rs`) and acceptance tests use segment `tangent_expr` for Parallel / Orthogonal.

## Goals / Non-Goals

**Goals:** invariant expressions reachable kind-agnostically; names that say what the vector constraints do; no change in residual numerics.

**Non-Goals:** named curve-relation constraints, Collinear, free curve parameters (see proposal).

## Decisions

### 1. New formula tags

- `VecFormula::Direction`. Segment: `b − a`, Jacobian `[[-1,0,1,0],[0,-1,0,1]]` (same as the segment tangent, but independent of `t`). Arc: `unreachable!`.
- `PointFormula::Center`. Arc: `(cx, cy)`, formula-local params `center.range()` (2 columns, identity Jacobian, sparse like `Start`). Segment: `unreachable!`.
- `LengthFormula::Radius` stays. Its value `|R|` and `signum` Jacobian come from `arc-radius-canonical`. Its doc changes from "signed radius chart atom" to "circle radius (magnitude)".

*Alternative:* reuse `TangentAt { Normalized(0.0) }` as the segment direction. Rejected, because it encodes "constant tangent" as a coincidence and lets callers pass any curve.

### 2. Capability traits

```rust
pub trait LineExprs: CurveExprs {
    fn direction_expr(id: GeometryId) -> VecExpr { VecExpr::new(id, VecFormula::Direction) }
}
pub trait CircularExprs: CurveExprs {
    fn center_expr(id: GeometryId) -> PointExpr { PointExpr::new(id, PointFormula::Center) }
    fn radius_expr(id: GeometryId) -> LengthExpr { LengthExpr::new(id, LengthFormula::Radius) }
}
impl LineExprs for Segment {}
impl CircularExprs for Arc {}
```

`Arc::radius_expr` (inherent) moves into `CircularExprs`. Callers need the trait in scope; the `lib.rs` re-exports include both traits. `Arc::start_heading_expr` / `sweep_expr` stay inherent: they are chart quantities of this kind, not invariants shared by a family.

### 3. Runtime capability query for dynamic ids

Factory traits protect statically typed code only. Tools work with picked `GeometryId`s. Add `GeometryKind::has_line_exprs()` / `has_circular_exprs()` in `geometry_kinds/catalog.rs` (single match next to `with_kind!`). Tools use them to reject unsuitable picks with a status message instead of panicking in an `unreachable!` arm. The query lives in the catalog, so constraints and Interact still never match on kinds.

*Alternative:* derive the flags from trait impls via `with_kind!` and a helper trait with an associated const. Possible, but more machinery than two `matches!` lines for two kinds today.

### 4. Rename the vector constraints

`Parallel` → `VectorsParallel`, `Orthogonal` → `VectorsOrthogonal`. Files become `constraint_kinds/vectors_parallel.rs` / `vectors_orthogonal.rs` (`git mv`, to keep history). Update the catalog list, `mod.rs`, the `lib.rs` re-exports and all uses (tests, bench scene, `sketch_ui`). Residuals and Jacobians are unchanged. There is no persisted file format yet, so no migration is needed.

### 5. UI and scenes

- Parallel / Orthogonal tools: accept two picks with `has_line_exprs()` (otherwise status "Parallel/Orthogonal needs two line segments"), then add `VectorsParallel` / `VectorsOrthogonal` on `Segment::direction_expr`. Tool labels stay "Parallel" / "Orthogonal": for lines that is the correct user-facing word.
- Demo scene: the fillet tangency stays pointwise on arc tangents (`t = 0 / 1`, intended). The segment–segment Orthogonal uses directions.
- Bench scene and acceptance tests: switch segment–segment uses to `direction_expr` (numerically identical). Keep one existing test with `tangent_expr` on segments to document pointwise use.

### 6. Documentation and spec Purpose

- `docs/constraint-mathematics.md`: a section "Constraint scope" (pointwise / existential / universal) and "Curve invariants" (line direction, circle center / radius), plus the rename.
- `openspec/config.yaml` context: catalog names, capability traits.
- `openspec/specs/constraint-catalog/spec.md` **Purpose** names "Parallel, Orthogonal". Purpose is not part of a delta, so it is edited directly in the main spec during apply.

## Risks / Trade-offs

- [Rename churn across crates and specs] → mechanical. Done with `git mv` plus sed-style replacement, verified by the build and `grep -rn "\bParallel\b\|\bOrthogonal\b"` (UI labels and prose excepted).
- [`unreachable!` on mismatched dynamic ids] → the runtime capability query (decision 3) guards the tools. Programmatic misuse with wrong ids still panics, as today for `Radius` on segments.
- [Spec deltas touch 11 requirements for a name] → only text changes. Scenario names are kept to preserve identity (the validator treats renamed scenarios as removed).

## Migration Plan

Code and spec rename in one change. Rollback: revert.
