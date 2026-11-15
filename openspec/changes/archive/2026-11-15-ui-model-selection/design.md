# Design: A shared selection in a device-neutral UI model

## Context

See proposal.md for motivation, and the spec deltas (`ui-model`, `sketch-commands`, `system-architecture`) for the contract. The target implementation is impls/rust, on top of branch `issue-67-viewport-navigation` (PR #106), whose view state and input code this change builds on.

- **`sketch_ui/src/app.rs` today:**
  - The Select tool drags a point with the primary button: `hit_point` (12 px) over `logical_points`, then `drag_begin` / `drag_to` / `drag_end`.
  - The other tools run in `handle_click`:
    - Segment collects two clicks in `segment_clicks`;
    - Join and FixPoint collect points in `pick_points` (`hit_end`, `hit_point`, 12 px);
    - Parallel and Orthogonal collect lines in `pick_geoms` (`hit_edge`, 24 px);
    - Brush styles the curve it hits (`hit_edge`, 12 px).
  - "Clear picks" empties all three pick lists.
  - The hit tests read `Mesh::hit_endpoint` / `hit_geometry` and the logical points built under the presentation filter (`PresentationMode::mesh_filter`).
  - `polylines(mesh)` groups Mesh edges per geometry for painting.
- **`sketch_session`:**
  - `Session::apply` snapshots the `Document`, executes the command, then pushes the snapshot on the undo stack, or restores the snapshot on failure.
  - `Command::Delete { name }` deletes one object; deleting a geometry also deletes its constraints, and `Outcome::deleted` names everything that went.
  - Undo and redo swap whole `Document` snapshots, so `GeometryId` / `ConstraintId` stay stable across undo and redo.
- **`constraint_runtime`:**
  - Names: `ObjectId { Geometry, Constraint }` and `Document::lookup(name)`.
  - Points: `Document::point_class(p)`, `logical_points(filter) -> Vec<LogicalPoint { representative, members }>` and `point_address(p)`.

## Goals / Non-Goals

**Goals:**

- **A crate `ui_model` that cannot depend on egui.** Its selection rules are tested headless against a real `Session`.
- **The App's Select tool drives selection only through intents.** Hit tests live in one place and are shared by the selection and the existing tools.

**Non-Goals:**

- Changing the existing tools' behaviour (the second half of #91).
- Moving the presentation mode or the label cache out of `sketch_ui`.

## Decisions

### 1. Crate `ui_model` (toolkit-free)

- **Location:** `crates/ui_model`, depending on `constraint_runtime` and `sketch_session` only.
- **Workspace:** a member and a default member. It is added to `scripts/check-wasm.sh`, because a touch / web front end shares it.
- `sketch_ui` depends on it.

*Alternative:* a module in `sketch_ui`. Rejected: nothing would stop egui types from creeping in, and #91 asked for a model that a second front end can reuse.

### 2. Selection by identity

```rust
pub enum Entry { Geometry(GeometryId), Constraint(ConstraintId), Point(PointExpr) }
pub struct Selection { entries: Vec<Entry> }   // insertion order, no duplicates
```

- **Renames are free:** ids do not change on rename, so the display name follows. Undo and redo keep ids too (whole snapshots).
- **Point entries** keep the `PointExpr` that was picked. Two point entries are equal when one lies in the other's `point_class`. Toggle, contains and dedup all use this class test, so `seg1.end` and a joined `seg2.start` are one entry. The class is computed on demand, because selections are small.
- **`reconcile(&Document)`** keeps the entries whose object exists and drops the rest.
  - A point entry survives while its geometry exists.
  - The App calls it after every `step`, undo, redo and drag end.
  - `clear()` runs on `replaced` (New / Open).
- **Display:**
  - `label(&Document, &Entry) -> String` gives the name, or `point_address` for a point;
  - `describe(&Document) -> String` gives "seg1, seg2.end" for the status row.
- **Other views:** `select_name(&Document, name, mode)` lets a view outside the Canvas (#92) select by name or point address. It ignores visibility (see decision 6).

*Alternative:* store names and addresses as text, as #91 phrased it. Rejected: a rename would need explicit re-mapping in every view, and undo of a rename would leave a stale name behind. Ids give the same observable behaviour, and the spec only asks for identity.

### 3. Intents and the scene

```rust
pub enum PickMode { Replace, Toggle }
pub enum BoxMode { Replace, Add }
pub enum Intent {
    Pick { at: Point2, tolerance: f64, mode: PickMode },
    Box { a: Point2, b: Point2, mode: BoxMode },
    Clear,
    Delete,
}
/// What the Canvas shows and can pick, built by the front end under its filter.
pub struct Scene<'a> { pub mesh: &'a Mesh, pub points: &'a [LogicalPoint] }

impl UiModel {
    pub fn apply(&mut self, intent: Intent, session: &mut Session, scene: &Scene) -> Result<Effect, GroupError>;
}
pub enum Effect { Selection, Deleted(Vec<String>), Nothing }
```

- `UiModel { selection }` is the state. The tool state joins it in the second half of #91.
- **Tolerances** are world lengths. The front end converts its pixel radius with `Viewport::px_to_world`.
- `Scene` keeps the presentation filter on the front-end side. The UI model only sees what is pickable.
- `Scene` holds no `&Document`: the pick functions take the document as an extra argument (`pick(doc, scene, at, tol)`), and `apply` reads it from the session. Otherwise a scene borrowed from the session would block the `&mut Session` that Delete needs (found while implementing).

### 4. Picking and box selection as pure functions

- **`pick.rs`:**
  - `nearest_point(scene, at, tol) -> Option<PointExpr>`, over the logical points' representatives (moved from `App::hit_point`);
  - `nearest_endpoint(scene, at, tol)` (from `hit_end`);
  - `nearest_curve(scene, at, tol) -> Option<GeometryId>` (from `hit_edge`);
  - `pick(scene, at, tol) -> Option<Entry>`: a point first, then a curve.
- **The existing tools** call these functions instead of the old `App::hit_*`. Their radii (12 / 24 px) stay as they are.
- **Box** (`box_select`): a geometry is inside when every vertex of its Mesh polyline lies in the normalized rectangle.
  - The Mesh chords lie inside the convex hull of their curve's vertices, and the vertices lie on the curve. At display tolerance (0.25 px), the vertex test is therefore exact enough.
  - Geometries without Mesh edges (labels) are never box-selected.
  - `polylines` moves to `ui_model` as `mesh_polylines(&Mesh)`, and the App paints with it.

### 5. Delete through a session group

- **`Session::apply_group(Vec<Command>) -> Result<Vec<Outcome>, GroupError>`:**
  - It snapshots once, then executes the commands in order with `Mode::Interactive`.
  - On success it pushes one undo step and clears redo. On the first failure it restores the snapshot and returns `GroupError { index, error }`.
  - It returns `CommandError::Busy` while dragging, as index 0.
  - An empty group succeeds without a step.
- **Delete intent:** it builds `Delete` commands for the selected constraints first, then for the geometries. Every name still exists at its turn: deleting a constraint never removes a geometry, and a geometry's cascade only removes constraints that are already gone. A point entry adds the geometries of the members of its `point_class` that are the start or end point of their geometry (`doc.start_expr` / `end_expr`), found through `PointExpr::referenced_geometry` and only if the scene's Mesh shows them, deduplicated. Curves through the point, arc centers, label origins and curves hidden by the presentation mode stay (review of PR #108). (First drafted as "points are skipped"; changed after the manual check at the user's request.)
  - The effect is `Deleted(all deleted names)`, and the selection is reconciled (it ends up empty).
  - With nothing deletable selected, the effect is `Nothing` and no step results.

*Alternative:* `Command::Delete { names: Vec<String> }`. Rejected: it changes the interchange form and the schema. Groups are also what tools and remote control (#19) need for multi-command actions.

### 6. Hiding deselects

- **`UiModel::hide_invisible(doc, filter)`** drops geometry entries that the filter hides, and point entries none of whose class members is a visible endpoint. Constraints stay.
- The App calls it when the presentation mode changes, not on every frame. That way a hidden object selected by name from another view stays selected until the next mode change (system-architecture delta).

### 7. Desktop bindings (Select tool)

The Select tool's gestures:

| gesture (primary button) | intent / action |
|---|---|
| click | `Pick { Replace }`; with Shift or Ctrl / Cmd: `Pick { Toggle }` |
| drag starting on a point (12 px) | point drag, as today; the selection is unchanged |
| drag starting on empty space | box rubber band; on release `Box { Replace }`, with Shift or Ctrl / Cmd `Box { Add }` |
| drag starting on a curve (not a point) | nothing, as today (moving geometries comes later) |

Keys and display:

- **Esc** → `Clear`; **Delete** / **Backspace** → `Delete`, in every tool. They (and `F` from #67) apply when no widget or the Canvas itself has the keyboard focus. `ctx.wants_keyboard_input()` is not usable: in egui 0.29 it is true for any focused widget, and clicking the Canvas focuses it (found in the manual check).
- **Pick tolerance:** 12 px through the viewport.
- **Gesture classification:** the drag start uses `press_origin()`, like the point drag does today.
- **Highlight:** selected curves get a translucent halo (6 px) in a selection colour, painted *under* their stroke so that the curve keeps its own colour and dashes. Selected points get a ring.
- **Box rubber band:** a thin rectangle with a light fill while dragging.
- **Status row:** `Selected: seg1, seg2.end` (nothing when empty). Delete reports `Deleted seg1, par1` through the existing status text.
- **Other tools:** clicks in the other tools do not change the selection yet (the second half of #91).

## Risks / Trade-offs

- **[A box drag on empty space shadows a future "drag to pan" on the primary button]** → navigation has the middle button and the wheel (#67).
- **[A point entry dies with its picked geometry, even if a joined class member survives]** → the join is deleted with the geometry, so the class splits and no single successor exists. Dropping the entry is the predictable choice.
- **[Ids could be reused after a delete]** → `reconcile` runs after every structural step, so no stale id survives until a reuse.
- **[Backspace in the web build]** → browsers no longer navigate back on Backspace, and egui prevents the default for keys it consumes.
- **[Class test on every toggle]** → `point_class` is a short closure walk. Selections hold a handful of entries.
