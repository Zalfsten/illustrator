# Design

## Context

See proposal.md for the motivation and specs/ for the contracts. The design applies to impls/rust.

What exists:

- **Curve geometry:** curves are only reachable through formulas (on-curve points, tangents, length). The Canvas strokes the display Mesh. `stroke.rs` already provides `pt_to_world`, `Dash::pattern`, `arrow_length_pt`, `arrow_head` and `arrow_tips`, which give the exact endpoint and outward tangent through capabilities. Its `trim` and `dash` work on polylines.
- **Label placement:** `label_layout` places typeset lines with `split_lines`, `baselines`, `line_origins`, `frame_map` and `place`. The `Typesetter` port delivers each result exactly once through `take_completed`, and its only consumer is the App's `LabelCache`. That cache keeps tessellated triangles, not the outlines.
- **Files:** `FileState` runs one host request at a time (`Request::Open` / `Request::Save`). Its `Saved` event updates the drawing's file and baseline.

## Goals / Non-Goals

**Goals:**

- A format-neutral layer of print primitives, which the PDF writers of #13 can reuse unchanged.
- Exact output: the shape pieces are copied as they are, with no sampling anywhere on the export path.
- Byte-for-byte deterministic SVG, testable against expected text.

**Non-Goals:**

- No reuse of glyphs through `<use>`. Each line's outlines are written in full.
- No changes to `stroke.rs` polyline functions; the Canvas keeps using them.

## Decisions

### D1: Exact shape as a curve capability method in the runtime

`constraint_runtime` gets `ShapePiece::{Line { start, end }, Arc { center, radius, start: HeadingAngle, sweep: TurnAngle }}`. Curve kinds implement it as a formula family of `Geometry`: `shape(chart) -> Vec<ShapePiece>`, with a panicking default like the other families a kind does not expose. This deviates from the first plan of a required method on `CurveExprs`, which `geometry_caps!` cannot enforce because it emits empty factory impls. `ShapePiece` lives in `geometry/shape.rs` with `length`, `point_at` and `sub` (part between two fractions, same piece kind). Document adds `curve_shape(id, &Vector) -> Option<Vec<ShapePiece>>`, which checks the curve capability first and then dispatches through `with_kind!`, so it returns `None` without the capability.

- The arc returns the canonical form: |R|, with the start heading shifted by π for a negative intermediate radius, as `Geometry::canonical` does.
- Tests check the geometry-ir scenarios against `on_curve` and `length` for both kinds.

Alternative: derive the shape from formula samples (fit lines and arcs). Rejected, because it is not exact and it guesses the kind back.

### D2: New crate `sketch_export` with a print scene

`sketch_export` depends on `constraint_runtime` and `label_layout`. It is serde-free and wasm-capable, and is listed in `scripts/check-wasm.sh`.

- **`scene.rs`:** `Scene { items: Vec<Item> }` in world coordinates.
  - `Item::Stroke { name, path: Vec<PathSeg>, width, color, dash: Option<(Vec<f64>, f64)> }`, where the dash is a pattern and phase in world units.
  - `Item::Fill { name, contours: Vec<Vec<PathSeg>>, color }` for arrow heads and glyphs.
  - `Item::Outline { name, contours, width, color }` for stroked label parts.
  - `PathSeg` is `Move | Line | Quad | Cubic | Arc { center, radius, start, sweep } | Close`.
  - `scene(doc, labels) -> Result<Scene, ExportError>` builds the scene from the committed State, walking geometries in id order (creation order).
- **`bounds.rs`:** the exact bounds of a scene (D6).
- **`svg.rs`:** `svg(&Scene, &DocumentSettings) -> String` (D5, D7).
- **`lib.rs`:**
  - `export_svg(doc, labels) -> Result<String, ExportError>`;
  - `exported_labels(doc) -> Vec<(GeometryId, Vec<LineRequest>)>`, the result labels and their line requests;
  - `ExportError::{NothingToExport, MissingLines(GeometryId)}`.

