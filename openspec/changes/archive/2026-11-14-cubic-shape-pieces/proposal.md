# Proposal: Cubic Bézier shape pieces with a tolerance

## Why

Function curves (#74) are the next step toward the product-goal figure (coordinate axes, a function curve, a tangent and labels). Exports write every curve from its shape, and the shape vocabulary in `geometry-ir` has only straight and circular-arc pieces. A function curve cannot be described with those. This change is **change C** of #74 (#80). The function curve kind (#81, change D) depends on it.

Every output format writes cubic Béziers natively. SVG and PDF already contain them for label outlines.

## What Changes

- **A third shape piece, the cubic Bézier:** start point, two control points, end point. Segments and arcs keep their single exact piece.
- **The shape is queried with a tolerance (BREAKING for the internal shape query; no file or command change).**
  - Kinds with closed-form pieces (segment, arc) ignore the tolerance and stay exact.
  - Other kinds describe their shape as pieces that stay within the tolerance of the curve. The pieces start and end on the curve, and the shape's length agrees with the curve's length expression within the tolerance.
- **Export:**
  - A cubic piece is written as one cubic path piece with its control points, in SVG and PDF.
  - The export asks for shapes at 1 µm on paper at its physical size, the bound that already applies to arcs in PDF.
  - Export bounds stay tight for cubic strokes: they are measured from the curve, not from its control polygon.
- **Operations on pieces work for cubics:**
  - length;
  - point and split at an arc-length fraction, used for trimming strokes at arrow heads;
  - bounding points.
- **Hermite helper for future kinds:** builds cubic pieces from a curve's point and tangent by cubic Hermite interpolation and halves pieces until they are within the tolerance.
  - The deviation check is a pluggable function.
  - This change provides a sampled estimate. A provable bound can be plugged in later (#99).
  - #81 uses the helper.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `geometry-ir`:
  - the shape vocabulary gains cubic Bézier pieces;
  - the shape is queried with a tolerance;
  - closed-form pieces stay exact;
  - other kinds stay within the tolerance in position and length.
- `drawing-export`:
  - cubic pieces are written as cubic path pieces;
  - curves without closed-form pieces are written within 1 µm on paper;
  - bounds of cubic strokes are tight.

## Non-goals

- **A kind that produces cubic pieces.** The function curve kind is #81 (change D). Here cubic pieces are tested through the helper and synthetic shapes.
- **A provable deviation bound.** It needs fourth-derivative bounds that the expression layer (#78) does not provide yet. The check stays pluggable, and #99 tracks the provable variant.
- **The Canvas.** It draws from the display mesh, not from shapes, so it is unaffected.
- **Converting segments or arcs to cubics in the shape.** PDF keeps converting arcs at write time, as today.
- **Further formats** (for example TikZ) and quadratic shape pieces.

The specs stay shared across implementations. The quadrature, the inversion, the deviation estimate and the subdivision rule are design decisions of each implementation.

## Impact

- **impls/rust `constraint_runtime`:**
  - `geometry/shape.rs`: the `Cubic` variant, with `length`, `point_at`, `sub` and `bounding_points` for cubics.
  - A new module for cubic helpers: evaluation, derivative, extrema, de Casteljau split, arc length, parameter at arc length, and the Hermite builder.
  - `geometry/formula.rs`, `document.rs`: `shape` / `curve_shape` take a tolerance.
  - `segment.rs`, `arc.rs`: the new signature, still exact.
- **impls/rust `sketch_export`:**
  - `scene.rs`: the path of a cubic piece and the export tolerance.
  - `bounds.rs`: tight stroked bounds for cubics, reusing the shared cubic helpers.
  - SVG and PDF writers need no new commands.
- **impls/rust `sketch_ui`:** `app.rs`, `drawing_box` passes a tolerance.
- **Docs:** `openspec/config.yaml` (the `shape.rs` line and a line for the new cubic module).
- No change to the sketch file, commands, solver or display mesh.
