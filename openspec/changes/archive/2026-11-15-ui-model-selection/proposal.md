# Proposal: A shared selection in a device-neutral UI model

## Why

The App is tool-first only: each tool collects clicks in its own pick lists and issues a command when it has enough (#91). Nothing stays selected after a tool, so delete, rename, role changes and the planned inspector (#93) and browser (#92) have nothing to act on. All of it is written against egui and the mouse.

This change is the first half of #91. It adds the selection and a toolkit-free UI model that receives device-neutral intents. The second half follows as its own change: tools as slot forms that pre-fill from the selection, and the ribbon / panel layout.

The product goal is figures whose relationships survive edits. Editing an existing figure starts with selecting what to change, so the selection is the base for every later editing feature of the UI plan (#90).

## What Changes

- **UI model (new, toolkit-free):** owns the selection and takes intents instead of mouse events:
  - pick at a world position with a tolerance;
  - toggle;
  - box select;
  - clear;
  - delete the selection.

  The front end translates its own input device into these intents and chooses the hit tolerance.
- **Selection:** one selection shared by every view.
  - **What it holds:** geometries, constraints and logical points (a congruence class). Entries are kept by object identity, not by screen objects, and are shown by name or address.
  - **Across edits:** the selection follows renames and keeps objects across undo / redo while they exist. It drops deleted objects, and New / Open clear it.
- **Picking:** at a position, a logical point wins over a curve.
  - A plain pick replaces the selection, and a toggle pick adds or removes one entry.
  - A plain pick on empty space clears the selection.
- **Box selection** selects the visible geometries lying completely inside the box, replacing or adding to the selection.
- **Delete the selection** as **one** undo step: the selected geometries (with their constraints), the selected constraints, and for a selected point every curve that ends in it (its congruence class).
- **Command groups (session):** a list of commands applied atomically as one history step. If one command fails, the drawing stays unchanged. Delete-selection uses this.
- **Hidden objects:** an object hidden by the presentation mode is not pickable **on the Canvas**. It can still be selected from views that list it by name (#92). Switching the presentation mode drops newly hidden objects from the selection.
- **Desktop bindings** (impls/rust, Select tool):
  - click selects; Shift/Ctrl+click toggles;
  - a click on empty space or Esc clears;
  - a primary drag on empty space draws the selection box;
  - a primary drag on a point still drags it;
  - Delete / Backspace deletes the selection;
  - selected objects are highlighted, and the status row lists them.

## Capabilities

### New Capabilities

- `ui-model`: the toolkit- and device-independent interaction model of a front end. This change adds the shared selection, picking, box selection, delete-selection and intents. Tool slots follow in the second half of #91.

### Modified Capabilities

- `system-architecture`: the presentation-mode requirement says "not pickable **on the Canvas**". Other views may select hidden objects, and a mode switch drops newly hidden objects from the selection.
- `sketch-commands`: a command group is applied atomically as one history step.

## Non-goals

- **Tools as slot forms, select-first tools, Esc to cancel a tool, the ribbon and the panels:** these come in the second half of #91. The existing tools and their pick lists ("Clear picks") stay as they are until then.
- **Selecting constraints and labels on the Canvas:** neither has a hit area yet (constraint glyphs #94, label boxes). They are selectable through the UI model, which the browser (#92) will use.
- **Hover highlight and the "active candidate" among overlapping handles:** these belong to canvas feedback (#94 / #47).
- **Crossing box selection** (right-to-left) and box selection of points.
- **Rename, role and style of the selection:** these come with the inspector (#93).
- **Touch front end:** the intents make it possible, but this change builds only the desktop bindings.

The specs stay shared across implementations. The intents, the selection rules and the command group are capability contracts. The crate layout, the key bindings, the highlight look and the hit tolerances are design decisions of the desktop front end.

## Impact

- **impls/rust:**
  - **new crate `ui_model`:** selection, picking over the Mesh and logical points, box selection, delete-selection and intents. It depends on `sketch_session` and `constraint_runtime`, not on egui;
  - **`sketch_session`:** `Session::apply_group`;
  - **`sketch_ui`:**
    - `app.rs`: the Select tool's input is translated to intents, with the selection highlight, the status listing, Esc and Delete;
    - the existing hit tests move into `ui_model`.
- **Tests:**
  - `ui_model` unit tests, headless;
  - `sketch_session` tests for groups;
  - `tests_app.rs` for the bindings that need the App.
- **Docs:** `openspec/config.yaml` (new crate, session group).
- No change to the file format, the export or the solver.
