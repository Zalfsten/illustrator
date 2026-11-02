# Design

## Context

See proposal.md for the motivation and specs/ for the contract. The design applies to impls/rust.

What #9 left in place:

- **`sketch_export`:** builds a `Scene` of objects with `Stroke` / `Fill` / `Outline` parts and `PathSeg` commands (`Move`, `Line`, `Quad`, `Cubic`, `Arc`, `Close`) in world units. `bounds()` gives the exact tight box, and `svg()` writes it in mm.
- **sketch_ui:** `ExportJob` waits for label lines via `LabelCache::ensure` and calls `export_svg`. `FileState::export(bytes)` asks the host with `export_name()`, which always ends in `.svg`. `App` keeps `Exporting::{Job, Bytes}`, and the File menu has one "Export SVG…" entry.
- **Web host:** every download is a Blob of type `application/json`, whatever the file name.

## Goals / Non-Goals

**Goals:**

- One PDF writer over the existing scene and bounds, with no second path through the drawing.
- Arc approximation provably within 1 µm on paper, with few pieces for small arcs.
- Deterministic bytes, testable by parsing the PDF.

**Non-Goals:**

- Object names in the PDF. PDF has no element ids; optional content groups would be overkill.
- Reusing glyph outlines as XObjects.

## Decisions

### D1: A format enum shared by export and UI

`sketch_export::Format::{Svg, Pdf}` provides:

- `extension()`, which is `"svg"` or `"pdf"`;
- `mime()`, which is `image/svg+xml` or `application/pdf`;
- `export(doc, labels, format) -> Result<Vec<u8>, ExportError>`, which builds the scene and bounds once and calls the writer.

`export_svg` stays as a convenience for the tests. `ExportJob::new(doc, format)` carries the format, and `App::Exporting::Bytes` keeps it next to the bytes. The menu has `MenuItem::Export(Format)` with the entries "Export SVG…" and "Export PDF…".

Alternative: a second job type for PDF. Rejected, because the label handling is identical and only the final writer call differs.

### D2: PDF writer with `pdf-writer`

`pdf.rs` uses `pdf-writer` 0.15, which is small, has no dependencies and builds for wasm. The objects are a catalog, a page tree and one page.

- **Page box:** `MediaBox = [0 0 W H]` in bp, with W and H being the bounds times `scale · 72/25.4`.
- **No metadata:** no Info dictionary and no file ID.
- **Coordinates:** PDF's y axis points up like the world, so there is no flip: x' = (x − min.x)·k and y' = (y − min.y)·k.
- **Content stream, per part:**
  - colour with `rg` (fill) or `RG` (stroke) in DeviceRGB, the colour values being 0–255 divided by 255;
  - `w` for width, `0 J` (butt caps) and `0 j` (miter joins);
  - `[d g …] phase d` for dashes, or `[] 0 d` to reset to solid;
  - path operators `m l c h`, ending with `S` for strokes and outlines or `f` (non-zero) for fills.

The graphics state is set explicitly for every part, so the parts do not depend on their order.

Alternative: our own minimal writer, which is feasible without fonts. Rejected, because cross-reference offsets and object numbering are exactly what `pdf-writer` gets right, and it is the writer Typst uses.

### D3: Arcs as cubic Béziers within 1 µm on paper

Each `PathSeg::Arc` is split into n equal parts of turn θ = |sweep|/n. Each part becomes one cubic with control distance k = 4/3 · tan(θ/4) · r. Its largest radial deviation is e(θ) ≈ 2.73·10⁻⁴ · r · (θ / (π/2))⁶ for θ ≤ π/2.

- **Choosing n:** n is the smallest integer ≥ ⌈|sweep| / (π/2)⌉ for which e(θ) · r_paper ≤ 0.5 µm. The factor 2 of headroom covers the approximation of e and the f32 coordinates of D5.
- **Exact end points:** the end points of the parts are computed from the circle (`on_circle`), so pieces join without gaps and the arc ends exactly at its end points.
- **Tests:** they measure the deviation numerically by sampling each cubic densely against the circle. They do not rely on the formula.

### D4: Quadratic glyph pieces as exact cubics

Typst outlines contain `QuadTo`, and PDF has only cubics. The degree elevation c1 = p0 + ⅔(c − p0), c2 = p1 + ⅔(c − p1) is exact.

### D5: Numbers and compression

- **Numbers:** `pdf-writer` writes `f32`. On a page of up to about 1000 bp that is a resolution of roughly 6·10⁻⁵ bp ≈ 0.02 µm, well inside the 1 µm budget.
- **Compression:** the content stream is compressed with `miniz_oxide::deflate::compress_to_vec_zlib` at level 6 and marked `/FlateDecode`. Glyph outlines shrink by a factor of 3 to 5.
- **Determinism:** the dependency versions are pinned in Cargo.lock, so the same input gives the same bytes.

### D6: Export name and MIME type per format

- **Export name:** `FileState::export_name(ext)` and `FileState::export(bytes, format, files)` use `UNTITLED_EXPORT_STEM = "untitled"` plus the extension. The `.sketch.json` / `.json` stripping rule from #9 stays.
- **MIME type:** the web host derives the Blob type from the suggested name's extension: `.svg` → `image/svg+xml`, `.pdf` → `application/pdf`, otherwise `application/json`. The `Files` trait stays unchanged.

### D7: Tests parse the PDF with `lopdf`

`lopdf` is a pure-Rust dev-dependency of `sketch_export`. The tests load the bytes and check:

- one page and its MediaBox;
- no `/Font` or `/XObject` resources;
- no `/Info`, `/ID` or `CreationDate`;
- the decompressed content operators.

They also check byte equality on re-export, and the arc deviation of D3 on the parsed `c` operands, mapped back to mm.

## Risks / Trade-offs

- **[f32 rounding]** → It stays below 0.05 µm for practical page sizes, and the 0.5 µm design target leaves headroom.
- **[Viewer dash rendering on Béziers]** → PDF viewers lay dashes along the flattened path. That is how PDF works, and the phase stays correct. Checked visually.
- **[`pdf-writer` and `miniz_oxide` updates could change bytes]** → Determinism is promised for the same build. The versions are pinned in Cargo.lock.
- **[Download MIME change]** → It affects only the web host's Blob type. Saving sketch files keeps `application/json`.
