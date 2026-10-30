# Design

## Context

See proposal.md for the motivation and `specs/label-typesetting` for the requirements. This design targets `impls/rust`.

- **#58 is in place:**
  - `Label` geometries with origin and heading;
  - `LabelAttrs` (source, markup, halign, valign, offset in pt, size);
  - `DocumentSettings` (scale in mm per world unit, font size, preamble);
  - the `labels` spec, which defines the frame, the reference point and the alignment.
  - Labels contribute no Mesh edges and have no handle yet.
- **The #11 spike** (local branch `spike/typst-math`, crate `spikes/typst_math`) already has the engine pipeline:
  - `mitex::convert_math`;
  - a rename table for symbols that Typst 0.15 removed;
  - a `World` with two embedded fonts and mitex's Typst spec (xarrow replaced);
  - one `eval` of the converted source;
  - a baseline marker glyph;
  - frame walk → glyph outlines (ttf-parser) + shapes → lyon tessellation.
  
  It handles one line of math at a fixed size. It measured 0.28 ms per re-typeset natively and 11.6 ms for the first one.
- **mitex 0.2.4** also has `convert_text`: `$...$` becomes `#math.equation(block: false, $...$)`. An unknown command or environment fails the whole conversion with `unknown command: \name` / `unknown environment: \name`. `\\` is passed through as token text, so line breaks must not be left to mitex.
- **`sketch_ui`:**
  - paints Mesh edges and handles through `Viewport` (world y-up ↔ screen);
  - `main.rs` is the only place with `cfg(target_arch)`;
  - `scripts/check-wasm.sh` builds the three crates for `wasm32-unknown-unknown`.

## Goals / Non-Goals

**Goals:**
- A port that serves the app now and SVG/PDF export later: exact outlines, not triangles.
- Line splitting, stacking and placement are engine-independent and tested without Typst.
- The browser build stays as it is (no Typst) and shows labels in the fallback form.

