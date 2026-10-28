# Design

## Context

Target implementation: `impls/rust` (crates `constraint_runtime`, `sketch_session`, `sketch_ui`).

- `Document` keeps a Congruence index `identification_by_geometry` (geometry → Congruence edges) and exposes `points_identified_with(p)`, the **direct** partners of `p`.
- `sketch_ui/src/app.rs` builds handles in `logical_points_for`: it walks `Mesh::endpoint_exprs()` and drops an endpoint if it is within 1e-3 of, or a direct partner of, an already kept one. It runs after every command, undo/redo, filter change **and every drag step**.
- `interact.rs` `soft_for_drag` puts the dragged expression plus its direct partners under the mouse target and excludes their columns from stay-near.
- The session's join command adds a Congruence without checking whether the two points are already identified.
- `PointExpr` is `Copy + PartialEq` but not `Hash`/`Eq` (it can carry an `f64` curve parameter).

## Goals / Non-Goals

**Goals:**
- One core query for classes and logical points, used by the GUI and by Interact.
- Handles that depend only on structure and filter, so they are computed once per structural change.

**Non-Goals:**
- Caching inside `Document` (a revision counter or memoized classes). The App keeps its list and rebuilds it at structural events, as today.
- Changing `points_identified_with` or the Congruence index.
- Picking rules for overlapping handles (#47).

## Decisions

### D1 New module `constraint_runtime/src/point_classes.rs` with `impl Document`

```rust
/// One grabbable point: a congruence class with a visible curve endpoint.
pub struct LogicalPoint {
    /// First visible endpoint of the class (geometry id order, start before end).
    pub representative: PointExpr,
    /// All expressions of the class, representative included.
    pub members: Vec<PointExpr>,
}

impl Document {
    /// `p` and every expression reachable from it through Congruence.
    pub fn point_class(&self, p: &PointExpr) -> Vec<PointExpr>;
    /// One entry per class that has a curve endpoint visible under `filter`.
    pub fn logical_points(&self, filter: MeshFilter) -> Vec<LogicalPoint>;
}
```

Same pattern as `names.rs` / `address.rs` (`Document` queries in their own module, tests in `tests_point_classes.rs`). Re-export `LogicalPoint` from `lib.rs`.

*Alternative:* a free function taking a `Mesh`. Rejected: the Mesh is evaluated at a State, but the result must not depend on State; building a Mesh just to list endpoint expressions also evaluates every curve.

### D2 Class = breadth-first search over the existing Congruence index

`point_class` starts at `p` and repeatedly adds `points_identified_with(q)` for every new member `q`, until no new member appears. Membership uses `Vec::contains` (`PointExpr` has no `Hash`; classes have a handful of members).

*Alternative:* union-find over all Congruences, rebuilt per query or kept in `Document`. Rejected for now: it needs either a hashable key for `PointExpr` or a second index kept in sync on add/remove/rebuild, for no measurable gain at current sizes. The BFS uses the index that is already maintained.

### D3 Logical points from Document geometries, not from the Mesh

`logical_points` walks `doc.geometries()` in id order, skips construction geometries unless `filter.include_construction` (same rule as `Mesh::from_document`), and takes `start_expr` / `end_expr` for kinds with the endpoint capability. An endpoint already in a collected class is skipped; otherwise its class is computed and it becomes that class's representative.

- Deterministic representative: lowest geometry id, start before end. It is always a visible endpoint, so the GUI can draw and address it.
- A class whose only visible members are non-endpoints (for example an arc center) gets no handle, as today.
- Members may include expressions of hidden geometries; dragging the representative moves them through Congruence, as today.

### D4 Interact uses the class for the mouse target

`soft_for_drag` replaces `points_identified_with(dragged)` with `point_class(dragged)` (which already contains `dragged`). For a single Join nothing changes; for chains stay-near no longer pulls on far members.

### D5 Session rejects a join within one class

In `Command::Join`, after resolving both addresses and before adding the constraint: if `doc.point_class(&a).contains(&b)`, return `CommandError::AlreadyJoined { a, b }` (the address strings). Display: `already joined: {a} and {b} are one point`. Equality is on expressions, so `seg1.start` and `seg1.at(0)` are different expressions and are not caught; that case belongs to degeneration (#39/#40).

### D6 App uses the query and rebuilds only on structural events

- `logical_points: Vec<LogicalPoint>`; painting and `hit_point` use `representative`.
- `logical_points_for` and `congruent` are deleted.
- Rebuild after `run` (commands), undo, redo and presentation-mode change. No rebuild in `drag_to`, `drag_end` or `drag_cancel`: drags change values only.
- `hit_point` keeps nearest-first; for exactly coincident handles the first in the list wins (lowest geometry id). Ring display and cycling are #47.
- The Join tool keeps using `hit_end`; with two coincident unjoined endpoints it still picks the same one twice, which now gives a clear `already joined` message instead of a no-op constraint. Excluding the first pick's class from the second pick is part of #47.

## Risks / Trade-offs

- [Coincident unjoined endpoints become two handles at one spot with no visual cue until #47] → Accepted for this slice; the drag now moves exactly the class the user picked, and Join reports `already joined` instead of silently doing nothing.
- [BFS per call is quadratic in class size] → Classes are tiny; `logical_points` runs only at structural events. Revisit together with the presolve (#2) if large drawings appear.
- [The representative can change after deleting a geometry] → It only names which member is addressed in a drag; all members move together, so the drag result is the same.
- [Hidden-geometry members under the mouse target] → Same as today for direct partners; they are tied by hard Congruence anyway.
