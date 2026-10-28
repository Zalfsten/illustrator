# Design

## Context

Implementation: `impls/rust`. See proposal.md for motivation and the `sketch-commands` spec for behavior. The decisions in #29 (snapshot undo), #30 (`unsatisfiable` + rollback) and the comment on #32 (addressing by text, one variant per constraint kind, the session owns Interact, the drag path, optional names) are inputs, not open questions.

Current state:

- `sketch_ui::App` owns `Document`, `InteractiveSolve` and `DragSession`. Tools call `doc.add_*` followed by `interact.structural_only`. On failure only the vector is restored, so the constraint stays (#28).
- `DragSession` keeps its own `undo_stack: Vec<Vector>`. Restoring a vector of the wrong length panics after a structural edit (#27). Its only users are `App` ("Undo drag") and one test in `tests_interact.rs`.
- `Document: Clone` includes the name table (object-naming). `Document::resolve_address`, `geometry_name`, `point_address`, `vec_address`, `rename` and `remove_*` (returning `Removal`) exist.
- `InteractiveSolve::structural_only` returns `Err(SolveError { residual_norm, .. })` when the structural gate fails.

## Goals / Non-Goals

**Goals:**

- A `Session` type that is the only mutation path the GUI uses, with snapshot-based atomicity and undo/redo.
- No changes to evaluation, assembly or the solver.

**Non-Goals:**

- serde / JSON (#33), transactions and `move` (#34), a bounded or compressed history.
- Building the sample scene through commands. `App::with_sample` still builds the initial `Document` and hands it to `Session::new`; it becomes the session's baseline without a history step (moved to examples in #5 / #35).

## Decisions

### D1: New crate `sketch_session`

The crate lives in `impls/rust/crates/sketch_session` and depends only on `constraint_runtime`. It is added to workspace `members` / `default-members` and to `scripts/check-wasm.sh`. Modules follow the project layout rule: `src/command.rs` (`Command`, `Outcome`, `CommandError`, `HistoryError`), `src/session.rs` (`Session`) and `src/tests_session.rs`.

- *Alternative:* a module in `constraint_runtime`. That would keep the solver crate free of product policy (history, command set) only by convention. A separate crate makes the layering explicit and is the place #33 adds serde to without touching the core.

### D2: Command and result types

```rust
pub enum Command {
    CreateSegment { start: Point2, end: Point2, role: GeometryRole, name: Option<String> },
    CreateArc { center: Point2, radius: f64, start_heading: f64, sweep: f64,
                role: GeometryRole, name: Option<String> },
    Join { a: String, b: String, name: Option<String> },          // point addresses
    Parallel { a: String, b: String, name: Option<String> },      // vector addresses
    Perpendicular { a: String, b: String, name: Option<String> },
    Fix { point: String, target: Point2, name: Option<String> },
    EqualLength { a: String, b: String, name: Option<String> },   // length addresses
    Delete { name: String },
    Rename { name: String, new_name: String },
    SetRole { name: String, role: GeometryRole },
}

pub struct Outcome { pub created: Option<String>, pub removed: Vec<String> }

pub enum CommandError {
    UnknownName(String),
    Name(NameError),                          // invalid / taken
    Address(AddressError),                    // malformed, unknown name, missing capability, …
    WrongFamily { address: String, expected: Family },
    WrongObjectType { name: String, expected: &'static str },
    Unsatisfiable { residual: f64 },
    Busy,                                     // drag in progress
}
pub enum HistoryError { NothingToUndo, NothingToRedo, Busy }
```

- Addresses are plain `String`s, so the GUI builds exactly what a script would send (#32 decision 1).
- `Perpendicular` is the user-facing name for `VectorsOrthogonal` (spec wording).
- An unknown name inside an address surfaces as `Address(AddressError::UnknownName)`; a bare unknown name (`Delete`, `Rename`, `SetRole`) as `UnknownName`. Both belong to the spec's "unknown name" class.
- `Family` is `Point | Vector | Length`, taken from `constraint_runtime::Address`.

### D3: `Session::apply`: snapshot, mutate, solve if needed, commit

```text
apply(cmd):
  if dragging -> Err(Busy)
  snap = doc.clone()
  result = execute(cmd)              // mutates self.doc
  match result:
    Ok(outcome) -> undo.push(snap); redo.clear(); Ok(outcome)
    Err(e)      -> doc = snap; Err(e)
```

`execute` per command:

- **Create:** validate the optional name up front (`is_valid_name`, `lookup`), `add_geometry`, then `rename` if a name was given. No solve: an unconstrained new geometry cannot violate anything.
- **Add constraint:** resolve each address and check its family, `add_constraint`, optional rename, then `interact.structural_only`. `Err(SolveError)` becomes `Unsatisfiable { residual: residual_norm }`.
- **Delete:** `lookup` → `remove_geometry` / `remove_constraint`; the `Removal` names go into `Outcome::removed`. No solve: removing constraints keeps a consistent state consistent.
- **Rename / SetRole:** `Document::rename` / `set_geometry_role`. `SetRole` on a constraint is `WrongObjectType`.

The snapshot restore is what fixes #28: the session never leaves a failed constraint behind, regardless of what `structural_only` restores itself.

- *Alternative:* validate everything before mutating and skip the snapshot. That duplicates the Document's checks and still needs a rollback for `unsatisfiable`.

### D4: History is a pair of `Vec<Document>`

`undo: Vec<Document>` holds the states before each step, and `redo: Vec<Document>` the undone results. `undo()` pushes the current document to `redo` and pops `undo` into the current one; `redo()` mirrors that. Both return `HistoryError::Busy` while dragging. The history is unbounded for now; sketch documents are small (a bounded history is a later, local change).

### D5: Drags go through the session

```rust
pub fn drag_begin(&mut self, point: &str) -> Result<(), CommandError>; // address of a point
pub fn drag_to(&mut self, target: Point2) -> Result<(), SolveError>;
pub fn drag_end(&mut self, target: Point2) -> Result<(), SolveError>;
pub fn drag_cancel(&mut self);
pub fn is_dragging(&self) -> bool;
```

- `drag_begin` resolves the address, which must be a point (`WrongFamily` otherwise), stores `drag_snapshot = doc.clone()` and calls `DragSession::pointer_down`.
- `drag_to` and `drag_end` delegate to `pointer_move` / `pointer_up`.
- On `Ok` from `drag_end`, `drag_snapshot` is pushed to `undo` and `redo` is cleared.
- On `Err`, `DragSession` has already restored the pre-drag vector; no step is created.
- `drag_cancel` restores and drops the snapshot.

`DragSession` loses `undo_stack` and `undo()`. Its own drag semantics (soft targets, congruence partners, fallback) are unchanged. The test in `tests_interact.rs` that used `drag.undo` is reduced to the drag result; history coverage moves to `tests_session.rs`.

### D6: GUI as a session client

`App` holds `session: Session` instead of `doc` / `interact` / `drag`; reads go through `session.document()`. The tools work as follows:

- **Segment:** `CreateSegment { start, end, role: Result, name: None }`, status `Added <created>`.
- **Join:** `Join { a: point_address(a), b: point_address(b) }`.
- **Parallel / Orthogonal:** `Parallel` / `Perpendicular` with `vec_address(direction_expr(id))`. The capability check before the pick stays.
- **FixPoint:** `Fix { point: point_address(p), target: p }`.
- **Drag:** `drag_begin(point_address(handle))`, then `drag_to` / `drag_end` / `drag_cancel`.
- **"Undo drag"** becomes **Undo** and **Redo** buttons, disabled when `can_undo()` / `can_redo()` are false.
- On `Err(CommandError)` the status shows the error's `Display`.

Logical points stay in `app.rs` for now (#35).

## Risks / Trade-offs

- [One `Document` clone per command and per drag] → Sketch documents are small; nothing on the solve path changes. A bounded history can come later.
- [GUI converts expressions to text and the session parses them back] → A negligible cost per click. In exchange, the GUI exercises the same path as scripts, which is the point of #32.
- [`DragSession` API change] → Only `sketch_ui` and one test use `undo`. The change removes the code behind #27 instead of leaving it beside the session.
- [Sample scene bypasses commands] → It is initial content, not an edit. Recorded as a non-goal; #5 / #35 move it.

## Migration Plan

This is an internal API only. There is no persisted data, and a revert is enough for rollback.
