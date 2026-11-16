# Proposal: Tools as slot forms, select-first, and the desktop layout

## Why

This is the second half of #91 (UI plan #90); the first half (#108) added the shared selection and the UI model.

**Problem:** the tools still collect clicks in private lists inside the egui App (`pick_points`, `pick_geoms`, `segment_clicks`). Because of that:

- they ignore the selection;
- they cannot be driven from another view or device;
- they need a "Clear picks" button.

**Change:** tools become forms with typed slots in the toolkit-free UI model. They fill from the selection (select-first) or from picks (tool-first). The desktop gets the ribbon and panel layout that the browser (#92), the inspector (#93) and the new tools (#95) fit into.

## What Changes

- **Tool slots:** a tool declares slots by capability, not by geometry kind:
  - a free position (Segment);
  - a point (Join, Fix);
  - a direction (Parallel, Orthogonal: lines only);
  - a curve (Brush).
- **Select-first:**
  - Choosing a tool moves fitting selection entries into its slots, in selection order, and clears the selection.
  - If all slots are filled, the command is issued at once.
- **Tool-first:**
  - Picks fill the next empty slot. Only an object of the slot's capability counts, and an object already in a slot is refused.
  - The UI model gives a prompt for the next slot ("Parallel: seg1 → pick the second direction").
- **A finished tool stays active** (Fusion behaviour): after issuing its command, it waits for the next set.
  - A rejected command is reported, the slots are emptied, and the tool stays.
- **Cancel** (Esc on the desktop):
  - in a tool, it ends the tool and returns to Select, dropping filled slots;
  - in Select, it clears the selection.
- **Current look in the UI model:** the style and overrides for new geometry and for Brush move from the egui App into the UI model, because the tools build their commands there.
- **Existing tools move onto slots:** Segment, Join, Parallel, Orthogonal, FixPoint and Brush. **BREAKING (UI only):** "Clear picks" goes away.
- **Desktop layout (impls/rust):**
  - **Menu bar:** File, Edit (Undo, Redo), View; the View menu gains the presentation mode.
  - **Ribbon:** tabs SKETCH / LABELS / FUNCTIONS / DESIGN. SKETCH has the groups SELECT, CREATE, CONSTRAIN and STYLE (Brush, Styles…); the other tabs are placeholders. A narrow window scrolls the group row.
  - **Panels:** a browser panel on the left (placeholder for #92) and an inspector panel on the right. The inspector holds the style for new geometry and Brush (moved out of the tool bar); the selection's properties follow with #93.
  - **Status line** at the bottom: zoom, selection or tool prompt, messages.
- **Desktop shortcuts** (without modifiers, only while no text field has the keyboard): L Segment, J Join, P Parallel, O Orthogonal, X Fix, B Brush, Esc Cancel. F stays Fit.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `ui-model`:
  - **new requirements:** tool slots, select-first pre-fill, slot picks, finished tools stay active, cancel, prompt;
  - **changed requirement:** the intents of "Front ends drive the UI model through intents" now include choosing a tool and cancel.

## Non-goals

- **New tools:** segment chains, arc, equal length and live preview of new geometry (#95); labels (#63); function curves (#84); horizontal / vertical (#97).
- **Snapping** of positions to existing points (#46).
- **A real ribbon** (icon items, priority-based overflow into drop-downs): #109. This change builds only a text-button skeleton.
- **Panel contents:** the browser tree (#92) and the inspector (#93). This change only reserves their place.
- **Context menus, configurable shortcuts and persistence** of layout and tab (#96).
- **Touch front end.**

The specs stay shared across implementations. Slots, pre-fill, picks, staying active, cancel and the prompt are UI-model contracts. The ribbon, the panels, the shortcuts and the tolerances are design decisions of the desktop front end.

## Impact

- **impls/rust:**
  - **`ui_model`:** new `tool.rs` (tool kinds, slot kinds, filled slots, the commands they build), plus the tool state, the look, and `ChooseTool` / `Cancel` intents in `model.rs`;
  - **`sketch_ui`:**
    - `app.rs`: tools go through intents; the pick lists, `handle_click` and "Clear picks" are removed;
    - a new layout with menu bar, ribbon, side panels and status line;
    - `CurrentStyle` feeds the UI model's look.
- **Tests:**
  - `ui_model`: `tests_tool.rs`, plus additions to `tests_model.rs`;
  - `sketch_ui`: `tests_app.rs` for the existing tool tests (Segment style, Brush), moved onto intents.
- **Docs:** `openspec/config.yaml`.
- No change to the session, the file format or the export.
