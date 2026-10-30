# Tasks

## 1. label_layout: port, lines, placement

- [ ] 1.1 Create the crate `crates/label_layout` (GPL-3.0-or-later; depends on `constraint_runtime` and `lyon_tessellation`) and add it to the workspace and to `scripts/check-wasm.sh`. Verify that `cargo build -p label_layout` and `scripts/check-wasm.sh` succeed.
- [ ] 1.2 Add the `Typesetter` trait, `TypesetLine`, `Shape`, `Contour`, `Paint` and `TypesetError` (design decision 2), with doc comments that state units (pt), axes (label frame, y up) and origin. Verify: `cargo doc -p label_layout` builds without warnings.
- [ ] 1.3 Add the line splitter: `split_lines(source, markup) -> Vec<&str>` (design decision 3). Verify in `tests_split.rs`:
  - the spec scenarios "Break inside a matrix stays" and "Newline in text markup";
  - `{a\\b}` stays one line;
  - `$a\\b$` stays one line;
  - `\$\\x` splits into two lines;
  - `a\\[2pt]b` gives `a` / `b`;
  - an empty source has no lines;
  - `a\\\\b` gives an empty middle line.
- [ ] 1.4 Add stacking and placement: `place(lines_metrics, attrs, settings, origin, heading) -> Vec<Affine2>` (design decision 4). Verify in `tests_place.rs` with a fake typesetter of fixed metrics:
  - the spec scenarios "Plain lines" (12 pt) and "Tall line pushes apart" (1 pt gap);
  - "Placement without an engine";
  - every halign × valign combination;
  - the `labels` scenario "Offset in a turned frame";
  - an empty middle line still takes 12 pt.
- [ ] 1.5 Add tessellation: `tessellate(&[Shape]) -> (vertices, indices)` in pt, filling glyph contours (non-zero rule) and stroking `Paint::Stroke`. Verify in `tests_tessellate.rs`: a unit square yields 2 triangles with area 1, and a stroked line of width 0.5 yields area length × 0.5 within 1e-6.

## 2. label_typst: Typst + mitex engine

- [ ] 2.1 Create the crate `crates/label_typst` (pins `typst = "=0.15.1"`, `typst-layout = "=0.15.1"`, `mitex = "=0.2.4"`, `ttf-parser`). Copy the assets from `spike/typst-math`:
  - the fonts plus the GUST licence text;
  - mitex's Typst files, with a NOTICE on the Apache-2.0 origin and the xarrow replacement.
  
  Port the shared `World` and the rename table. Verify that `cargo build -p label_typst` succeeds and the rename-table unit test from the spike passes.
- [ ] 2.2 Check mitex text mode first: `latex` lines go through `convert_text(preamble + "\n" + line)`, then `eval(mode: "markup")` with the baseline marker. Verify in `tests_engine.rs`:
  - `velocity $\vec v$`, `$\frac{a}{b}$` and the preamble scenario "Macro from the preamble" typeset;
  - the outlines of `$x \in \R$` and `$x \in \mathbb{R}$` are equal.
  
  If `convert_text` fails here, stop and update design decision 5 (fallback: split at `$` and use `convert_math`) before going on.
- [ ] 2.3 Implement `text` markup (escaped Typst string) and the frame walk into `Shape`s (y flipped up, origin at the left end of the baseline). Height and depth come from the outline points, and the width is the advance width. Verify:
  - the scenario "Text markup is literal";
  - "Single line metrics" for `Ag`;
  - the advance width of `x` equals that of a single `x` glyph at 10 pt from the font.
- [ ] 2.4 Map errors to `TypesetError`. Verify the scenarios:
  - "Unknown command is named" (`\foo`);
  - "Unclosed math" (`$x`);
  - an unknown environment is reported as unsupported with its name;
  - a failing line leaves the next call unaffected.
- [ ] 2.5 Run an end-to-end check through `label_layout` with the real engine: the spec scenario "Tall line pushes apart" (`\displaystyle\sum` over `x`) has baselines more than 12 pt apart and a 1 pt gap within 0.01 pt. Verify with `cargo test -p label_typst`.

## 3. sketch_ui: display

- [ ] 3.1 Inject the typesetter.
  - `App::new(Option<Box<dyn Typesetter>>)`; `with_sample` delegates to it.
  - Native `main.rs` passes `TypstTypesetter`; the wasm `main.rs` passes `None`.
  - `label_typst` sits only in `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]`.
  
  Verify that `cargo build -p sketch_ui` succeeds natively, that `scripts/check-wasm.sh` succeeds, and that `cargo tree --target wasm32-unknown-unknown -p sketch_ui` does not list `typst`.
- [ ] 3.2 Add the per-line cache (key: line, markup, size, preamble) with sweep per frame, and a testable `label_paint(doc, settings, viewport, filter, cache) -> Vec<PaintItem>` that returns triangles in screen space or fallback items. Verify in `tests_labels.rs` with a fake typesetter:
  - "Label follows its geometry";
  - "Zoom keeps proportions";
  - "Hidden construction label";
  - "Labels are painted beside the Mesh";
  - unchanged lines are typeset only once across two frames.
- [ ] 3.3 Add the fallback: egui text in the UI font, rotated, stacked and aligned with `label_layout`, painted in the "not typeset" colour with a dashed frame. The status line names the first failure. Verify in `tests_labels.rs`:
  - "Failure is marked in place" (the fake typesetter fails on `\foo`);
  - "No typesetter available" (`None`): every label becomes a fallback item at its placement.
- [ ] 3.4 Paint the items in `App::update` after the Mesh edges and before the handles. Add one label to the sample scene, joined to an endpoint and parallel to a segment. Verify by running the native app: the label shows and follows a drag of its endpoint. Also check that the sample's file round trip test still passes.

## 4. Docs and integration

- [ ] 4.1 Update the impls/rust notes in the `openspec/config.yaml` context and `impls/rust/README.md`:
  - the two crates and their roles;
  - the engine only native;
  - the fallback in the browser;
  - the Typst/mitex pins and where the assets come from.
  
  Verify that the names match the code.
- [ ] 4.2 Run the full check: `cargo test --workspace`, `cargo clippy --workspace --all-targets` (no warnings), `scripts/check-wasm.sh` and `openspec validate label-typesetting --strict` all pass.

## Workflow follow-up

- Archive the change after review. Close #59; #60 and #63 build on it.
