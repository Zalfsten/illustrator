# Tasks

Paths relative to `impls/rust` unless noted. Tests: `cargo test --workspace` from `impls/rust`. Format only touched or new files; no crate-wide `cargo fmt`.

## 1. Crate scaffold

- [ ] 1.1 Create `crates/sketch_session` (D1): `Cargo.toml` depending on `constraint_runtime`, `src/lib.rs` with modules `command` and `session`, and re-exports. Add it to workspace `members` / `default-members` and to `scripts/check-wasm.sh`. Verify that `cargo build --workspace` and `scripts/check-wasm.sh` succeed.

## 2. Commands and errors

- [ ] 2.1 `src/command.rs` (D2): `Command`, `Outcome`, `Family`, `CommandError`, `HistoryError`, with `Display` / `Error` impls naming the error class and object. Verify with `cargo doc -p sketch_session --no-deps` (0 warnings).

## 3. Session: apply

- [ ] 3.1 `src/session.rs` (D3): `Session::new(doc)`, `document()`, `apply` with snapshot / rollback and `execute` for all ten commands (address resolution with family check, optional names, structural solve only for constraint commands, `Removal` names into `Outcome`). Verify with `cargo build -p sketch_session`.
- [ ] 3.2 Tests in `src/tests_session.rs`, one per spec scenario of "Create geometry commands", "Add constraint commands", "Delete, rename and set-role commands", "Commands are atomic", "Drawing stays structurally consistent" and "Command errors are classified": auto name `seg1`; explicit name `fillet`; parallel by direction (check the directions are parallel after apply); wrong family `seg1.start`; delete cascade reports `seg1`, `par1`, `join1`; rename; taken name changes nothing (compare object names and values before/after); parallel + perpendicular gives `unsatisfiable` with no perpendicular constraint and unchanged values (regression for #28); unknown name; `arc1.direction` reports missing capability; `SetRole` on a constraint is `WrongObjectType`. Verify that the tests pass.

## 4. History and drags

- [ ] 4.1 `src/session.rs` (D4, D5): `undo`, `redo`, `can_undo`, `can_redo`, `drag_begin` / `drag_to` / `drag_end` / `drag_cancel` / `is_dragging`; `apply` returns `Busy` while dragging, and `undo` / `redo` return `HistoryError::Busy`. Verify with `cargo build -p sketch_session`.
- [ ] 4.2 `crates/constraint_runtime/src/interact.rs`: remove `DragSession::undo_stack` and `undo()` (D5); reduce the drag test in `src/tests_interact.rs` to checking the drag result. Verify `cargo test -p constraint_runtime` is green.
- [ ] 4.3 Tests in `src/tests_session.rs` for "Undo and redo" and "Drags share the command history": undo/redo of a join restores constraints and values exactly; a failed command creates no step; a new command clears redo; nothing to undo / redo; drag then create segment then undo twice (regression for #27, which panicked before); join then drag, undo order; cancelled drag creates no step; commands and undo rejected while dragging. Verify that the tests pass.

## 5. GUI

- [ ] 5.1 `crates/sketch_ui` (D6): depend on `sketch_session`; `App` holds a `Session`; Segment, Join, Parallel/Orthogonal and FixPoint tools issue commands built from `point_address` / `vec_address`; drags go through the session; Undo and Redo buttons replace "Undo drag". Status shows `Added <name>` or the command error. Verify with `cargo build -p sketch_ui`, `cargo clippy -p sketch_ui`, `scripts/check-wasm.sh`, and that `grep -n "add_constraint\|add_geometry\|structural_only\|DragSession" crates/sketch_ui/src/app.rs` only hits the sample scene in `with_sample` / `add_fixed_length_gauge`. Manual smoke test by the user: draw, join, parallel, drag, then Undo/Redo through all of it; a contradicting constraint shows the `unsatisfiable` message and leaves nothing behind.

## 6. Docs

- [ ] 6.1 `openspec/config.yaml` context: add the `sketch_session` crate (Session, commands, history; GUI edits only through it) and drop the drag undo stack from the `DragSession` description if mentioned. `impls/rust/README.md`: mention the new crate if it lists crates. Verify `openspec list --json` parses and `openspec validate command-session --strict` passes.

## 7. Integration

- [ ] 7.1 Run `cargo test --workspace`, `cargo clippy --workspace --all-targets` (0 warnings), `cargo doc --workspace --no-deps` (0 warnings), `cargo bench -p constraint_runtime --bench solve --no-run`, and `scripts/check-wasm.sh`. Verify that all pass.

## Workflow follow-up

- After merge: archive the change, close #32, #27 and #28, and tick #32 in #17.
