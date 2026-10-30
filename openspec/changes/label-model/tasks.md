# Tasks

## 1. Core: geometries without a curve

- [ ] 1.1 Make `curve` an explicit capability.
  - `geometry_caps!` no longer implies `CurveExprs`, and its base is `Caps::NONE`.
  - Segment and arc declare `curve`.
  - `LineExprs` / `EndpointExprs` / `CircularExprs` take `Geometry` as their supertrait.
  - Add `Caps.anchor`, `AnchorExprs::origin_expr`, `PointFormula::Origin` and `Document::origin_expr`.
  - `point` / `vec` / `length` in `Geometry` get panicking defaults.
  - Verify: `cargo test -p constraint_runtime` passes unchanged, and `tests_catalog.rs` asserts that segment and arc report the same caps as before, with no `anchor`.
- [ ] 1.2 Switch the curve paths to id-based factories.
  - Mesh tessellation samples through `doc.on_curve_expr` and skips geometries without `curve`.
  - `interact::curve_length_exprs` uses `doc.length_expr` and `filter_map`.
  - Verify: the existing `tests_mesh.rs` / `tests_interact.rs` pass. A test with a stub kind or reduced caps shows that a geometry without `curve` yields no mesh edges and no length floor.

## 2. Core: label kind, attributes, document settings

- [ ] 2.1 Add `geometry_kinds/label.rs` and register `Label` in `GeometryKind` / `with_kind!`.
  - `chart_atoms! { Label { origin: Point2, heading: HeadingAngle } }`, `geometry_caps!(Label: anchor, line)`, `NAME_PREFIX = "lbl"`.
  - Origin and direction `(cos θ, sin θ)` with sparse Jacobians: origin uses only the origin slot, direction only the heading slot.
  - Verify with `tests_label.rs`:
    - values;
    - finite-difference Jacobian checks;
    - the caps are exactly `anchor` and `line`;
    - curve expressions are unavailable by id.
- [ ] 2.2 Add `KindAttrs { None, Label(LabelAttrs) }` and the label enums (`Markup`, `HAlign`, `VAlign`) with their defaults.
  - `GeometryInst` gets `attrs` and becomes `Clone`, not `Copy`.
  - Add `Geometry::default_attrs`, `Document::add_geometry_with`, `label_attrs` and `set_label_attrs`.
  - Verify with `tests_document.rs`:
    - `add_geometry::<Label>` gets the default attrs;
    - `set_label_attrs` round-trips;
    - removing a label drops its attrs;
    - a cloned Document keeps independent attrs.
- [ ] 2.3 Add `DocumentSettings { scale, font_size, preamble }` (default 10, 10, "") to `Document` with `settings()` / `set_settings()`. Verify in `tests_document.rs`: the defaults, and that a clone snapshot restores the previous settings.
- [ ] 2.4 Add label solve scenarios to `tests/acceptance.rs`.
  - `join(lbl.origin, seg.end)` plus `parallel(lbl.direction, seg.direction)` follows a drag of `seg.end`.
  - A fix on `lbl.origin` holds after solving.
  - A label in a drag scope gets no length floor.
  - Verify: `cargo test -p constraint_runtime --test acceptance`.
- [ ] 2.5 Add the `origin` suffix to `resolve_address` and `point_address` (capability name `anchor`).
  - Verify in `tests_address.rs`:
    - `lbl1.origin` / `lbl1.direction` resolve to the right families;
    - `lbl1.start` fails with a missing `endpoints` capability;
    - `seg1.origin` fails with a missing `anchor` capability;
    - `origin` prints back.
- [ ] 2.6 Update the docs.
  - `docs/constraint-mathematics.md`: capability traits, `AnchorExprs`, curve as an opt-in capability, attributes on `GeometryInst`.
  - The impls/rust module notes in `openspec/config.yaml` context: geometry_caps, the label kind, `KindAttrs`, `DocumentSettings`.
  - Verify: the docs name the new types exactly as in the code.
- [ ] 2.7 Run `impls/rust/scripts/check-wasm.sh` and verify that the core still builds for `wasm32-unknown-unknown`.

## 3. Session: label create, `set`, `set_document`

