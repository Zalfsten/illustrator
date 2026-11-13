# Design: Local, provably bounded mesh refinement

## Context

See proposal.md for motivation and the spec deltas for the contract. Target implementation: impls/rust, crate `constraint_runtime`.

- **`mesh.rs` today** (after #75 and its review), for each curve:
  - **Uniform kinds** (`Geometry::UNIFORM_CURVE`: segment, arc) use `uniform_samples(speed, curvature, ε, cap)`: equal steps, with the count from the exact osculating-circle form `φmax = 4·asin(√(εκ/2))`, capped at π.
  - **All other kinds** use `place_samples(speed, curvature, ε, cap)`. It samples point values on a grid of 64 intervals (bisected up to depth 6 on density jumps) and places vertices by equal shares. While any edge fails `L·κmax ≤ φmax(κmax)`, it raises `n` by one for the whole curve. The function's own doc comment records the limitations #86 lists.
- **No caller** of `place_samples` exists outside tests yet: no non-uniform kind exists.
- **Existing queries:** `Geometry::curvature(t, c, a)` and `Document::curve_curvature` are the point queries. `Geometry::shape` is the precedent for kind-owned, evaluation-only queries.
- **Kind attributes:** kinds receive their attributes (#79), so a function curve can bound its curvature from its expression.
- **Interval evaluation:** `expression::Expression` (#78) evaluates `Dual2<Interval>`, so #81 can bound `|f″|/(1+f′²)^{3/2}` over an x-range.

## Goals / Non-Goals

**Goals:**

- **Soundness:** every edge provably stays within ε unless the cap is reached, including for curvature features between samples.
- **Locality and cost:** a failing edge costs extra edges only near itself. The #86 cases stay within about twice their density integral plus a few edges.
- **No change for segments and arcs:** they keep the closed-form path and its exact minimal counts.

**Non-Goals:**

- **Optimal (minimal) counts for non-uniform curves:** only the bound above.
- **Smoothing edge lengths across refinement boundaries.**

## Decisions

### 1. Piece bound as a kind query

```rust
pub struct PieceBound { pub length: f64, pub curvature: f64 }
fn Geometry::piece_bound(t0: f64, t1: f64, c: Self::Init, a: &Self::Attrs) -> PieceBound
fn Document::curve_piece_bound(id, p, t0, t1) -> Option<PieceBound>
```

The default panics, as `shape` and `curvature` do. A `curve_kinds_report_shape_and_curvature`-style catalog test makes sure every curve kind overrides it.

| kind | length | curvature bound |
|---|---|---|
| segment | `|b − a|·(t1 − t0)` | 0 |
| arc | `|R|·|Δθ|·(t1 − t0)` | `1/|R|` (∞ for R = 0) |
| function curve (#81) | tabulated arc-length chart: upper bound (table length plus a margin) | interval evaluation |

**The length is an upper bound, exact for segment and arc.** (Changed in review of PR #89; the first version required exact lengths.) The function curve's arc-length chart (#81) comes from a table `t → x`, so its piece lengths are exact only up to quadrature and interpolation error, and an under-estimate would make the deviation bound of decision 3 too small. An upper bound is sound: the turning `Σ κ̄ᵢ ℓᵢ` grows with every `ℓᵢ`, and the deviation bound does not decrease when a piece length grows (tested on random pieces, `green_bound_grows_with_piece_lengths`; the randomized check in the review found no decrease in 200 000 cases). The closed-form path for uniform kinds needs the exact length for its minimal count, which segment and arc provide.

*Alternatives:*

- **Point samples plus a documented blind-spot limitation:** the user chose the sound variant.
- **Bounding speed as well and treating lengths as upper bounds:** it complicates the deviation bound (positions become intervals) for no current kind.

### 2. Initial placement from piece bounds

Split [0, 1] into 64 equal cells. Query the piece bound of each cell (ℓᵢ, κ̄ᵢ). The cell's edge demand is

    Dᵢ = ℓᵢ · max( √(κ̄ᵢ / 8ε),  κ̄ᵢ / (π/2) )      (0 if ℓᵢ = 0 or κ̄ᵢ = 0)

**Why these two terms:**
- The first is the small-angle sagitta form. It matches the check in decision 3 exactly for constant curvature: an edge of length `√(8ε/κ)` has bound `κL²/8 = ε`.
- The second keeps the turn of an edge at or below π/2, the other condition of the check.

Using the same small-angle form for placement and check means a constant-curvature non-uniform curve passes on the first try, with no splits.

**Placement.** Take `n = clamp(ceil(ΣDᵢ − 1e-9), 1, cap)` and place vertices at equal shares of the cumulative demand, interpolating linearly inside a cell, as today. This replaces the point-sampled grid, its density-jump refinement (`MAX_REFINE_DEPTH`, `REFINE_RATIO`, `REFINE_MIN_EDGES`) and the `Node` type.

*Alternative:* keep the exact `φmax` form for placement as well. For constant curvature it places edges slightly longer than the small-angle check accepts, so every edge would fail by a hair and be split, doubling the count. Rejected. The closed-form path for uniform kinds keeps the exact form.

### 3. Sound per-edge check from sub-piece bounds

For an edge [ta, tb], split it into M = 4 equal sub-pieces and query (ℓᵢ, κ̄ᵢ) for each. Let `L = Σℓᵢ`, and let the sub-pieces occupy the arc-length ranges `[uᵢ, uᵢ₊₁]` of [0, L].

- **Distance function.** Let `d(s)` be the signed distance of the curve point at arc length `s` from the chord line. Then `d(0) = d(L) = 0` and `|d″(s)| = |κ cos(θ − θ_c)| ≤ κ(s)`.
- **Green bound.** With the Green function of the interval,

      G(s, u) = u(L − s)/L   (u ≤ s),      s(L − u)/L   (u ≥ s),

  the distance satisfies

      |d(s)| ≤ ∫ G(s, u) κ(u) du ≤ B(s) = Σᵢ κ̄ᵢ ∫_{uᵢ}^{uᵢ₊₁} G(s, u) du.

  The integrals are closed-form quadratics. `B` is concave in `s` (a sum of concave tents), and on each piece it is a quadratic in `s` (other pieces contribute linearly), so its maximum is exact from the two ends and the vertex of every piece: at most 4 evaluations per piece, no further queries. (The first version used 100 rounds of ternary search, about 200 evaluations per edge; replaced in review of PR #89.)
- **The edge passes when:**
  1. `max_s B(s) ≤ ε (1 + 1e-9)`: deviation from the chord line;
  2. `Σ κ̄ᵢ ℓᵢ ≤ π/2`: the total turning. The chord direction lies within the range of tangent directions, so every tangent stays within π/2 of the chord. The arc-length coordinate along the chord is then monotone, every point projects inside the edge, and its distance to the edge equals its distance to the chord line.
- **Constant curvature.** `B` peaks at `κL²/8`, the small-angle sagitta.
- **Line followed by a curve.** If a straight piece is followed by a short curved piece at the end of an edge, `G` is small there and the edge passes. This is the case that #86 measured as reaching the cap.

*Alternatives:*
- **`κmax · L` (today):** fails the line-then-curve case.
- **`∫κ ds` alone (the turning):** a turning bound alone does not bound the distance without the length distribution.
- **Measuring the actual distance at sample points:** not sound.

### 4. Local refinement by halving

Check the edges left to right. A failing edge is replaced by its two halves in t, each checked recursively. Refinement stops when:
- the edge passes;
- the edge is narrower than `1e-12` in t;
- the curve's edge count reaches the cap. Remaining edges are then kept unrefined; the spec allows this at the cap.

**Why halving in t rather than shares of the edge's own demand.** Near a junction the demand is concentrated at one end. Equal demand shares then reproduce a long first sub-edge with the same failing shape. Halving moves the boundary geometrically toward the junction. The overhead is about `log₂(ℓ_straight / ℓ_allowed)` edges per junction, for example 9 edges for a 50-unit line before a κ = 1 circle at ε = 10⁻³.

**Cost** in piece queries: 64 for the grid, plus 4 per checked edge (each split adds 2 checks). That is a few thousand at most per curve and mesh build, and only for non-uniform kinds.

### 5. Uniform kinds and the point curvature query

- **Uniform kinds:** `uniform_samples` stays. It reads length and curvature from `piece_bound(0, 1)` instead of `TangentAt` and `curve_curvature` at 0.5, so the mesh uses one query family.
- **`Geometry::curvature` and `Document::curve_curvature` stay** (geometry-ir requires them; #81 may use them for tangents and labels), but the mesh no longer calls them.

## Risks / Trade-offs

- [A kind whose chart is only approximately arc length] → It returns length upper bounds (with a margin), never estimates. The trait doc states this, and the catalog test checks for every kind that the 64 piece lengths sum to at least the length expression, and exactly for uniform kinds.
- [Depth-first halving shares one budget] → When the cap is reached, a region that converges slowly can spend the whole budget before the edges to its right are checked; those keep their initial placement. The spec allows it at the cap, and no current kind reaches it. #81 should refine by priority (largest deviation bound first) if interval bounds make this visible.
- [An unbounded grid cell (κ̄ = ∞ on a piece of positive length)] → The whole curve gets cap edges at equal steps without checks, as the spec allows at the cap.
- [Over-estimated curvature bounds (intervals in #81) cost edges] → They only add edges and never break soundness. Function-curve bounds are tested in #81.
- [The App rebuilds the Mesh for every hit query] → Unchanged cost for segments and arcs. For function curves, caching the Mesh can follow if profiling asks for it (non-goal).
- [Refinement order at the cap favours the left part of a curve] → Only at the cap, where the spec gives no guarantee anyway.
- [Small-angle placement gives non-uniform constant-curvature curves up to one edge more than the exact minimum] → The spec's minimality requirement applies to arcs, which use the exact closed form.

## Migration Plan

Internal change in `constraint_runtime`. `place_samples` changes signature, but only tests call it. No file, command or export change. Rollback is a revert.
