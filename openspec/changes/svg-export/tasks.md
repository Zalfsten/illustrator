# Tasks

## 1. Exact curve shape (constraint_runtime)

- [ ] 1.1 Add `ShapePiece` (Line, Arc), a required `shape` method on the curve capability for Segment and Arc (canonical |R|), and `Document::curve_shape(id, &Vector) -> Option<Vec<ShapePiece>>` (D1); verify with `src/tests_shape.rs`, covering the geometry-ir scenarios "Segment shape", "Arc shape", "Geometry without the curve capability has no shape", "Samples lie on the shape" and "Shape length equals curve length" (including a negative intermediate radius), and with `cargo test -p constraint_runtime`
- [ ] 1.2 Update the impls/rust notes in `openspec/config.yaml` (curve capability `shape`, `ShapePiece`) and verify that `openspec validate svg-export --strict` still passes

## 2. Export crate (sketch_export)

- [ ] 2.1 Create the crate `sketch_export` (deps `constraint_runtime`, `label_layout`; no serde), add it to the workspace and to `scripts/check-wasm.sh`; verify with `cargo build -p sketch_export` and `scripts/check-wasm.sh`
- [ ] 2.2 Move label line-request building into `label_layout::label_requests(attrs, settings)` and use it from `sketch_ui/src/labels.rs`; verify that the existing label tests still pass and that a new `label_layout` test gives the same requests as before for latex and text labels
- [ ] 2.3 Implement `scene.rs`: walk geometries in id order, result role only (D2):
  - curves from `curve_shape` with the resolved style;
  - exact trimming at arrow heads, with Fill triangles from `arrow_head` (D3);
  - dash pattern and phase in world units (D4);
  - heads only when the curve is too short.

  Verify with `src/tests_scene.rs`: construction left out, segment as one straight piece, arc pieces not polylines, arrow tip at the end with the stroke ending at its base, dash phase from the curve start, and width and colour.
- [ ] 2.4 Add labels to the scene: `exported_labels(doc)` (result labels only, with their requests); placed outlines as Fill and Outline items in the resolved colour; `ExportError::MissingLines`; an empty source adds nothing. Verify with `tests_scene.rs` using hand-built `TypesetLine`s: placement matches `label_layout::place`, colour, and the construction label left out
- [ ] 2.5 Implement `bounds.rs` (D6): butt rectangles, annular sectors with axis extrema, triangles, quadratic and cubic extrema, outline widening; `NothingToExport` for empty scenes. Verify with `src/tests_bounds.rs`: the 30 mm × 0.4 pt segment, the quarter-arc bounds, the arrow-head height 0.75 L, a cubic extremum inside the control box, and nothing to export
- [ ] 2.6 Implement `svg.rs` and `export_svg` (D5, D7):
  - mm page mapping with y flip;
  - 3-decimal numbers;
  - arcs split at π with correct sweep flags;
  - `<g id=name>` per object, and dash attributes.

  Verify with `src/tests_svg.rs`:
  - expected text for a small drawing;
  - the arrow pointing up for the up segment;
  - a full-circle arc;
  - identical bytes on a second export;
  - a `roxmltree` (dev-dependency) parse that confirms only `svg`, `g` and `path` elements, with no `text` or external reference.

## 3. App integration (sketch_ui)

- [ ] 3.1 Keep `Rc<TypesetLine>` in `LabelCache` entries and add `ensure(ts, reqs)`, which requests and pins lines for the frame (D8). Verify with `tests_labels.rs`: a pinned line survives `sweep`, results arrive for both Canvas and export, and painting is unchanged
- [ ] 3.2 Add `Request::Export`, `FileEvent::Exported` and `FileState::export` with the suggested name rule (D9). Verify with `tests_files.rs` against `FakeFiles`:
  - `triangle.sketch.json` → `triangle.svg`, and `untitled.svg` without a file;
  - cancel writes nothing;
  - a write error is shown;
  - success keeps the handle, baseline, unsaved state and title;
  - no export while busy.
- [ ] 3.3 Add `ExportJob` and "Export SVG…" in the File menu (D8, D10): the Document is cloned at request time; the job waits with the "Export waiting for labels…" status; failing labels are named; no typesetter or a failed load is reported; the item is disabled during drags, pending file requests and running jobs. Verify with `tests_app.rs`:
  - export of the sample scene produces bytes through `FakeFiles`;
  - the export waits while a fake typesetter is pending, then completes;
  - a failing label is named and no save happens;
  - an edit while waiting does not change the exported bytes;
  - no export during a drag.
- [ ] 3.4 Document the export in the README: the menu entry, what is exported, units and bounds, and that labels are outlines. Verify that the README describes the behaviour the tests above check

## 4. Integration checks

- [ ] 4.1 Run `cargo test --workspace`, `cargo clippy --workspace --all-targets` (no warnings), `scripts/check-wasm.sh` and `openspec validate svg-export --strict`, and verify that all pass
- [ ] 4.2 Export a drawing with dashed, arrowed and coloured segments, an arc and a typeset label, natively and in the browser build (download). Verify by rendering the SVG in Chromium (Playwright screenshot) that it matches the Canvas: arrows, dashes, label position and colour, and tight bounds

## Workflow follow-up

- Commit on branch `issue-9-svg-export`, push and open the PR for #9 after confirmation.
- Archive the change with spec sync after review.
