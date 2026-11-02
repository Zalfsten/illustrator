# Tasks

## 1. Format and PDF writer (sketch_export)

- [x] 1.1 Add `Format::{Svg, Pdf}` with `extension()`, `mime()` and `export(doc, labels, format)` building scene and bounds once (D1); keep `export_svg`. Verify that the existing SVG tests still pass and that a new test shows `Format::Svg` gives the same bytes as `export_svg`
- [x] 1.2 Add the dependencies `pdf-writer` 0.15 and `miniz_oxide`, plus the dev-dependency `lopdf`. Verify with `cargo build -p sketch_export` and `scripts/check-wasm.sh`
- [x] 1.3 Implement arc-to-cubic subdivision (D3) and exact quadratic-to-cubic elevation (D4) as pure functions. Verify with `src/tests_pdf.rs`:
  - dense sampling shows deviation ≤ 1 µm on paper for r = 100 mm with sweep 3π/2, and for r = 1 m;
  - a 1 mm quarter arc becomes one piece;
  - parts join without gaps and end exactly at the arc's end points;
  - an elevated quadratic matches the original at sampled parameters.
- [x] 1.4 Implement `pdf.rs` (D2, D5): one page with MediaBox in bp; DeviceRGB colours, widths, butt caps and miter joins, dash arrays with phase; fills with the non-zero rule; the content stream Flate-compressed; no Info and no ID. Verify with `tests_pdf.rs` using `lopdf`:
  - one page with a MediaBox of 30 mm × 0.4 pt for the reference segment;
  - no `/Font`, `/XObject`, `/Info`, `/ID` or date;
  - the content operators for stroke, fill, dash and colour;
  - arc pieces as `c` operators within 1 µm;
  - identical bytes on a second export;
  - `NothingToExport` for an empty scene.

## 2. App integration (sketch_ui)

- [x] 2.1 Make `FileState::export_name(ext)` / `export(bytes, format, files)` per format (D6). Verify with `tests_files.rs`: `triangle.sketch.json` gives `triangle.pdf` and `triangle.svg`, a drawing without a file gives `untitled.pdf`, and the existing export tests are kept
- [x] 2.2 Carry the format through `ExportJob`, `Exporting::Bytes` and `MenuItem::Export(Format)`, and add "Export PDF…" below "Export SVG…" (D1). Verify with `tests_export.rs` and `tests_app.rs`:
  - the PDF export of the sample scene reaches `FakeFiles` as `untitled.pdf`, with bytes starting `%PDF-`;
  - waiting for labels and naming failing labels work for PDF as for SVG.
- [x] 2.3 Let the web host derive the download's MIME type from the suggested name's extension (D6). Verify that `scripts/check-wasm.sh` passes and that a unit test of the extension → MIME mapping covers svg, pdf and json
- [x] 2.4 Update the README export section (PDF entry, Béziers within 1 µm, page = tight bounds, no fonts) and the impls/rust notes in `openspec/config.yaml`. Verify that `openspec validate pdf-export --strict` passes

## 3. Integration checks

- [x] 3.1 Run `cargo test --workspace`, `cargo clippy --workspace --all-targets` (no warnings), `scripts/check-wasm.sh` and `openspec validate pdf-export --strict`, and verify that all pass
- [x] 3.2 Export the styled test drawing as PDF in the browser build (download), render it in Chromium's PDF viewer or with a rasterizer, and verify that it matches the SVG export and the Canvas: arrows, dashes, arc, label position and colour, tight page box. Include it in a LaTeX document with `\includegraphics` if a TeX installation is available

## Workflow follow-up

- Commit on branch `issue-13-pdf-export`, push and open the PR for #13 after confirmation.
- Archive the change with spec sync after review.
