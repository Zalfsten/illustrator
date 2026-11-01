# Proposal

## Why

Figures for scientific publications and teaching have to leave the app. Today a drawing can only be saved as a sketch file. SVG is the simplest print-quality target: one file, exact lines and circular arcs, and no fonts to embed. It also sets the export contract (exact geometry, only what is visible, physical size, tight bounds) that print export (#13, PDF and PDF+LaTeX overlay) builds on later. Styles (#10), typeset labels (#59) and the host save service (#8) are in place, so this can be done now. GitHub issue #9.

## What Changes

- **Export command:** the drawing can be exported as an SVG file, saved through the host under a suggested name `<drawing>.svg`, or `untitled.svg` without a file.
  - The export is not a history step. It changes neither the drawing's file nor its unsaved-changes state.
- **Exact geometry:** curves are written from their exact shape, not from the display mesh.
  - A segment becomes a straight path piece and a circular arc becomes an elliptical-arc path command with its exact radius.
  - Curve kinds expose this shape through the curve capability, so the export does not branch on kinds.
- **Only what is visible:** result geometry and result labels are written. Construction geometry, constraint glyphs and selection highlights are left out.
- **Styles at physical size:** the resolved style is applied at its physical size: colour, width in TeX points, dash pattern laid from the curve start, flat ends, and filled arrow heads with the stroke ending at their base (styling).
- **Labels as outlines:** labels are written as filled and stroked outlines in their style colour, so the file looks the same everywhere. The text is not searchable.
  - The export waits while labels are still being typeset.
  - It fails, naming the label, when a label cannot be typeset.
- **Size and bounds:** the file has its physical size in millimetres through the document scale. The bounds are tight to the drawn shapes, with no margin.
  - A drawing with nothing visible is not exported.
- **Same bytes:** the same drawing always gives the same file.

## Non-goals

- PDF and PDF+LaTeX overlay export (#13); they reuse this export contract later.
- Bézier approximation of future curve kinds (function graphs, splines). There are none yet, and the error bound is specified when the first one arrives.
- Fill, a configurable margin, a figure-width or rescaling option, and selectable or searchable text.
- Exporting only a selection (there is no selection model yet).

Within the product goal this is the first output path for the illustration side: the constraint runtime stays untouched apart from describing curve shapes exactly. The specs below are shared contracts. The SVG writer, its crate and the UI wiring are design decisions of the Rust implementation (impls/rust).

## Capabilities

### New Capabilities

- `drawing-export`: exporting a drawing as a standalone vector file.
  - Shared guarantees: which objects are included, exact curves, styles at physical size, labels as outlines, the size and tight bounds, deterministic output, and behaviour when labels are pending or fail.
  - The export command: suggested name, saving through the host, no change to the drawing or its file.
  - The SVG form. PDF formats join this capability later.

### Modified Capabilities

- `geometry-ir`: the curve capability additionally describes a curve's exact shape as pieces from a fixed vocabulary (straight pieces, circular-arc pieces), consistent with its on-curve samples.

## Impact

- **constraint_runtime:** an exact shape description for curve kinds (segment, arc) behind the curve capability, plus an identity-based query on Document.
- **New crate `sketch_export`** (platform-neutral, wasm-capable, serde-free): turns a drawing and its typeset label lines into print primitives and writes SVG text.
- **sketch_ui:**
  - "Export SVG…" in the File menu;
  - an export job that waits for typeset lines through the existing label cache;
  - saving through the `Files` host service with `SaveTarget::Ask`, without touching `FileState`'s file or baseline.
- No change to the sketch file format, the command set or the solver.
