# Design: Curvature-based display mesh

## Context

See proposal.md for motivation. Target implementation: impls/rust.

- `constraint_runtime/src/mesh.rs` builds the Mesh: per curve, `sample_normalized_params` bisects `[0, 1]` while the unsigned turn between the tangents at an interval's ends exceeds `MAX_CHORD_TURN` (π/8), capped at `MAX_MESH_SAMPLES` (64). Vertices come from `OnCurve` point expressions; the first and last vertex come from the `Start` / `End` expressions where the kind has endpoints.
- Tangent expressions evaluate the chart derivative, so `|γ′(t)|` (the parametric speed) is already available through the expression path.
- `Geometry::shape` / `Document::curve_shape` is the precedent for an evaluation-only, per-kind query by identity that is not an expression family and has no Jacobian.
- `sketch_ui` builds the Mesh on demand (`App::mesh`, for paint and every hit query) at the committed State; `Viewport` has `scale` (pixels per world unit) and `px_to_world`.

## Goals / Non-Goals

**Goals:**

- A sampler that keeps the deviation within a world-length tolerance, uses as few edges as the tolerance allows on arcs, and works unchanged for future kinds with varying curvature (function curves, #81).
- The sampler is a pure function of "speed at t" and "curvature at t", so it can be unit-tested with synthetic curves (for example a sine) before such a kind exists.

**Non-Goals:**

- Caching the Mesh between paint and hit queries; the App keeps rebuilding it on demand.
- Curvature as a constraint-usable expression (no Jacobian).

## Decisions

### 1. Tolerance as an explicit argument; the App derives it from the view

`Mesh::from_document(doc, p, filter, tolerance)` with `tolerance` in world units, asserted finite and > 0 (like `Viewport::new` asserts its scale). The App passes `viewport.px_to_world(MESH_TOLERANCE_PX)` with `MESH_TOLERANCE_PX = 0.25`, so the display stays within a quarter pixel at every scale and follows zoom (#67) for free, since the Mesh is rebuilt on demand.

*Alternatives:* a `MeshOptions` struct bundling filter and tolerance: more churn for two values, can come later; a fixed world tolerance inside the Mesh: ignores the view scale, which is exactly the scale-blindness this change removes.

### 2. Curvature as a kind query, not an expression family

`Geometry::curvature(t, chart) -> f64` with the panicking default for kinds without the curve capability (as `shape`); segment returns 0, arc `1 / |R|` (`+∞` for `R = 0`). `Document::curve_curvature(id, p, t) -> Option<f64>` resolves by identity and returns `None` without the curve capability (as `curve_shape`).

*Alternative:* a second-derivative vector formula (`γ″`) as a new `VecFormula`, with curvature derived generically. Every formula family must provide a Jacobian and becomes a constraint operand; neither is needed for display, and it widens the opaque expression API. Rejected. #77 (generic formulas with automatic differentiation) may later make `γ″` cheap for every kind; the query can then be implemented generically without changing its contract.

### 3. Density from the osculating circle, exact for arcs

Locally the curve is approximated by its osculating circle of radius `r = 1/κ`. A chord of turn `φ` on that circle has sagitta `r (1 − cos(φ/2))`, so the largest turn within tolerance `ε` is

    φmax(κ) = 2 · acos(clamp(1 − ε·κ, −1, 1))

and the number of edges needed per unit of arc length is `κ / φmax(κ)`. Per unit of the normalized parameter:

    ρ(t) = |γ′(t)| · κ(t) / φmax(κ(t))          (ρ = 0 where κ = 0 or |γ′| = 0)

For small `ε·κ`, `κ / φmax ≈ sqrt(κ / 8ε)`, the formula from #75, and it tends to 0 as `κ → 0`, so straight parts cost nothing.

*Why exact rather than the small-angle form:* the small-angle bound `φ ≤ sqrt(8ε/r)` is slightly smaller than `φmax`, so it sometimes yields one edge more than needed and would violate "as few edges as the tolerance allows" for arcs. The exact form makes the arc's edge count `ceil(|Δθ| / φmax)`, the smallest count within tolerance.

Numerically, `φmax` is evaluated as the equivalent `4 · asin(sqrt(min(ε·κ / 2, 1)))`: `1 − ε·κ` rounds to 1 for tiny `ε·κ`, which would make the acos form 0 and the density infinite.

Guard: `|γ′| = 0` gives `ρ = 0` before multiplying, so a zero-radius arc (speed 0, `κ = ∞`) is a single edge instead of NaN.

### 4. Placement by equal shares of the density integral

The sampler is a private pure function in `mesh.rs`:

    place_samples(speed: impl Fn(f64) -> f64, curvature: impl Fn(f64) -> f64,
                  tolerance: f64, cap: usize) -> Vec<f64>

1. **Grid:** 64 equal intervals in `t`. An interval is bisected (up to depth 6) when the densities at its ends differ by more than a factor of 2 and its trapezoid contribution is at least 0.25 edges. Constant-curvature kinds (segment, arc) never refine.
2. **Integral:** trapezoid rule over the grid gives the cumulative count `C(t)`; `I = C(1)`.
3. **Count:** `n = clamp(ceil(I − 1e-9), 1, cap)`; the small offset keeps an exact integer from rounding up to one edge too many.
4. **Placement:** interior samples `t_j` with `C(t_j) = j·I/n`, `j = 1 … n−1`, by linear interpolation inside the grid interval. For constant density this gives `t_j = j/n` exactly, so arc edges are of equal length.
5. **Verification:** equal shares bound the deviation exactly only for constant curvature; where curvature varies along an edge (next to an inflection, curvature rising linearly from 0) a full share deviates up to about 15 % more than `ε`. Each edge is therefore checked with the comparison bound `L · κmax ≤ φmax(κmax)` (a curve of length `L` whose curvature stays at most `κmax` deviates from its chord no more than a circular arc of that length and curvature), with `L` and `κmax` taken from the grid nodes (linear interpolation at the edge ends). While an edge fails and `n < cap`, `n` grows by one and the placement is repeated. Constant curvature passes at the first `n`, so arcs keep the minimal count and stepwise growth.

Speed comes from the `TangentAt { Normalized }` expression, curvature from `Document::curve_curvature`; vertex positions stay on the expression path (`OnCurve`, `Start`, `End`) as display-mesh requires.

*Alternative:* keep bisection and replace only the stopping test by the sagitta at the interval midpoint. A midpoint test is fooled by symmetry (a full sine period's midpoint lies on its chord), and bisection still produces power-of-two counts on arcs. Rejected.

### 5. Edge cap 1024

`MAX_MESH_EDGES = 1024` per curve replaces `MAX_MESH_SAMPLES = 64` and is documented in `mesh.rs` (spec: at least 256). At 0.25 px a half circle of radius 20 cm at 50 px/cm needs about 220 edges, well below the cap. `MAX_CHORD_TURN` is removed from `mesh.rs` and from the crate exports.

## Risks / Trade-offs

- [More grid evaluations per curve than before (65 speed and curvature evaluations versus up to 64 tangent evaluations), and the App rebuilds the Mesh for every hit query] → Same order of cost as today; curvature for segment and arc is a constant. Caching the Mesh is a separate optimisation if profiling asks for it.
- [Before the first paint the App's viewport has scale 1, so the first Mesh uses a tolerance of 0.25 world units] → Only the first frame is coarse; the Mesh is rebuilt after `Viewport::fit`.
- [Strong zoom makes the tolerance tiny and curves hit the cap] → The cap keeps the cost bounded; the polyline still starts and ends at the curve's ends (spec).
- [Grid refinement is only exercised by synthetic curves until function curves exist] → Unit tests of `place_samples` with a sine (inflection points, equal end tangents) and with a curvature spike cover it now; #81 adds Document-level tests.
- [Tests that pin edge counts of the old algorithm] → `tests_mesh.rs` is updated to the new contract (equal arc edges, minimal count, stepwise growth, near-full arc).

## Migration Plan

Internal API change only: `Mesh::from_document` gains a `tolerance` argument and `MAX_CHORD_TURN` leaves the public exports. All callers are in this repository (`sketch_ui/src/app.rs`, `sketch_ui/src/tests_app.rs`, `sketch_ui/src/tests_labels.rs`, `constraint_runtime/src/tests_mesh.rs`). No file format, command or export change; rollback is a revert.
