# Proposal

## Why

Scientific figures need labels: axis names, angle markers, formulas next to a tangent. Labels must survive edits just as geometric relationships do. #12 settles how labels work. This first slice (#58) adds the label as an object of the drawing: the model, the commands and the file format. Typesetting, display and editing come later. Doing the model first fixes the shared contract that the typesetting engine (#59), the browser module (#60), SVG export (#9) and print export (#13) build on. It can be tested without any typesetting engine.

## What Changes

- **Label geometry kind** (name prefix `lbl`, kind tag `label`).
  - A label's anchor is a **baseline**: an origin location plus a heading. These are 3 continuous parameters in State. The text's extent comes from typesetting and is not a parameter.
  - A label is not a curve: it has no on-curve positions, tangents, length or display edges.
  - It exposes two expressions:
    - an **origin** point, new address `lbl1.origin`;
    - a unit **direction** free vector, existing address `lbl1.direction`.
  - Attaching and orienting a label therefore use the existing constraints, for example `join(lbl1.origin, seg1.end)` and `parallel(lbl1.direction, seg1.direction)`.
- **Label attributes**, which are not parameters:
  - `source`: the text;
  - `markup`: `latex` (LaTeX text mode with `$...$` math) or `text` (literal characters);
  - `halign`: `left`, `center` or `right`;
  - `valign`: `first` or `last`, the line whose baseline lies on the anchor;
  - `offset`: `[dx, dy]` in pt, in the label's own frame;
  - `size`: in pt, or inherited from the drawing.
  - Labels may have several lines, with explicit breaks only.
- **Document settings**, a new part of the drawing:
  - a physical **scale** (millimetres per world unit, default 10, i.e. 1 unit = 1 cm);
  - a **font size** (pt, default 10);
  - a LaTeX **preamble** (default empty).
  - Undo and redo cover them like any other step.
- **New commands:**
  - **`set`** changes attributes of a named object. It carries the object's kind tag and is checked against that kind's attribute schema: partial, atomic, one undo step, and it never changes parameter values.
  - **`set_document`** changes document settings.
  - `create` with kind `label` carries all attributes. The declarative subset is still `create` + `constrain`.
  - **BREAKING (interchange):** the closed operation set grows from 5 to 7 operations, and the JSON Schemas change accordingly.
- **Sketch file version 2.** The envelope gains `scale`, `font_size` and `preamble`. Saving writes version 2; version 1 files still load, with default settings, through the loader's migration step.
- **Geometries that are not curves.** The capability `curve` becomes optional for a kind: label has `anchor` + `line` but not `curve`. Drag length floors, the display Mesh and load-report displacement handle such geometries without assuming a curve.

## Non-goals

- Typesetting, measuring (bounding box, baseline), drawing and editing labels in the app (#59), the browser module (#60). Until #59, the GUI shows no labels and has no label tool.
- Flagging unsupported LaTeX constructs: that depends on the engine (#59).
- SVG export of labels (#9), PDF and PDF+LaTeX overlay export, including how the preamble is exported (#13).
- An "always upright" option for labels whose direction points left, labels with automatic line wrapping, and vertical anchors other than first/last baseline.
- Styling attributes of other kinds (#10). Their `set` attribute schemas will extend the one introduced here.
- Folding `set_role` into `set`.

The label model, the commands and the file format are shared specs: every implementation reads and writes the same labels and files. Storage, dispatch and code layout are design decisions of each implementation.

## Capabilities

### New Capabilities

- `labels`: the label as a drawing object, covering its anchor frame (origin + heading), attributes and their meaning (markup modes, alignment, offset in the label frame, size, line breaks) and defaults.
- `document-settings`: drawing-wide settings: physical scale, font size and preamble, with defaults, value ranges and undo.

### Modified Capabilities

- `geometry-ir`:
  - admits geometry kinds that are not curves (the label);
  - adds an anchor capability (origin point);
  - extends the line capability to non-curve kinds (label direction);
  - capability queries report `anchor`.
- `object-naming`: new address suffix `origin`.
- `sketch-commands`:
  - create a label;
  - new `set` and `set_document` operations in the closed operation set;
  - kind tag `label`;
  - attribute field shapes and validation;
  - document settings next to the declarative command list.
- `sketch-file`:
  - version 2 envelope with settings;
  - version 1 still loads;
  - displacement for geometries without a curve.
- `display-mesh`: geometries without the curve capability contribute no edges.

## Impact

- `impls/rust/crates/constraint_runtime`:
  - new `Label` kind (chart, formulas, capabilities);
  - `Caps` gains `anchor`, and `curve` becomes opt-in in `geometry_caps!`;
  - label attributes and document settings stored in `Document`, so undo snapshots cover them;
  - curve-only paths (Mesh, drag length floors) skip geometries without a curve;
  - `origin` address.
- `impls/rust/crates/sketch_session`:
  - label command kind;
  - `set` / `set_document` commands and their JSON forms;
  - attribute export in the declarative list;
  - file version 2 with migration from version 1;
  - regenerated JSON Schemas;
  - displacement for labels.
- `impls/rust/crates/sketch_ui`: must not fail on drawings with labels, for example loaded files. No label UI yet.
- No new dependencies.
