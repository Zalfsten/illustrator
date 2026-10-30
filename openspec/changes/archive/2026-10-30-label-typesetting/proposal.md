# Proposal

## Why

Since #58 a drawing can hold labels, but nothing shows them. A figure for a paper or a lecture needs its axis names, angle markers and formulas visible next to the geometry, typeset like LaTeX. This change (#59) makes labels visible in the app: typeset by the engine chosen in #11, placed by the anchor frame from #58, and moving with their geometry. Editing labels in the app follows in #63, and loading the engine in the browser in #60.

## What Changes

- **Typesetting contract.** A label's source, markup, size and the document preamble are typeset into lines.
  - Each line delivers its outlines in pt in the label frame, its width along the baseline and its visible extent above and below the baseline.
  - Failures are reported as a result rather than a crash: invalid source, or an unsupported construct (naming the command or environment).
- **Lines are stacked by a fixed rule.** The source is split at its explicit breaks (`\\` at the top level in `latex`, a newline in `text`), and each line is typeset on its own. Consecutive baselines are `max(1.2 · size, depth of the upper line + height of the lower line + 1 pt)` apart, which mirrors LaTeX's `\baselineskip` / `\lineskip`.
- **Markup.**
  - In `latex`, text and `$...$` math are typeset together, and the preamble's macros apply.
  - In `text`, every character is set literally.
- **Display.**
  - Labels are painted from their typeset lines and their placement (origin, heading, alignment, offset, document scale) in world space, so they scale with the drawing when zooming.
  - Construction labels follow the same presentation filter as construction lines.
  - A label that cannot be typeset stays visible at its place and is marked as not typeset.
- **Replaceable engine.** Typesetting is a backend behind a port, like the solver. The model, the session and placement do not depend on any engine.
  - The native app uses a Typst + mitex engine.
  - The browser build has no engine until #60 and shows every label in the "not typeset" form.

## Non-goals

- Creating, selecting, editing or dragging labels in the app, a handle on the label origin, and picking by bounding box (#63).
- Loading the engine in the browser (#60).
- Labels in SVG export (#9), PDF and the PDF+LaTeX overlay (#13), although the port is made to serve them.
- Automatic line wrapping, an "always upright" option, vertical anchors other than first/last baseline (deferred in #58).
- Byte-exact equality with LaTeX metrics: the overlay export is the path for exact LaTeX output.

The typesetting contract and the line-spacing rule are shared specs: every implementation, and every export path, stacks lines the same way. The engine, its version, the fonts and the tessellation are design decisions of the Rust implementation.

## Capabilities

### New Capabilities

- `label-typesetting`: what typesetting a label delivers and how it is displayed:
  - line splitting and line spacing;
  - markup and preamble handling;
  - reported failures and unsupported constructs;
  - the "not typeset" fallback;
  - display in world space with the presentation filter;
  - typesetting as a replaceable backend.

### Modified Capabilities

- `display-mesh`: "Canvas paints from Mesh edges" now names labels as the one kind of geometry painted from its typesetting result and placement, not from Mesh edges.

## Impact

- **New crate `label_layout`** (engine-neutral):
  - the typesetting port and its result types;
  - line splitting and stacking;
  - placement from pt in the label frame to world coordinates.
  - No dependency on any engine or GUI. SVG export (#9) can reuse it.
- **New crate `label_typst`:** the Typst + mitex engine, based on the #11 spike (`spike/typst-math`). Typst and mitex versions are pinned, with an own patched copy of mitex's Typst spec and two embedded fonts (New Computer Modern Math, NewCM10). Linked only in native builds.
- **`sketch_ui`:**
  - paints labels;
  - caches typeset lines;
  - shows the fallback form;
  - gets the engine from `main.rs` (the only place with `cfg(target_arch)`).
- **`constraint_runtime` and `sketch_session`:** unchanged.
- **Build:** native builds gain the Typst dependency tree (release binary about +50 MB, see #11). `scripts/check-wasm.sh` still builds without it.