- [ ] 3.1 Add wire mirrors for `Markup` / `HAlign` / `VAlign` (lowercase) and a `wire::double_option` module with a hand-written schema. Verify in `tests_wire.rs`: absent, `null` and a number each read to the right value, and the enums read and write only their lowercase forms.
- [ ] 3.2 Extend `GeometryCommand`: `attrs(p)` (default `KindAttrs::None`) and `params(init, &KindAttrs)`.
  - Add `kinds/label.rs`:
    - `LabelParams`: `origin`, `heading` and `source` required, the other attributes defaulted, `size` skipped when inherited;
    - validation: finite numbers, `size > 0`.
  - Register `Label` under `create:` in `command_catalog!` and switch `add_to` to `add_geometry_with`.
  - Verify in `tests_kinds.rs` / `tests_json.rs`:
    - the spec's label interchange scenario;
    - a missing `source` is rejected;
    - `size: 0` gives `invalid value`;
    - the write/read round trip is bit-identical, including a source with line breaks.
- [ ] 3.3 Add the `AttributeCommand` trait with `LabelSet` (all fields optional, `size` as double option, non-empty check).
  - Add the `set:` list to `command_catalog!` (generating `AttrSpec`) and `Command::Set { name, attrs }`.
  - Execution: unknown name, wrong object type for constraints and kind mismatch; no solve; no frozen check.
  - Verify in `tests_session.rs` with the spec scenarios:
    - change the source;
    - several attributes undone in one step;
    - set on a constraint;
    - kind mismatch;
    - `size: null` restores inheritance;
    - set inside a frozen component succeeds and leaves the values unchanged.
- [ ] 3.4 Add `Command::SetDocument(SettingsPatch)` and `Session::settings()`.
  - The `set_document` fields are optional and must be non-empty; `scale` / `font_size` must be > 0 and finite.
  - Verify in `tests_session.rs`:
    - change the scale and leave the rest untouched;
    - an empty `set_document` is rejected;
    - `scale` -1 and 0 give `invalid value`;
    - undo / redo restore the settings.
- [ ] 3.5 JSON interchange.
  - `OPS` gains `set` and `set_document`; `command()` reads both.
  - `decl_command()` rejects them as not declarative; `Serialize` writes both.
  - Verify in `tests_json.rs`:
    - the spec's set interchange scenario;
    - reading `set` without attributes fails;
    - `set` is rejected in the declarative subset;
    - round trips for both ops.
- [ ] 3.6 Schemas.
  - Add the set branches (op `set`, kind const, `minProperties`) and the `set_document` fixed branch.
  - The declarative schema stays create + constrain.
  - Regenerate `sketch_session/schema/*.json` with `UPDATE_SCHEMA=1`.
  - Verify: `cargo test -p sketch_session --test schema` passes, and the schema lists `set` with kind `label` and `set_document`.
- [ ] 3.7 Declarative export of labels. Verify in `tests_session.rs`:
  - the spec's "label attributes after set" scenario;
  - export of a label with an own size and with an inherited size (`size` absent);
  - export → re-execute → export yields the same list.

## 4. Sketch file version 2

- [ ] 4.1 `VERSION = 2`.
  - `load` accepts 1 and 2.
  - `migrate(1)` rejects settings fields present in v1, then inserts the defaults.
  - v2 requires `scale`, `font_size` and `preamble`, checks their ranges (`LoadError::Envelope`) and applies them before executing the commands.
  - Verify in `tests_file.rs` with the spec scenarios:
    - the minimal v2 file;
    - v1 loads with defaults;
    - v1 with `scale` is rejected;
    - a missing `preamble` is rejected;
    - `scale` 0 is rejected;
    - version 3 is unsupported.
- [ ] 4.2 `save` writes `format`, `version`, `scale`, `font_size`, `preamble` and `commands`, each field and each command on its own line. Update the existing byte-layout tests to version 2. Verify:
  - save / load / save is byte-identical for a drawing with a multi-line preamble and a multi-line label source;
  - the one-line-per-command test still holds.
- [ ] 4.3 Displacement: geometries without an on-curve expression compare `origin_expr` before and after. Verify with the spec scenario in `tests_file.rs` or `tests_session.rs`: a label whose origin is moved by 0.2 reports a displacement of 0.2.

## 5. Integration

- [ ] 5.1 Check that `sketch_ui` keeps working with labels.
  - Compile against the changed core: `GeometryInst` is not `Copy`, and the caps are explicit.
  - Add a test in `tests_app.rs` that loads a drawing with a free label and a joined label, builds the mesh and logical points, and drags the joined endpoint without panicking.
  - Verify: `cargo test --workspace`.
- [ ] 5.2 Verify the whole workspace: `cargo test --workspace`, `cargo clippy --workspace --all-targets`, `impls/rust/scripts/check-wasm.sh` and `openspec validate label-model --strict` all pass.

## Workflow follow-up

- Archive the change after review. Close #58 and update #12.
