# Proposal

## Why

Figures must stay editable by GUI, scripts and AI clients alike (#17). Today the GUI edits the document directly: each tool mutates `Document`, calls a solve and reports, and only drags can be undone. This has two visible defects. A failed solve keeps the unsatisfiable constraint in the drawing (#28), and "Undo drag" panics after a structural edit (#27). It also blocks every non-GUI front-end, because there is no single place that defines what an edit is.

This change (GitHub #32, first slice of #17) introduces the command layer: a session that applies named, atomic commands, solves eagerly, and keeps one undo/redo history for all edits including drags. The GUI becomes its first client. Later slices add the JSON protocol (#33) and scripting (#34) on top of the same session.

## What Changes

- **Command set** of this slice, addressing objects by name and address (object-naming):
  - create a segment or an arc (optional name, role);
  - add a constraint of each kind: join (two points), parallel and perpendicular (two free vectors), fix (point to coordinates), equal length (two lengths), each with an optional name;
  - delete an object (geometry deletion cascades to its constraints), rename, set role.
- **Atomic commands:** a command either succeeds completely or leaves the drawing (structure, values, names) unchanged and returns a structured error.
- **Eager consistency:** after every successful command the drawing satisfies its structural constraints; otherwise the command fails with class `unsatisfiable` and is rolled back (decision #30; rank analysis #16 refines this later).
- **Undo/redo** for every successful command, plus completed drags, in one history. A failed command creates no undo step. A new command clears redo.
- **GUI on commands:** tools turn picks into commands (names and addresses as text); drags run through the session; "Undo drag" becomes Undo/Redo.
- **Fixes** #27 and #28.
- **BREAKING (Rust API only):** `DragSession` loses its own undo stack. History lives in the session.

## Capabilities

### New Capabilities

- `sketch-commands`: the command layer of a drawing session: command set of this slice, addressing by name, atomicity, eager structural consistency, error classes, undo/redo history.

### Modified Capabilities

- `system-architecture`: "App owns presentation and hosts Interact" becomes "App owns presentation and edits through the command session". The App changes the drawing only through commands, and Interact is hosted by the session (RENAMED + MODIFIED).

## Non-goals

- JSON/serde, JSON Schema, queries, structured error payloads for remote clients (#33).
- Transactions, `move` / `strict move`, drag as a command, `set_parameter` (#34, #31). A drag is a session operation here, not a command.
- Conflict and redundancy classification by rank (#16).
- Kind-specific command parameters supplied by the kinds themselves (#24). Here the create commands are written out per kind.
- Logical-point computation in the core (#35, #36).

Specs stay shared across implementations; the crate, the Rust enum shape and the drag API are design decisions of `impls/rust`.

## Impact

- New crate `impls/rust/crates/sketch_session` (depends on `constraint_runtime`), added to the workspace and built for wasm through `sketch_ui`.
- `constraint_runtime`: `DragSession` without its undo stack (`interact.rs`, `tests_interact.rs`).
- `sketch_ui`: `App` holds a `Session` instead of `Document` + `InteractiveSolve` + `DragSession`. Tools issue commands; Undo/Redo buttons.
- Specs: new `sketch-commands`; delta for `system-architecture`.
