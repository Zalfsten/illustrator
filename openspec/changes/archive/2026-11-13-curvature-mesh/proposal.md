# Proposal: Curvature-based display mesh

## Why

The display mesh decides how curves look on the Canvas and where they can be picked. Today it bisects each curve's parameter interval while the tangents at the interval ends differ by more than a fixed turn (π/8), with at most 64 samples. This has three visible defects:

- **Coarse/fine flicker:** for constant curvature the edge count is always a power of two. Dragging an arc across 45°, 90° or 180° sweep doubles or halves its edge count at once.
- **Scale blindness:** the criterion ignores the radius and the view scale. At 22.5° per edge, a 0.5 cm arc deviates from its polyline by 0.01 cm and a 20 cm arc by 0.38 cm, so large arcs look polygonal and small ones are over-sampled.
- **Blind spots:** comparing only the tangents at the interval ends misses curves whose tangents happen to agree there. The turn is measured unsigned in [0, π], so per the current code an arc whose sweep is close to a full turn (more than 337.5°) has end tangents within π/8 of each other and is drawn as a **single chord**. The same defect would draw `sin(x)` on [0, 2π] as one straight edge, so it also blocks function curves (#74, #81).

This advances the product goal's figure types: teaching figures with large arcs and, next, function curves need a display that stays faithful at every size and zoom. Tracked as #75.

## What Changes

- Building a Mesh takes a **chord tolerance**: a world length that bounds how far an edge may deviate from its curve. The App derives it from the view scale (a fraction of a pixel), so the Mesh follows zoom without the Mesh knowing about pixels.
- Curve tessellation is driven by **curvature**: dense where the curve bends strongly, sparse where it is flat. The edge count follows the curve's length and curvature, so it changes by single edges as a curve is dragged, never by doubling.
- For a circular arc the edges are of equal length, and their number is the smallest that keeps the deviation within the tolerance. A line segment stays one edge.
- Every curve kind reports its **curvature** at a normalized curve parameter, as an evaluation-only query by geometry identity (like the exact shape). It is not a constraint input and needs no derivatives.
- The fixed per-curve limit of 64 samples is replaced by a much higher safety cap; the tolerance holds unless that cap is reached.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `display-mesh`: the Mesh is built from Document, State and building options (visibility filter, chord tolerance); edges stay within the chord tolerance; arcs tessellate into equal edges, as few as the tolerance allows; edge counts no longer jump by powers of two.
- `geometry-ir`: curves report their curvature at a normalized curve parameter (segment 0, arc 1/|R|), by identity, without branching on the kind.

## Non-goals

- Pan and zoom of the Canvas (#67). The tolerance follows whatever view scale the App has; this change adds no navigation.
- Function curves themselves (#81) and the singularity handling they need (#83). This change only makes the Mesh ready for curves with varying curvature and inflection points.
- Export. It writes exact shapes, not the Mesh, and is unaffected.
- A curvature expression usable in constraints (for example "curvature equals 1/r"), with Jacobians. Curvature here is an evaluation-only query for display.
- Hit-test radii and handle rendering. Hits keep working on the (finer) edges as before.

The specs stay shared across implementations; the tolerance value, the sampling algorithm and the cap are design decisions of each implementation.

## Impact

- **impls/rust `constraint_runtime`:** `mesh.rs` (new sampler, tolerance argument, removal of `MAX_CHORD_TURN` from the public API), `geometry/formula.rs` and the curve kinds `segment.rs`, `arc.rs` (curvature formula), `document.rs` (curvature query by identity), `lib.rs` exports, `tests_mesh.rs`.
- **impls/rust `sketch_ui`:** `app.rs` passes a tolerance derived from `Viewport::scale`; tests that build a Mesh pass a tolerance.
- No change to the sketch file, commands, export or solver.