**Non-Goals:**
- Label UI interaction (#63).
- A second engine.
- Matching LaTeX metrics exactly.

## Decisions

### 1. Two new crates

```
constraint_runtime   sketch_session
        ^                  ^
        |                  |
   label_layout  <--  label_typst (native only)
        ^                  ^
        +------ sketch_ui -+  (label_typst via [target.'cfg(not(wasm32))'])
```

- **`label_layout`** is engine-neutral and pure Rust. It holds:
  - the `Typesetter` trait and the result types;
  - the line splitter;
  - stacking;
  - placement;
  - tessellation of outlines into triangles for display, with `lyon_tessellation`.
  
  It depends on `constraint_runtime` for `Point2` / `Vec2` / `LabelAttrs` / `DocumentSettings`. It builds for wasm32 and is added to `check-wasm.sh`.
- **`label_typst`** implements `Typesetter` with Typst + mitex. Only `sketch_ui`'s native target table and `label_typst`'s own tests link it.

*Alternative:* put the port in `constraint_runtime`. Rejected: typesetting is unrelated to solving, and the core would grow a tessellation dependency. *Alternative:* keep everything in `sketch_ui`. Rejected: SVG export (#9) needs the port without egui.

### 2. Port and result types

```rust
pub trait Typesetter {
    /// One line, already split; `preamble` only for Markup::Latex.
    fn typeset_line(&self, line: &str, markup: Markup, size_pt: f64, preamble: &str)
        -> Result<TypesetLine, TypesetError>;
}
pub struct TypesetLine {
    pub shapes: Vec<Shape>,   // in pt, label frame, origin = left end of baseline, y up
    pub width: f64,           // advance width as set
    pub height: f64,          // visible extent above the baseline
    pub depth: f64,           // visible extent below the baseline
}
pub struct Shape { pub contours: Vec<Contour>, pub paint: Paint }  // Paint::Fill | Paint::Stroke(width)
pub enum Contour cmd: MoveTo / LineTo / QuadTo / CubicTo / Close (pt)
pub enum TypesetError { Unsupported { construct: String }, Invalid { message: String } }
```

- **The port works per line.** Splitting and stacking are engine-neutral (spec), so engines only set lines. This also makes the cache per line (decision 6).
- **Outlines are exact paths in pt, y up.** The engine flips Typst's y-down frame. Glyph outlines are filled. Typst strokes (fraction bars, radicals) keep `Paint::Stroke`, so SVG export can write them 1:1.
- **Height and depth are measured from the outlines** (the bounding box of the path points, control points included). This holds for any engine and is close enough for the 1 pt rule. Typst's `top-edge`/`bottom-edge: "bounds"` is not needed.

### 3. Line splitting (label_layout)

A small scanner, not a full LaTeX parser.
- **`latex`:** track `{}` depth, `$` math (toggles; `\$` is a literal), and `\begin{..}` / `\end{..}` depth. Split at `\\` only when all three are at the top level, and skip the optional `[len]` after `\\` (LaTeX's `\\[2pt]`, whose length is ignored).
- **`text`:** split at `\n`.
- An empty source has no lines. An empty line between breaks is a line with no shapes: it has no extent, so it gets height and depth 0 and still takes its 1.2 × size of spacing.

*Alternative:* split with `mitex-parser`'s syntax tree. Rejected for now: it is an extra dependency for a rule that a 50-line scanner covers, and the browser fallback needs the same splitter without mitex.

### 4. Stacking and placement (label_layout)

- **Stacking:** `baseline[i+1] = baseline[i] − max(1.2·size, depth[i] + height[i+1] + 1)` in pt, with the first baseline at 0 (y up).
- **Block anchor:** `valign = first` puts baseline 0 on the reference point; `last` shifts the block so that the last baseline is there.
- **Horizontal alignment:** each line is shifted by `0`, `−width/2` or `−width` for `left` / `center` / `right`.
- **Placement** is an affine map from pt in the label frame to world: `world = origin + R(heading) · (offset + p) · k`, where `k = (25.4 / 72.27) / scale` world units per pt.
- The pieces are separate, so that typeset lines and the fallback share them:
  - `line_origins(metrics, attrs, size)` gives each line's origin in the label frame (pt), after alignment and offset;
  - `frame_map(settings, origin, heading)` gives the frame → world `Affine2`;
  - `place` composes them into one `Affine2` per line (`frame_map(..).then_translate(origin_i)`).
  
  Display and later export apply the same maps.

### 5. Typst engine (label_typst)

The spike code is ported, changed as follows:
- **Version pins:** `typst = "=0.15.1"`, `typst-layout = "=0.15.1"`, `mitex = "=0.2.4"`. The assets are byte-identical to the spike's:
  - the two fonts, taken from `typst-assets` 0.15.1, with a NOTICE quoting its licence sections: NewCMMath-Regular is under the GUST Font License, NewCM10-Regular under GPL-3.0-or-later with font exception;
  - mitex's Typst files (Apache-2.0), with xarrow replaced by `math.stretch` and a NOTICE.
- **`latex` markup:** `mitex::convert_text(preamble + "\n" + line)`, so `\newcommand`s in the preamble are defined for the line, then the rename table. The table applies only inside `$…$` (outside `[…]` text content). Words of the surrounding markup text, such as "diff", stay unchanged. The main source does `#box(eval(.., mode: "markup", scope: mitex-scope))`, followed by the baseline marker.
- **`text` markup:** the line becomes a Typst string expression, `#"…"` with `\` and `"` escaped. It goes through the same `eval`; no mitex is involved.
- **Fonts:** `#set text(font: "New Computer Modern", size: <size>pt)`. Math uses Typst's default math font, New Computer Modern Math, which is the other embedded font.
- **Errors:**
  - a mitex error starting with `unknown command: ` / `unknown environment: ` → `Unsupported { construct }`;
  - other mitex errors and Typst diagnostics → `Invalid { message }`.
- **World:** one shared `Library` / `FontBook` / fonts (`OnceLock`, as in the spike), and a new `World` per line.

### 6. Display in sketch_ui

- **Engine injection:** `App::new(typesetter: Option<Box<dyn Typesetter>>)`. Native `main.rs` passes `Some(Box::new(TypstTypesetter::new()))`, the browser passes `None`. `App::with_sample()` (tests only) is `new(None)`. No `cfg` is added outside `main.rs` and the Cargo target table.
- **Cache:** `LabelCache` owns the typesetter and a `HashMap<LineKey, Result<(LineMetrics, Triangles), TypesetError>>` with `LineKey = (line text, markup, size bits, preamble)`. Only metrics and triangles are kept, because display needs nothing else. Entries not used during a frame are dropped at the end of that frame, so edits do not accumulate.
- **Paint data:** `label_paint(doc, viewport, filter, cache) -> Vec<PaintItem>` (in `labels.rs`) works without a window. It returns `Typeset` (screen triangles) or `Fallback` (lines, dashed frame corners, error). `App::paint_label` turns the items into egui shapes.
- **Paint:** for each visible label (role filter as for the Mesh):
  - if every line typesets, place the lines and transform the triangles pt → world → screen, then paint them as an `egui::Mesh` in the stroke colour of their role;
  - if any line fails, or there is no typesetter, draw the **fallback** instead.
- **Fallback:** the source lines as egui text in the UI font, rotated to the heading. Font size in px is `size_pt · k · viewport.scale`. The lines are stacked and aligned with the same `label_layout` functions (`line_origins`, `frame_map`). The metrics are fixed monospace proportions (advance 0.6 em per character, ascent 0.75 em, descent 0.25 em) rather than the galley's measurements, so `label_paint` needs no egui font context and stays testable. The text is drawn in egui's monospace font, so the frame fits it closely; any small mismatch is acceptable for a form marked as not typeset. They are painted in an orange "not typeset" colour, with a thin dashed frame around the block. The status line names the first failure (for example "lbl2: unsupported \foo").

*Alternative:* hide failed labels. Rejected by the spec ("stay visible").

### 7. Tests

- **`label_layout`:** splitter cases (groups, `$`, environments, `\\[2pt]`, escaped `\$`, empty lines), stacking with fixed metrics, and placement for each halign/valign, offset, heading and scale, using a fake typesetter. This covers "Placement without an engine".
- **`label_typst`:** one test per spec scenario of `label-typesetting` that needs a real engine:
  - metrics of `Ag`;
  - a matrix with `\\` on the second line;
  - preamble macro vs `\mathbb{R}` with equal shapes;
  - text markup is literal;
  - `\foo` is unsupported;
  - `$x` is invalid;
  - the tall line with its 1 pt gap.
- **`sketch_ui`:** the paint data is built in a function that can be tested without a window (`label_paint(...) -> Vec<PaintItem>`). It covers:
  - the label follows a dragged endpoint;
  - zoom doubles both size and distance;
  - a construction label is hidden in result mode;
  - with no typesetter, the fallback is used.

## Risks / Trade-offs

- **Height and depth from path points, control points included,** can overestimate the visible extent slightly (a few hundredths of a pt on round glyphs). → This is accepted, and the spec scenario tolerance (0.01 pt) refers to our own measure. If it ever matters, compute exact Bézier extrema.
- **mitex text mode is not covered by the spike.** → The first `label_typst` task checks `convert_text` on the spec scenarios before anything is built on it. If it fails, fall back to splitting at `$` and `convert_math` for the math parts.
- **Prepending the preamble to every line** costs a little per typeset. → It is negligible at 0.3 ms per line; the per-line cache hides it while typing.
- **Native build time and size grow** (Typst dependency tree; release binary +50 MB). → This is accepted (#11). The wasm build is unaffected.
- **The first typeset takes about 12 ms** (font book, library). → It happens once, on the first frame with a label.
- **Upside-down labels** (anti-parallel solutions) render as specified in `labels`. #42 handles this; there is nothing to do here.

## Open Questions

- The exact orange of the fallback and the line pattern of its frame are cosmetic and get settled during implementation.
