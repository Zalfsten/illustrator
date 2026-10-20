# Proposal

## Why

The catalog's "Parallel" and "Orthogonal" read like relations between **curves**, but they constrain two **free vectors**. Applied to segment tangents they mean "lines parallel" only by coincidence (a segment's tangent is constant). Applied to an arc tangent they mean "aligned at this one curve location", and nothing warns the user. For figures in publications and teaching (constructions that must survive edits), users need relations with their geometric meaning: parallel lines, concentric arcs, equal radii.

Discussion showed that these relations need no new constraint machinery. They are **equality / vector constraints on the right quantities**: direction of a line, center and radius of a circle. Those quantities are invariants of the curve: they depend neither on the curve location `t` nor on the chart. Two pieces are missing today:

- there is no center expression,
- quantities are only reachable through kind-specific methods (`Arc::radius_expr`) or by accident (`tangent_expr` of a segment at an arbitrary `t`).

We also lack vocabulary for *where* a constraint holds: at a fixed curve location, at some location, or along the whole curve.

## What Changes

- **Terminology:** the constraint catalog distinguishes **pointwise** (fixed curve location), **existential** (some location; not yet supported) and **universal** (whole curve) scope. Today's catalog is pointwise. Universal relations between curves are expressed as pointwise constraints on curve invariants.
- **BREAKING (catalog names):** `Parallel` → **`VectorsParallel`**, `Orthogonal` → **`VectorsOrthogonal`**. Semantics unchanged (two free vectors linearly dependent / perpendicular).
- **Curve invariant capabilities** (opt-in per kind, like the endpoint capability):
  - **line capability:** direction free-vector expression (segment: `end − start`, independent of `t`);
  - **circular capability:** center point expression and radius length expression (arc: chart center, `|R|`).
- Relations are expressed with the existing catalog: parallel / perpendicular lines = `VectorsParallel` / `VectorsOrthogonal` on directions; concentric = `Congruence` of centers; equal radius = `EqualLength` of radii. The UI's Parallel / Orthogonal tools and the demo scene use the direction expressions.

## Capabilities

### New Capabilities

- (none)

### Modified Capabilities

- `constraint-catalog`: rename the two vector constraints (RENAMED + MODIFIED), add the constraint-scope terminology (ADDED), update every requirement that names them.
- `geometry-ir`: add the line and circular invariant capabilities (ADDED); update "On-curve point expressions and frame vector expressions" for the new constraint names.
- `geometry-eval`: update the two requirements that name Parallel / Orthogonal.
- `solve-interaction`: update the two requirements that name Parallel / Orthogonal.

## Non-goals

- Named curve-relation constraints (`ParallelLines`, `Concentric`, `EqualRadius`, …). Relations are compositions of existing constraints. Storing user intent ("concentric") can come with UI work later.
- **Collinear:** it needs a cross-geometry quantity (point of one line on the carrier of another), i.e. a new constraint or composite expressions. Deferred.
- Existential scope (free curve parameter): see the point-on-curve issue. This change only names the concept.
- Curve families / chart switching (issue #1) and presolve elimination: unaffected, but the capabilities are designed so further kinds (circle, bulge arc) can implement them.

Specs stay shared across implementations. Trait names and module layout are linux-rust design.

## Dependencies

Requires `arc-radius-canonical` to be applied first: the circular capability's radius expression is the non-negative `|R|` defined there.

## Impact

- `impls/linux-rust/crates/constraint_runtime`: `geometry/capabilities.rs` (new tags `PointFormula::Center`, `VecFormula::Direction`; traits `LineExprs`, `CircularExprs`), `geometry_kinds/segment.rs`, `geometry_kinds/arc.rs` (`radius_expr` moves into `CircularExprs`), `constraint_kinds/{parallel,orthogonal,catalog,mod}.rs` (rename), `lib.rs`, tests, bench scene.
- `crates/sketch_ui`: tool construction uses `direction_expr`, renamed constraint types; tool labels stay "Parallel" / "Orthogonal" (user-facing, for lines).
- Docs: `docs/constraint-mathematics.md` (scope terminology, invariants), `openspec/config.yaml` context.
- Specs: deltas for `constraint-catalog`, `geometry-ir`, `geometry-eval`, `solve-interaction`. The `constraint-catalog` Purpose names the old constraints and is edited in the main spec during apply.