PDF (#13) adds another writer over the same `Scene` and bounds.

Alternative: put the export into `sketch_session`. Rejected, because it would pull `label_layout` into the session crate, and the export needs no commands or history.

### D3: Exact trimming at arrow heads

`scene` takes the resolved style and gets the tips from `stroke::arrow_tips`. For each head it adds a `Fill` triangle from `stroke::arrow_head`, which is already exact.

- The stroke path is the shape trimmed by L at those ends, cut by arc length on the pieces themselves: a line piece is interpolated, and an arc piece is shortened in its start heading or sweep.
- If the heads together are at least as long as the curve, it gets heads and no stroke (styling).

### D4: Native dash arrays with phase

`dash` holds `Dash::pattern(width_pt)` converted to world units, with phase equal to the trimmed start length. The SVG writer emits `stroke-dasharray` and `stroke-dashoffset` in mm. A renderer then lays the pattern at path distance d as pattern(d + offset), which is the pattern from the curve start (styling). Solid strokes get no dash attributes.

Alternative: cut dash pieces ourselves. Rejected, because the export would grow many small subpaths and stop being editable as one stroke.

### D5: Page mapping and number format

- **Page coordinates:** a world point maps to x' = (x − xmin)·s and y' = (ymax − y)·s, with s the scale in mm per unit. The y axis is flipped in the coordinates, not through a `transform`, so stroke widths stay plain mm.
- **Root element:** `width="{W}mm" height="{H}mm" viewBox="0 0 W H"`.
- **Arcs:** an arc piece is written as `A r r 0 0 f x y`. Pieces with |sweep| > π are split into equal parts of at most π first, so the large-arc flag is always 0 and full circles work. The sweep flag is 0 for a counterclockwise world sweep, because of the y flip.
- **Numbers:** fixed 3 decimals in mm (1 µm), trailing zeros and a trailing `.` removed, and `-0` written as `0`. This is what makes D7 deterministic.

### D6: Exact bounds

The bounds are computed in world units over all items:

- **Straight stroke:** the four corners of its butt rectangle, ±w/2 along the normal.
- **Arc stroke:** the end corners of the annular sector, plus the points of the inner and outer circle at axis headings 0, π/2, π and 3π/2 that lie within the sweep.
- **Fill:** the end points of every piece plus the curve extrema found by derivative roots, for quadratic and cubic pieces.
- **Outline:** the same, widened by w/2 along each straight piece's normal. For curved stroked pieces, the control-polygon box widened by w/2 is used.

Dashed strokes count as undashed (spec). `NothingToExport` is raised when there are no items or the bounds are empty.

### D7: SVG layout

The writer emits a fixed layout:

- an `<?xml …?>` header and one `<svg xmlns="http://www.w3.org/2000/svg" version="1.1" …>` root;
- per object, a `<g id="<object name>">` holding its path elements, each with explicit `fill`, `stroke`, `stroke-width` and `stroke-linecap="butt"`;
- one element per line, `\n` line endings.

Object names follow the name syntax and are therefore valid XML ids, so editors such as Inkscape show them. There is no `<text>`, `<style>`, `<image>` or `xlink`.

### D8: Label lines for the export through the LabelCache

The port has a single consumer, so the export does not call `take_completed` itself. Instead:

- **Line requests:** building them for a label (split, size, preamble) moves from `labels.rs` into `label_layout::label_requests(attrs, settings)`. Canvas and export then build identical keys.
- **Cache entries:** `LabelCache` keeps `Rc<TypesetLine>` next to the triangles in `Entry::Done`. A new `ensure(ts, reqs) -> Option<Result<Vec<Rc<TypesetLine>>, TypesetError>>` requests missing lines and marks them used for this frame, so `sweep` keeps them.
- **Export job:** an `ExportJob` in the App holds a clone of the Document taken at request time (D9) and the result of `exported_labels` for that clone.
  - **While lines are pending:** each frame, while the job runs, it calls `ensure` for every label. The status line shows "Export waiting for labels…", and the App requests repaints as it does for pending labels.
  - **When all lines are delivered:** it calls `export_svg` and hands the bytes to `FileState::export`.
  - **When a label fails:** it collects every failed label with its error and reports `cannot export: lbl3: unsupported \foo; …`.
  - **When the typesetter is missing or failed to load:** it reports the same way with `no typesetter` or the load error.

Alternative: a second port consumer. Rejected, because results would be lost to whichever consumer takes them first.

### D9: Export through FileState without touching the drawing's file

- **Request and event:** `FileState` gets `Request::Export` and `FileEvent::Exported(SaveResult)`.
- **`export(bytes, suggested_name, files)`:** sends `SaveTarget::Ask { suggested_name }`. On download-only hosts that is the download already.
- **Exported event:** it reports `Exported <name>` or the error. It never changes `handle`, `baseline` or `unsaved`.
- **Suggested name:** taken from the drawing's file name with `.sketch.json`, or else `.json`, stripped, plus `.svg`. Without a file it is `untitled.svg`.
- **Starting:** the App starts an `ExportJob` only when no drag is active, `FileState::busy()` is false and no job is running. The Document clone makes the result independent of edits while waiting (spec: edit while waiting).
- **Bytes while busy:** `FileState::export` returns `false` and starts nothing while another request is open (for example a Save started while the export waited for labels). The App then keeps the finished bytes (`Exporting::Bytes`) and hands them over in a later frame.
- **Test doubles:** the job's tests and the App's tests share `fake_typesetter.rs` (`Boxes` engine, `Manual` loading typesetter), next to `fake_files.rs`.

### D10: UI entry

The File menu gets "Export SVG…" after Save As, with no shortcut. It is disabled while dragging, while a file request is pending or while an export job runs. Messages use the existing status line.

## Risks / Trade-offs

- **[Glyph outlines are large]** Each line is written in full. → Acceptable for figures. `<use>` reuse can come later without changing the spec.
- **[Stroked label parts]** Typst stroke caps and joins are assumed butt and miter for rules such as fraction bars, and curved stroked parts use a conservative box. → The bounds may exceed the ink by up to w/2 there. This is rare (curved stroked glyph parts) and noted in tests.
- **[Viewer differences in dash rendering on arcs]** → Uses only SVG 1.1 core attributes. Checked visually in a browser and in Inkscape.
- **[3-decimal rounding]** Bounds touch the ink only within 1 µm. → The spec allows the file's numeric precision.
- **[Document clone per export]** → Cheap compared with typesetting, and done once per export request.
