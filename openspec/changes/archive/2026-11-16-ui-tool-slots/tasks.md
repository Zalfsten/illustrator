# Tasks

Implementation: impls/rust (see design.md), on top of `issue-91-ui-model-selection`. Run the commands from `impls/rust`.

## 1. Tools in `ui_model` (ui-model, design decisions 1–3)

- [x] 1.1 Add `tool.rs`:
  - `ToolKind`, `SlotKind`, `Filled`, `name()`, `slots()`;
  - the capability test (`fits(slot, entry, doc)`);
  - `command(doc, filled, look)` for Segment, Join, Parallel, Orthogonal, Fix and Brush (`brush_command`, moved from `CurrentStyle::brush`).

  Add `nearest_line` to `pick.rs`. Verify in `tests_tool.rs` that each tool builds the same command the App builds today:
  - Segment with the look;
  - Join `seg1.end` / `seg2.start`;
  - Parallel / Orthogonal on `.direction`;
  - Fix at the current location;
  - Brush with all four patch fields;
  - an arc does not fit a direction slot ("An arc does not fill a direction slot", capability part);
  - `nearest_line` skips an arc that is nearer than a line.
- [x] 1.2 Extend `UiModel` with `tool`, `filled`, `look`, `set_look`, `tool()`, `prompt(doc)`, `last_detail()`, and the intents `ChooseTool` / `Cancel`, the effects `Issued` / `Slot` / `Refused` and `UiError`:
  - pre-fill;
  - picks per slot;
  - duplicate refusal;
  - issue, stay active, report;
  - Box ignored in tools;
  - `clear()` also ends the tool.

  Verify in `tests_model.rs`:
  - "Parallel asks for two directions";
  - "An arc does not fill a direction slot" (`Refused`, slot empty);
  - "Select first, then Parallel" (one constraint, selection empty, tool still Parallel);
  - "Partly filled from the selection";
  - "Unfitting entries are ignored";
  - "Tool first" (Join by picks);
  - "The same object twice is refused";
  - "Two constraints in a row" (Fix twice);
  - "Rejected command keeps the tool" (Join of two points fixed at different locations → `UiError::Command`, drawing unchanged, slots empty);
  - "Cancel a half-filled tool", and Cancel in Select clears the selection;
  - "Prompt names the next slot";
  - "Same intents, same commands" (two models on clones, the same declarative export).
- [x] 1.3 Update the `ui_model` paragraph in `openspec/config.yaml` (tool.rs, tool state, look, new intents and effects). Verify that `openspec validate ui-tool-slots --strict` passes.

## 2. Desktop bindings and layout in `sketch_ui` (design decision 4)

- [x] 2.1 Move the tools onto the UI model:
  - remove `Tool`, `pick_points`, `pick_geoms`, `segment_clicks`, `handle_click` and "Clear picks";
  - non-Select clicks send `Pick` (12 px), tool buttons and L / J / P / O / X / B send `ChooseTool`, and Esc sends `Cancel` (under `canvas_has_keyboard`);
  - `set_look` from `CurrentStyle` each frame;
  - map `Effect::Issued` / `Refused` / `UiError` to the status texts of design decision 3;
  - paint filled slots (dots, halo).

  Verify by rewriting `segment_tool_draws_with_the_current_style` and `brush_replaces_style_and_overrides` onto intents with their assertions unchanged, and by adding `tests_app.rs` tests:
  - select two segments, then choose Parallel → status `Added par… (…)`;
  - Esc in a half-filled tool returns to Select;
  - `opened_file_replaces_the_drawing_and_resets_picks` now checks that the tool and slots are reset.
- [x] 2.2 Build the layout:
  - menu bar (File, Edit with Undo / Redo, View with Show and Panels);
  - ribbon with tabs and the SKETCH groups SELECT / CREATE / CONSTRAIN / STYLE (Brush, Styles…), plus placeholder tabs; the group row scrolls horizontally in a narrow window;
  - left browser panel (placeholder) and right inspector panel with "Style for new geometry" (the current look as a grid), both toggled in View;
  - bottom status line (zoom · prompt or selection · status; label status and errors on a second line).

  Verify with `cargo build -p sketch_ui`, and that the existing tests still pass.
- [x] 2.3 Update the `sketch_ui` lines in `openspec/config.yaml` (layout, ribbon, shortcuts, tools through `ui_model`). Verify that `openspec validate ui-tool-slots --strict` passes.

## 3. Integration checks

- [x] 3.1 Run `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings` and `scripts/check-wasm.sh`. Verify that all three succeed.
- [x] 3.2 Check the native and web builds manually (needs a person):
  - select-first Parallel and Join;
  - tool-first Fix twice in a row;
  - Segment with the current style;
  - Brush;
  - Esc in a half-filled tool;
  - the L / J / P / O / X / B shortcuts;
  - ribbon tabs, panels toggled in the View menu;
  - status prompt.
  - Checked by the user. The check moved the style settings out of the ribbon into the inspector ("Style for new geometry") and made the ribbon's group row scroll instead of wrap. A real ribbon (icons, priority-based overflow) is #109.

## Workflow follow-up

- Archive the change after review; sync the `ui-model` delta.
- Close #91 when this lands; check it off in #98 and #90. #95 can start (slot forms exist).
