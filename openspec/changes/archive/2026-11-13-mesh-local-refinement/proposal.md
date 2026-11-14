# Proposal: Local, provably bounded mesh refinement

## Why

The display mesh samples curves with varying curvature badly (#86). Function curves (#81) are the first such kind, so this must land before them.

- **Global growth:** one failing edge raises the edge count of the whole curve by one and re-places every vertex. In the worst case that is quadratic in the edge cap.
- **Over-conservative check:** each edge is checked against its largest curvature times its whole length. An edge that covers a straight stretch and ends just inside a curved part therefore practically never passes. Measured with the current sampler (ε = 10⁻³):

  | curve | edges |
  |---|---|
  | half line, half circle | 1024 (the cap) |
  | line of length 2 to 50, then a quarter circle | 1024 |
  | a curvature ramp | about 17× the density integral |

- **Blind spots:** curvature is only seen at sample points. A spike narrower than a grid interval can break the display-mesh requirement that every point lies within the chord tolerance.

None of this shows today, because segments and arcs take the closed-form path for uniform curves. It would show with the first function curve.

This change keeps the product goal's teaching figures (a function curve with a tangent) drawable at sensible cost and provably within the tolerance.

## What Changes

- **New query, piece bound:** every curve kind reports, for a normalized parameter interval [t0, t1], the arc length of that piece and an upper bound of its curvature on the whole piece. It is evaluation-only, like the curvature query.
  - Segment: length · (t1 − t0) and curvature 0.
  - Arc: |R|·|Δθ|·(t1 − t0) and 1/|R|.
  - Function curves (#81) will bound curvature by interval arithmetic (#78).
- **Sampler built on piece bounds instead of point samples:**
  - **Initial placement:** equal shares of the density integral, as today.
  - **Check per edge:** a sound and tighter deviation bound built from the piece bounds of a few sub-pieces of the edge, so a short curved part near an edge's end costs little.
  - **Local refinement:** only failing edges are split, in halves, until they pass or the cap is reached.
- **Spikes:** a narrow curvature spike can no longer be missed, because the bounds cover whole pieces, not points.
- **Uniform kinds unchanged:** segments and arcs keep the closed-form placement, so their edges stay equal and minimal in number.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `geometry-ir`: curves report the length and a curvature upper bound of a parameter piece; the evaluation-only rule covers this query too.
- `display-mesh`: curve tessellation refines locally; a straight stretch next to a curved part costs few edges; curvature features narrower than the sampling are still kept within the tolerance.

## Non-goals

- **The function curve kind** (#81) and its interval-based bound. This change defines the query and implements it for segment and arc.
- **Caching the Mesh** between paint and hit queries.
- **Changing the tolerance value, the cap or the closed-form path** of uniform kinds.
- **Export.** It uses exact shapes, not the Mesh.

The specs stay shared across implementations. The deviation bound, sub-piece count and split rule are design decisions of each implementation.

## Impact

- **impls/rust `constraint_runtime`:**
  - `geometry/formula.rs`: a new `Geometry` method with a panicking default.
  - `geometry_kinds/segment.rs`, `arc.rs`: the new method implemented.
  - `document.rs`: the query by identity.
  - `mesh.rs`: the new sampler, which replaces the point-sample grid, the global `n + 1` loop and `edges_within_tolerance`.
  - `tests_mesh.rs`: the `place_samples` tests move to the new signature, and the #86 measurement cases are added with upper bounds.
- **Docs:** `docs/constraint-mathematics.md` (Mesh row) and `openspec/config.yaml` (`mesh.rs` line).
- No change to the sketch file, commands, export or solver.
