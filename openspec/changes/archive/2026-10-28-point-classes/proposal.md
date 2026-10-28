# Proposal

## Why

Grabbable UI points are meant to be congruence classes (solve-interaction), but the GUI merges endpoints that are merely close (< 1e-3 world units), so one handle can stand for two unconnected endpoints and a drag tears the drawing apart (#36). The distance rule also hides a second defect: classes are built from **direct** Congruence partners only, so a Join chain of three endpoints would otherwise split into two handles. Snapping (#46) and overlap disambiguation (#47) both need a correct, shared notion of point classes, so it comes first and moves into the core (#35, item 1).

This advances the product goal directly: persistent relationships must survive edits, and a figure whose corners come apart on drag breaks that promise.

## What Changes

- New core query: the congruence classes of point expressions, as the transitive closure of all Congruence constraints, and the logical points of a document under a display filter (one per class with at least one visible curve endpoint, with its members and a deterministic representative). No distance criterion; the result depends only on structure and filter, not on State.
- The GUI takes its handles from that query; its own `logical_points_for` / `congruent` and the distance merge are removed. Handles are rebuilt after commands, undo/redo and filter changes, not on every drag step.
- Dragging a logical point applies the soft mouse target to all members of its class (previously the dragged expression and its direct partners only), so stay-near no longer fights longer Join chains.
- A join command whose two addresses already belong to one class (including a point with itself) is rejected with a new error class `already joined`; today the GUI can produce `Join seg1.end = seg1.end` when two endpoints coincide.

## Non-goals

- Snapping when drawing (#46), ring display, cycling, highlighting and the selection menu for overlapping handles (#47, #48). After this change two coincident unconnected endpoints are two handles drawn at one spot; picking keeps the current nearest-first rule.
- Moving the sample scene out of the App (#35 item 3, #5) and the remaining App checks (#35 item 2).
- Degeneration of a segment whose own endpoints are joined (#39/#40).
- Presolve elimination of Congruence (#2); the class query is a structural query and does not change assembly.

Specs stay shared across implementations; the Rust crates are one implementation of them.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `constraint-catalog`: adds an enumerable congruence class (transitive closure) next to the direct-partner enumeration.
- `solve-interaction`: "Logical UI points are congruence classes" becomes precise: transitive classes, no distance merge, independent of State, soft mouse target over the whole class.
- `sketch-commands`: a join within one class is rejected; the error classes gain `already joined`.

## Impact

- `constraint_runtime`: new module for point classes (query on `Document`, uses the existing Congruence index), re-exports; `interact.rs` mouse-target set; tests.
- `sketch_session`: `CommandError::AlreadyJoined`, check in the join command; tests.
- `sketch_ui`: `app.rs` uses the core query; drops `logical_points_for`, `congruent` and per-drag-step rebuilds.
- No change to assembly, solver or the persisted model.
