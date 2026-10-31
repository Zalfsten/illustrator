# Tasks

## 1. Style model in the runtime (D1)

- [x] 1.1 Add `constraint_runtime/src/style.rs` with `Rgb`, `Dash`, `Arrow`, `StyleProps`, `Style` (with the built-in values), `Styles` (always containing `default`) and `StyleRef`, and export them from `lib.rs`. Verify with `src/tests_style.rs`:
  - a new table holds only an empty `default`;
  - resolution covers the scenarios "Override wins", "Default fills the gaps" and "Built-in look".
- [x] 1.2 Add `style: StyleRef` to `GeometryShell`. Add `styles` to `Document` with `styles()`, `define_style`, `delete_style` (rejects `default`, and rejects a style in use, naming the users), `set_geometry_style`, `geometry_style` and `resolved_style`; check width ranges in the runtime. Verify with `tests_style.rs`, covering:
  - "Changing a style changes its users";
  - "Style in use";
  - "Default stays";
  - "Style and object may share a name";
  - "Restyling does not move anything" (the parameter vector is unchanged);
  - a cloned Document carries the styles.

## 2. Stroke geometry (D2)

- [x] 2.1 Add `constraint_runtime/src/stroke.rs` with `pt_to_world`, `Dash::pattern`, `arrow_head`, `arrow_tips` (exact endpoints and outward tangents through capabilities), `trim` and `dash` by arc length. Verify with `src/tests_stroke.rs`, covering:
  - the spec scenarios "Width in world units", "Head on a segment", "Head on an arc follows the tangent", "Shortened stroke", "Very short arrow" and "Dashed segment";
  - a pattern continuing across polyline vertices ("Pattern continues around an arc").

## 3. Commands and interchange (D3)

- [x] 3.1 In `wire.rs`, add colour (`#rrggbb`, either case in, lower case out), dash and arrow wire forms. Add `StylePatch` and `CommandError::UnknownStyle` / `StyleInUse`. Verify with `tests_wire.rs`: colours in both cases round trip; `#c00` and `c00000` are rejected; unknown dash and arrow values are rejected.
- [x] 3.2 Add `look: StyleRef` to `Command::Create` and `DeclCommand::Create`. Add the style fields to `RESERVED`, and read them in `create` (overrides) and write them (style unless `default`, then the overrides in fixed order). Verify with `tests_json.rs`, covering "Styled segment in interchange form", "Unknown dash pattern is rejected" and "Misspelled colour is rejected", plus a bit-exact round trip of a styled `create`.
- [x] 3.3 Add the commands `define_style`, `delete_style` and `set_style`: interchange, execution as history steps without solving, and validation. Make `create` fail on an unknown style. Verify with `tests_session.rs` and `tests_json.rs`, covering the scenarios of:
  - "Define style command";
  - "Delete style command";
  - "Set style command";
  - "Style values are validated";
  - "Style error classes";
  - "Style commands are history steps";
  - "Create with a style and an override" and "Create with an unknown style";
  - "Set style is outside the subset".
- [x] 3.4 Export the style in `declarative_commands` and add `Session::styles` and `Session::style_of`. Regenerate the schemas with `UPDATE_SCHEMA=1`. Verify with tests for "Style after set style" and "Resolved style of a geometry", with `tests/schema.rs` passing against the regenerated files, and with the schema listing the three new operations.

## 4. File version 3 (D4)

- [x] 4.1 Raise `VERSION` to 3. Read `styles` from the envelope (names, fields and values checked, with errors naming the style) and migrate versions 1 and 2 by adding an empty table. Reject style fields in commands of version 1 and 2 files. Verify with `tests_file.rs`, covering:
  - "Minimal file";
  - "Styles and a styled command";
  - "Invalid style in the envelope is rejected";
  - "Newer version" (version 4);
  - "Version 2 file", "Styles in version 2 are rejected" and "Old file, new settings";
  - existing version 2 fixtures still load.
- [x] 4.2 Write `styles` in the save layout: one style per line in name order, `default` only when set, an empty table on one line. Verify with `tests_file.rs`:
  - "One style per line";
  - save, load and save of a styled drawing is byte-identical;
  - the existing byte-identical sample test in `sketch_ui` still passes, now as version 3.

## 5. Canvas (D5)

- [x] 5.1 Paint result curves from their resolved style: group the Mesh edges per geometry; trim and dash with `stroke.rs`; use the width `max(1 px, physical)`; draw arrow heads as filled polygons. Construction keeps the fixed look; typeset labels take their resolved colour. Put the pure part (polyline pieces and heads in screen space for one geometry) in a function. Verify with `tests_app.rs`:
  - a dashed segment yields the expected pieces;
  - an arrow end yields a head whose tip is the endpoint's screen position;
  - "Thin line stays visible" (1 px);
  - "Construction ignores its style".
- [x] 5.2 Verify the look in the browser with Playwright against `trunk serve`. Load a file with styles `axis` (0.8 pt, arrow `end`), `hidden` (`dashed`) and a red label, take a screenshot, and check that the arrows, dashes and colours are visible.

## 6. Editor UI (D6)

- [x] 6.1 Add the toolbar style row: current style combo box, four switchable overrides, and the Segment tool creating with them. Verify with `tests_app.rs`: a segment drawn with current style `axis` and colour override `#c00000` exports with those fields.
- [x] 6.2 Add the Brush tool: a click on a curve issues `set_style` with the current style and all four properties (unchecked as `null`). Verify with `tests_app.rs` (an overridden segment brushed with `default` and no overrides loses its overrides) and in the browser with Playwright (brushing a segment changes its look; Undo restores it).
- [x] 6.3 Add the "Styles…" window: list, define, change (commit on release), delete with the error shown. Fall back to `default` when the current style is deleted. Verify in the browser with Playwright:
  - define `vector` with arrow `end`;
  - brush a segment with it and check that the arrow appears;
  - deleting `vector` shows "style in use";
  - changing the width of `vector` changes the brushed segment.

## 7. Documentation and integration

- [x] 7.1 Update `impls/rust/README.md`: styles, the toolbar row, the brush tool, the Styles window, and file version 3. Update the `openspec/config.yaml` context notes (`style.rs`, `stroke.rs`, `StyleRef` in `GeometryShell`, the new commands, file version 3). Verify that the README matches what groups 5 and 6 deliver.
- [x] 7.2 Run `cargo test --workspace`, `cargo clippy --workspace --all-targets`, `scripts/check-wasm.sh` and `openspec validate illustration-styling --strict`, and verify that all pass.

## Workflow follow-up

- Create the selection-model issue (blocked earlier by the permission check) and link it under #14. It is needed for an override panel and for brushing labels.
- Archive the change with spec sync once the PR is reviewed.
