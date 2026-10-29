# Design

## Context

Target implementation: `impls/rust`, crates `constraint_runtime` (components, scoped solve) and `sketch_session` (command rules). See proposal.md for motivation and the spec deltas for the observable contract.

- `AssembledSystem` stacks a row block for **every** constraint in the document; columns are global State indices. `InteractiveSolve::solve_commit_soft` solves a working copy, canonicalizes, and commits if the **global** structural norm is at most `opts.tol * 10` (1e-5); otherwise it restores a fallback vector. `structural_only` is the same without soft terms.
- `soft_for_drag` puts stay-near terms on every State slot except the dragged class.
- `assemble::is_structurally_satisfied` uses `SOLVE_TOL` (1e-6), a different threshold from the commit gate; nothing calls it.
- `Session::new(doc)` takes the document as is, with no solve. `Session::apply` snapshots the document and restores it on error. The constrain path is: `ConstraintSpec::build` (incl. `already joined`), then `add_constraint`, then `structural_only`, which reports `Unsatisfiable`.
- `Document` keeps `constraints_on_geometry`, and `ConstraintInst::referenced_geometries` lists a constraint's geometries. `point_classes.rs` is the existing pattern for structure-only derived sets.
- LM already handles columns without rows: an unconstrained geometry next to a constrained one gives such columns today, and the normal-matrix pattern always has the diagonal slot.

## Goals / Non-Goals

**Goals:**
- One threshold that both the commit gate and the classification use, so a committed component is never classified as conflicted.
- A solve scoped to components whose results provably leave other components' values bit-identical.
- Session rules from the spec as a few explicit checks around the existing `apply` snapshot.

**Non-Goals:**
- Solving components as separate smaller systems for speed (see D3 alternative, #2).
- Caching components across commands.

## Decisions

### D1 Components in the core, structure-only

New module `constraint_runtime/src/components.rs` (tests in `tests_components.rs`), next to `point_classes.rs`:

```rust
pub struct Component { pub geometries: Vec<GeometryId>, pub constraints: Vec<ConstraintId> }
impl Document {
    pub fn components(&self) -> Vec<Component>;            // union-find over referenced_geometries
    pub fn component_of(&self, g: GeometryId) -> Component;
}
```

Both lists are in id order, and components are ordered by their smallest geometry id, so output is deterministic. Computed on demand: O(geometries + constraint references) per call, negligible next to a solve. No cache, so no invalidation on structural edits.

*Alternative:* store a component id per geometry in `Document` and update it on add/remove. Rejected: removal splits components, so incremental updates need a full recompute anyway, and a stored index is one more thing that undo snapshots must keep consistent.

### D2 One consistency predicate

```rust
pub const COMMIT_TOL: f64 = 10.0 * SOLVE_TOL;  // the existing gate value
impl Document {
    pub fn component_residual(&self, c: &Component, p: &Vector) -> f64;
    pub fn is_consistent(&self, c: &Component) -> bool;   // residual at committed p <= COMMIT_TOL
}
```

The commit gate in `InteractiveSolve` and the classification both use `COMMIT_TOL`. `assemble::is_structurally_satisfied` (threshold `SOLVE_TOL`) has no callers and is removed, so no second threshold remains.

*Alternative:* keep the gate on `opts.tol * 10`. Rejected: then a caller with custom `opts.tol` could commit values that `is_consistent` later calls conflicted. The gate is a document invariant, not a solver option.

### D3 Scoped solve: filter rows, commit only the component's windows

`AssembledSystem` gets a constraint scope (`Option<&[ConstraintId]>`, `None` = all, as today): only those constraints contribute row blocks. Columns stay global. `InteractiveSolve::solve_commit_soft` and `structural_only` take a `&Component`:
- they assemble with its constraints;
- the gate checks `component_residual` of that component;
- on success they copy **only that component's parameter windows** from the working vector into the committed vector, and canonicalize only those geometries;
- on failure they restore the fallback as today.

Copying windows instead of the whole vector makes "other components keep their values" exact by construction. It doesn't rely on LM leaving row-less columns untouched.

`soft_for_drag` limits stay-near terms to the dragged component's geometries. Terms elsewhere would be inert rows that only cost time.

*Alternative:* assemble a reduced system over the component's columns only, with a local-to-global column map. Smaller systems and per-component parallelism are the real performance win (#2). Rejected for now: `BlockSparseMatrix` and the expression Jacobians write global columns, so remapping touches every kind. The scoped row filter gives the same semantics with a contained change, and the reduced form can replace it later behind the same `&Component` API.

### D4 Session rules

Additions to `Session`, all inside the existing `apply` snapshot/rollback:

- **Constrain:**
  1. `build` as today, so resolve errors and `already joined` come first.
  2. Before adding, for each operand geometry: if its component is not consistent, fail with `CommandError::Frozen { name }`, where `name` is that geometry's name.
  3. Add the constraint, then `structural_only` over `component_of(any operand)`, which is the merged component. Failure gives `Unsatisfiable` as today.
- **Delete:**
  1. Before removing, take `component_of` the target (for a constraint, its first referenced geometry) and note whether it is consistent.
  2. Remove.
  3. If it was conflicted: for each surviving geometry of the old component, take its new component once (deduplicated). If that component is not consistent, run one `structural_only` on it and ignore failure.
  4. If the target's component was consistent, no solve is needed: removing constraints cannot violate the others.
- **Rename / SetRole / Create:** unchanged.
- **`drag_begin`:** after resolving the point, if the component of the point's geometry is not consistent, return `CommandError::Frozen`. `DragSession::pointer_down` stores the dragged `Component` and uses it as the scope of every pointer move and of pointer-up.
- **`Session::new(doc)`:** for each component that is not consistent, run one `structural_only`. No history step.
- **`Session::conflicts() -> Vec<Conflict>`** with `Conflict { geometries: Vec<String>, constraints: Vec<String>, residual: f64 }`, using names in the D1 order.

`CommandError::Frozen { name: String }`, displayed as "`{name}` is in a conflicted component". `sketch_ui` only shows errors through `Display`, so it compiles unchanged and needs no new UI code (scope (i); display in #52).

*Alternative for the operand check:* check after adding, with "a constraint touches a frozen component iff the merged component contains a geometry that was conflicted". That is equivalent but needs the before-state again, so it's harder to read.

### D5 Tests from conflicted documents

The session tests build a contradicting `Document` directly (add geometries and constraints without solving, for example the spec's fix + equal length + parallel + perpendicular set) and pass it to `Session::new`. This is the only way a contradiction can enter the system in this change, which also tests the start-up solve. Core tests cover components (`tests_components.rs`) and the scoped solve (`tests_interact.rs`: bit-identical other components, gate per component).

## Risks / Trade-offs

- [A solvable component that the solver can't solve from its current values is classified conflicted and frozen] → Same known limitation as #30. The user resolves it by deleting a constraint. #16 refines detection.
- [Per-command component computation and scoped assembly add work to every constraint command and drag step] → Linear in the drawing size, negligible next to the LM iterations. Reduced systems (D3 alternative) later improve speed.
- [A delete in a conflicted component may change the values of the remaining pieces through the resolving solve] → This is the specified "unfreeze"; undo restores the previous values exactly.

## Migration Plan

No persisted data. The `InteractiveSolve` and `DragSession` signatures change (an added scope argument); their callers are `sketch_session` and the core tests. Rollback is a revert.
