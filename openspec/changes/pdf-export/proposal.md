# Proposal

## Why

LaTeX includes PDF figures natively. `\includegraphics` with pdfLaTeX, LuaLaTeX or XeLaTeX needs no extra package, while SVG needs Inkscape in the build. PDF is also the format arXiv, co-authors, slides and print shops expect. The SVG export (#9) set the export contract and built a format-neutral print scene, so a standalone PDF is now a small step. GitHub issue #13. The PDF+LaTeX overlay is split off into #71 and is not needed yet.

## What Changes

- **Export PDF:** a drawing can be exported as a standalone single-page PDF, saved through the host under `<drawing>.pdf`, or `untitled.pdf` without a file.
- **Same guarantees as SVG (`drawing-export`):**
  - only visible result objects;
  - styles at physical size;
  - labels as filled and stroked outlines, so no fonts are embedded and the text is not searchable;
  - waiting for labels being typeset, and failing with the label named when one cannot be typeset;
  - tight bounds without a margin;
  - deterministic bytes;
  - no change to the drawing, its history, file or unsaved state.
- **Curves in formats without circular arcs:** PDF has no circular-arc primitive.
  - Arcs are approximated by cubic Bézier pieces that stay within 1 µm of the exact circle on paper.
  - Straight pieces stay exact.
  - Formats with an arc primitive (SVG) keep writing arcs exactly.
- **PDF form:**
  - one page whose media box is exactly the tight bounds at physical size;
  - only vector paths (fills and strokes);
  - no fonts, images or external resources;
  - no creation date or file identifier, so equal drawings give equal bytes.
- **Export command:** it now offers SVG and PDF, each with its own suggested name.

## Non-goals

- The PDF+LaTeX overlay (#71).
- CMYK colour, PDF/X or PDF/A. Colours stay sRGB as in styling, which is enough for LaTeX, arXiv and slides.
- Document metadata (title, author), multi-page output, a configurable margin or figure width.
- Searchable or selectable text, and embedded fonts.

This continues the print-output side of the product goal. The specs below are shared contracts. The PDF writer, the Bézier subdivision and the UI wiring are design decisions of impls/rust.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `drawing-export`:
  - **Curves are written from their exact shape:** formats without a circular-arc piece approximate arcs within 1 µm on paper.
  - **Export command:** covers SVG and PDF, with a suggested name per format.
  - **PDF form:** a new requirement for the PDF file itself.

## Impact

- **sketch_export:**
  - new `pdf.rs` over the existing `Scene` and `Bounds`;
  - arc-to-Bézier subdivision bounded by the paper deviation;
  - exact quadratic-to-cubic conversion for glyph outlines;
  - new dependencies `pdf-writer` and `miniz_oxide` for content compression, both wasm-capable.
- **sketch_ui:**
  - File → Export PDF…;
  - `ExportJob` and `FileState::export` take the format: the writer, the extension and `untitled.pdf`.
- No change to the runtime, the sketch file format, commands or the solver.
