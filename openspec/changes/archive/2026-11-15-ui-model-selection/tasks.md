# Tasks

Implementation: impls/rust (see design.md), on top of `issue-67-viewport-navigation`. Run the commands from `impls/rust`.

## 1. Command groups (sketch-commands, design decision 5)

- [x] 1.1 Add `Session::apply_group(Vec<Command>) -> Result<Vec<Outcome>, GroupError { index, error }>`: one snapshot, commands in order, one undo step on success (redo cleared), snapshot restored on the first failure, `Busy` while dragging, an empty group succeeds without a step. Document it in the `sketch_session` line of `openspec/config.yaml`. Verify in `tests_session.rs`:
  - "Group undoes in one step";
  - "A failing command rolls back the group" (index 1, no step, `seg1` still there);
  - "Later commands see earlier results" (create `a`, rename to `b`);
  - empty group, no step;
  - a group during a drag is `Busy`.

## 2. Crate `ui_model`: selection, picking, intents (ui-model, design decisions 1–6)

- [x] 2.1 Create `crates/ui_model` (depends only on `constraint_runtime` and `sketch_session`). Add it to the workspace members and default members, and to `scripts/check-wasm.sh`. Verify that `cargo build -p ui_model` and `scripts/check-wasm.sh` succeed, and that `cargo tree -p ui_model` lists no egui / eframe.
- [x] 2.2 Add `Selection` / `Entry` (insertion order, class-aware point equality), `reconcile`, `clear`, `label`, `describe` and `select_name`. Verify in `tests_selection.rs` against a real `Session`:
  - "Point shown by address";
  - "Joined ends are one entry";
  - "Rename keeps the entry";
  - "Deleted objects leave the selection" (`seg1` + `par1`);
  - "Undo keeps existing objects selected";
  - `select_name` with a geometry, a constraint and a point address.
- [x] 2.3 Add `pick.rs`: `nearest_point`, `nearest_endpoint`, `nearest_curve`, `pick` (point before curve), `mesh_polylines` and `box_select` (all polyline vertices inside; no labels). Verify in `tests_pick.rs`:
  - "Point wins over curve";
  - "Tolerance comes from the front end";
  - "Only geometries completely inside";
  - a label is never box-selected;
  - a construction segment is not picked when the scene is built under the result-only filter.
- [x] 2.4 Add `UiModel`, `Intent`, `Scene`, `Effect` and `apply`: Pick (Replace / Toggle, empty space), Box (Replace / Add), Clear, Delete (constraints first, then geometries, through `apply_group`; a point deletes the curves of its class; reconcile). Add `hide_invisible(doc, filter)`. Verify in `tests_ui_model.rs`:
  - "Toggle adds and removes";
  - "Replace pick on empty space clears", and a toggle pick on empty space changes nothing;
  - "Add keeps the earlier selection";
  - "One undo step" (delete two segments, one undo restores both);
  - "A point deletes the curves at it" (seg1 + seg2 + join, seg3 stays, one step) and "Empty selection deletes nothing" (no step, `Effect::Nothing`);
  - "Same intents, same selection" (two `UiModel`s on clones of one drawing);
  - "Switching to result only deselects construction" (`hide_invisible`);
  - "Hidden objects stay selectable by name" (`select_name` on a construction segment under result-only).
- [x] 2.5 Document the crate in `openspec/config.yaml` (a `ui_model` paragraph next to `sketch_session`). Verify that `openspec validate ui-model-selection --strict` passes.

## 3. Desktop bindings in `sketch_ui` (design decision 7)

- [x] 3.1 Depend on `ui_model`. Replace `App::hit_point` / `hit_end` / `hit_edge` and `polylines` with the `ui_model` functions, building a `Scene` under the presentation filter when an input needs a hit test (not per frame, which would build the Mesh twice). Keep every tool's radius (12 / 24 px). Verify that the existing `tests_app.rs` tests pass unchanged.
- [x] 3.2 Add `ui: UiModel` to `App`. Call `reconcile` after `step`, undo, redo and drag end, `clear` in `replaced`, and `hide_invisible` when the presentation mode changes. Verify in `tests_app.rs`:
  - "Open clears the selection" (through the fake file host);
  - a mode switch drops a selected construction segment.
- [x] 3.3 Translate Select-tool input into intents:
  - a click picks, Shift / Ctrl / Cmd toggles;
  - a primary drag from empty space draws the rubber band and sends `Box` on release (Add with a modifier);
  - a drag from a point still drags it and leaves the selection unchanged;
  - Esc clears and Delete / Backspace deletes, without keyboard focus.

  Paint the selection highlight (curves re-stroked wider, point rings) and the rubber band, and show `Selected: …` in the status row. Verify with `tests_app.rs` tests driven through the App's intent helper (no egui frame):
  - a click on a segment selects it;
  - a drag on its end point moves the point and keeps the selection;
  - Delete with two selected segments gives one undo step and the status `Deleted …`.

  Also verify that `cargo build -p sketch_ui` succeeds.
- [x] 3.4 Update the `app.rs` line in `openspec/config.yaml`: the Select tool goes through `ui_model` intents, with the highlight, the rubber band and Esc / Delete. Verify that `openspec validate ui-model-selection --strict` passes.

## 4. Integration checks

- [x] 4.1 Run `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings` and `scripts/check-wasm.sh`. Verify that all three succeed.
- [x] 4.2 Check the native and web builds manually (needs a person): click, Shift+click, box from left and right, box with Shift, Esc, Delete + Undo, a point drag that keeps the selection, and that Backspace in the browser does not navigate.
  - Checked natively and in Firefox. The check found that Delete / Backspace (and `F` from #67) were ignored after a click on the Canvas, because egui's `wants_keyboard_input()` is true for any focused widget; fixed with `canvas_has_keyboard`.
  - At the user's request, a selected point now deletes the curves at it (spec, design and tests updated).

## Workflow follow-up

- Archive the change after review; sync the `ui-model`, `sketch-commands` and `system-architecture` deltas.
- Propose the second half of #91 (tool slots, select-first, ribbon / panel layout) as its own change.
- Check off the first half of #91 in #98 and #90.
