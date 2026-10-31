# Design

## Context

- **`constraint_runtime`:**
  - `Document` holds the geometries: a `GeometryShell` (id, role, parameter offset), a kind, and `KindAttrs`, which is `None` or `Label(LabelAttrs)`.
  - `DocumentSettings` sit beside them, and `Document` is cloned for every undo snapshot.
  - `Mesh` gives `MeshEdge { geometry, role, a, b }`, with one geometry's edges consecutive and in curve order.
  - The crate is serde-free.
- **`sketch_session`:**
  - `Command::Create` carries `name`, `role` and `GeometrySpec`. The interchange form takes the reserved fields (`op`, `kind`, `name`, `role`) out of the JSON map and hands the rest to the kind's parameter type with `deny_unknown_fields`.
  - `file.rs` writes the envelope by hand, one field per line, and migrates version 1 to 2 (`migrate`).
  - Schemas are generated (`schema.rs`, `UPDATE_SCHEMA=1`).
- **`sketch_ui`:**
  - The Canvas strokes `MeshEdge`s with `role_color` and a fixed 2 px or 1.5 px width.
  - Typeset labels are coloured by role; fallback labels are grey or orange.
  - There is no selection, and tools keep their own picks.

## Goals / Non-Goals

**Goals:**
- One place that resolves a geometry's style and computes the stroke geometry (pt to world, arrow head, trimming, dashing), usable by the Canvas now and by the SVG and PDF exports (#9, #13) later.
- Style data takes part in undo, the byte comparison for unsaved changes (#8) and the file round trip without extra code paths.

**Non-Goals:**
- Exact-curve dash or trim output for exports: this change computes them on polylines for the Canvas. The functions take arc length, so an export can apply the same lengths to exact curves.

## Decisions

### D1: The style model lives in `constraint_runtime` (`style.rs`)

```rust
pub struct Rgb(pub u8, pub u8, pub u8);
pub enum Dash { Solid, Dashed, Dotted, DashDot }
pub enum Arrow { None, Start, End, Both }
/// A value for any subset of the properties (a named style, or overrides).
#[derive(Clone, Default, PartialEq)]
pub struct StyleProps { pub width: Option<f64>, pub color: Option<Rgb>, pub dash: Option<Dash>, pub arrow: Option<Arrow> }
/// Every property resolved.
pub struct Style { pub width: f64, pub color: Rgb, pub dash: Dash, pub arrow: Arrow }
/// Named styles; `default` always present.
pub struct Styles(BTreeMap<String, StyleProps>);
/// A geometry's style reference and overrides.
pub struct StyleRef { pub style: String, pub overrides: StyleProps }
```

- **Where the data sits:**
  - `GeometryShell` gains `style: StyleRef` (geometry-ir: shared metadata), so every kind has it without touching `KindAttrs`.
  - `Document` gains `styles: Styles` beside `settings`.
  - Both are cloned with the Document, so undo and redo, and the unsaved byte comparison, work unchanged.
- **API:**
  - `Document::styles()` / `define_style(name, &StylePatch)` creates the style or applies the patch. `StylePatch` (`Option<Option<T>>` per property: leave, set, remove) lives in `style.rs` too, so the session and the runtime share it.
  - `delete_style(name)`, `set_geometry_style(id, StyleRef)`, `resolved_style(id) -> Style`, and `set_styles(Styles)` for loading a file.
  - Rejections are a `StyleError`: `InvalidName`, `Unknown(name)`, `InUse(users)` (also for `default`), `InvalidWidth`, `UnknownGeometry`. The session maps them to its error classes.
  - `delete_style` scans the geometries; drawings are small, so there is no reverse index.
- **References by name string:** a geometry refers to its style by name. Styles cannot be renamed in this change, so a string reference stays valid. A rename later would need either an update of all references or ids.
- **Validation in the runtime:** ranges (width finite and > 0) are checked in the runtime like `DocumentSettings`, so every caller gets them. The colour is parsed into `Rgb` at the interchange layer; the runtime never sees a malformed colour.
- *Alternative:* styles as a separate side table in `sketch_session`. It would keep `constraint_runtime` unaware of appearance, but a second clone path for undo and a second id map would be needed, and Mesh paint would need the session. Rejected: geometry-ir already makes role shared metadata, and the style sits beside it.

### D2: Stroke geometry as pure functions (`constraint_runtime/src/stroke.rs`)

- **`pt_to_world(pt, &DocumentSettings) -> f64`:** `pt × 25.4/72.27 / scale`.
- **`arrow_head(tip: Point2, out: Vec2, width_pt, settings) -> [Point2; 3]`:**
  - `out` is the unit tangent pointing away from the curve (end: the curve tangent; start: its negation);
  - L = 3 + 4.5 w, and the base is 0.75 L wide.
- **`arrow_tips(doc, id) -> (Option<(Point2, Vec2)>, Option<(Point2, Vec2)>)`:**
  - start and end points with their outward tangents, from the endpoint and tangent expressions (`EndpointExprs`, `CurveExprs`, by capability, not by kind);
  - exact, not from the Mesh.
- **`trim(polyline, start_len, end_len) -> Vec<Point2>`** and **`dash(polyline, pattern) -> Vec<Vec<Point2>>`:** they work by arc length on a world-space polyline. The pattern lengths come from `Dash::pattern(width_pt)` in pt, converted with `pt_to_world`. The phase starts at the polyline's start.
- **Unit tests in `tests_stroke.rs`:** the spec scenarios (the head on a segment, the head on an arc, the shortened stroke, a very short arrow, a dashed 12 pt segment, and a pattern continuing across polyline vertices).
- *Alternative:* computing all of this in `sketch_ui`. Rejected: #9 and #13 need the same numbers, and they must not depend on egui.

### D3: Commands and interchange in `sketch_session`

- **Command variants:**
  - `Command::DefineStyle { style: String, patch: StylePatch }`, `DeleteStyle { style }` and `SetStyle { name, style: Option<String>, patch: StylePatch }`.
  - `StylePatch` (from D1) holds `Option<Option<T>>` per property: absent, set, or remove (`null`), as label `size` does today.
  - `Command::Create` and `DeclCommand::Create` gain `look: StyleRef`.
- **Interchange:**
  - `style`, `width`, `color`, `dash`, `arrow` join `RESERVED`.
  - `create` takes them out of the map before the kind's fields, just like `role`, and writes them after the kind fields: `style` only if it is not `default`, then the overrides in a fixed order.
  - `wire.rs` gets a `color` module (`#rrggbb`, reading either case, writing lower case) and the `dash` and `arrow` string enums.
- **Errors:**
  - `CommandError::UnknownStyle(String)` and `StyleInUse { style, users: Vec<String> }`;
  - `InvalidValue` for the width and colour;
  - `Name` for an invalid style name (`is_valid_name`).
- **History:** the three new commands are history steps like `set` (snapshot, no solve). `create` with an unknown style fails before any change.
- **Queries:** `Session::styles()`, plus `Session::style_of(name) -> (StyleRef, Style)` for the queryable requirement.
- **Schema:** the generated schemas gain the three operations and the style fields on `create`, and the checked-in schema files are regenerated.

### D4: File version 3

- **`VERSION = 3`.** `migrate` brings versions 1 and 2 forward by inserting `"styles": {}`.
- **Style fields in older files:** after the commands of a version 1 or 2 file are read, any `create` with a style field is rejected, as the spec requires.
- **Reading `styles`:** each entry is checked as an object of style fields (`deny_unknown_fields`), then the names, then the values; an error names the style. `default` is optional.
- **Save layout:** `"styles": {` then one line per style, `"<name>": {…}`, in name order (BTreeMap), then `},`. `default` is written only when it is not empty, and an empty table is written as `"styles": {},` on one line. The settings come first, then `styles`, then `commands`. Byte determinism follows from the fixed field order inside each style object.

### D5: Canvas paint

- **Grouping:** the edges of one geometry are consecutive. `app.rs` groups them into a polyline per geometry. A result geometry's polyline is styled from `resolved_style`, then:
  1. **trim** by the arrow lengths that apply;
  2. **dash** by the pattern;
  3. **stroke** each piece as an `egui::Shape::line` with width `max(1 px, pt_to_world(w) × viewport.scale)`.
     - egui strokes with butt ends and mitred joins, so a polyline's interior joins do not show at these widths.
  4. **arrows:** each head is a filled convex polygon at the exact tip from `arrow_tips`.
- **Construction geometry** keeps today's look: grey, 1.5 px, solid, no arrows.
- **Labels:** a typeset label's colour becomes its resolved colour, while construction labels stay grey. Fallback labels keep their status colours (spec: status displays keep their colours).
- **Why the paint lives in `sketch_ui`:** egui shapes are UI. The numbers come from D2.

### D6: Editor UI without a selection

- **The toolbar's style row:**
  - a combo box for the current style (style table names);
  - four optional overrides (a colour button, a width drag value in pt, combo boxes for dash and arrow), each behind a checkbox that turns it on;
  - all held in the App as `current: StyleRef`.
- **New segments:** the Segment tool creates segments with `current`: `style` plus the overrides that are switched on.
- **Brush tool (`Tool::Brush`):**
  - A click on a curve (`hit_edge`) issues `set_style` with the current style and all four properties: checked ones as values, unchecked ones as `null`.
  - After brushing, the object therefore looks exactly like a new object drawn with the current settings.
  - Labels cannot be brushed in this change.
- **"Styles…" window** (a button in the toolbar), with one row per style:
  - name, the four properties with a checkbox meaning "set in this style", and a delete button;
  - an entry field plus "Add" to define a new style.
  - Each edit issues one `define_style` command, so it can be undone. Errors (style in use, invalid name) go to the error line.
- **UI-only state:** the current style and its overrides are not part of the drawing and are not saved. If the current style is deleted, the selection falls back to `default`.
- **Where it lives:** `sketch_ui/src/style_ui.rs` (`CurrentStyle`, `StylesWindow`); the window is anchored below the toolbar.
- **Adding a style:** a style must hold at least one property, so "Add" defines the new style with the built-in width 0.4 pt.

## Risks / Trade-offs

- [Every width or colour drag in the Styles window would make one history step per frame] → commit on release: `DragValue` changes apply when the drag ends, and colour edits when the picker closes. During the drag the window shows the pending value only.
- [The polyline is trimmed along the polyline while the arrow head is computed from the exact curve, so on a tightly curved, coarsely tessellated arc the stroke end can sit slightly off the head's base] → tessellation already bounds the turn per chord (π/8), and L is a few pt. Exports trim the exact curve by arc length (constant speed for segments and arcs).
- [Style names are strings in every geometry; memory and clone cost grow with the drawing] → drawings are small, and the clone already copies label sources. Interning can come later without a spec change.
- [Version 3 files cannot be opened by builds of #8] → expected for a format bump; `unsupported version` names 3.
- [The 1 px minimum makes thin lines look relatively thicker when zoomed out] → display only; exports use real widths.

## Migration Plan

- Files of versions 1 and 2 load unchanged, with only `default`; the next save writes version 3.
- The sample drawing at start keeps its geometry and gets no styles. It renders with the built-in look (0.4 pt black), not today's fixed 2 px.

## Open Questions

- The exact UI layout of the style row and the Styles window: settled while building, within D6.
