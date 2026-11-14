# Design: Cubic Bézier shape pieces with a tolerance

## Context

See proposal.md for motivation and the spec deltas (`geometry-ir`, `drawing-export`) for the contract. Target implementation: impls/rust, crates `constraint_runtime`, `sketch_export`, `sketch_ui`.

- **Shapes today** (`constraint_runtime/src/geometry/shape.rs`): `ShapePiece::{Line, Arc}` with `length`, `point_at(s)` and `sub(s0, s1)`. Here `s` is a **fraction of arc length**, and the trimming at arrow heads relies on it. There is also `bounding_points`.
  - `Geometry::shape(c, a)` takes no tolerance; `Document::curve_shape(id, p)` reads it by identity.
  - Callers: `sketch_export/src/scene.rs` (`curve_parts` → `trim` → `path`) and `sketch_ui/src/app.rs` (`drawing_box` → `bounding_points`).
- **Export paths** (`sketch_export/src/scene.rs`): `PathSeg::Cubic` already exists, because label outlines contain cubics.
  - The SVG writer emits `C`, the PDF writer emits `c`.
  - PDF converts arcs to cubics at write time within `ARC_TARGET_MM` = 0.5 µm (`arc_cubics`).
- **Export bounds** (`sketch_export/src/bounds.rs`):
  - Fills: cubic extrema in closed form (`extrema_cubic`), so they are tight.
  - **Stroked cubics:** the control-polygon box widened by half the width (`widened_box`), which is conservative, not tight. Today this only affects stroked label outlines.
- **Canvas:** draws from the display mesh, never from shapes. It is unaffected.
- **No kind produces cubic pieces yet.** The first will be the function curve (#81). It has an arc-length chart, so `γ(t)` and `γ′(t)` are available from its table, while derivative bounds beyond `f″` are not (#78 provides `Dual2`).

## Goals / Non-Goals

**Goals:**

- **Cubic pieces through the whole export path:** shape, trim, path, SVG, PDF and tight bounds.
- **A reusable builder** for kinds without closed-form pieces. Its deviation check is pluggable, so a provable bound can replace the estimate without changing callers.
- **No behaviour change for segments and arcs**, including their exported bytes.

**Non-Goals:**

- A provable deviation bound (follow-up issue).
- Minimal piece counts: the builder only has to stay within the tolerance at a reasonable count.

## Decisions

### 1. `ShapePiece::Cubic` keeps the arc-length fraction contract

```rust
ShapePiece::Cubic { start: Point2, c1: Point2, c2: Point2, end: Point2 }
```

The control points are locations (`Point2`). Derivatives are free vectors (`Vec2`). The fraction `s` in `point_at(s)` and `sub(s0, s1)` stays a fraction of **arc length**, as for lines and arcs:

- **`length`:** adaptive Gauss–Legendre quadrature of `|B′(u)|`.
  - 8 points per interval;
  - halve while the two halves disagree with the whole by more than 1e-13 relative;
  - depth at most 20.
- **`point_at(s)` / `sub(s0, s1)`:** find the Bézier parameter `u` with `length(0, u) = s · length`.
  - Newton on `|B′(u)|`, safeguarded by bisection, to 1e-13 relative in length (the quadrature's own accuracy).
  - Then evaluate, or split with de Casteljau at `u0` and `u1` (`sub` re-parameterizes the middle part).
- **`bounding_points`:** the ends plus the points at the closed-form axis extrema.

*Alternative:* let `s` be the Bézier parameter for cubics. Rejected: `trim` would cut at wrong lengths, and styling requires the stroke to end at arc length L from the endpoint.

### 2. Shared cubic helpers in `constraint_runtime`

A new module `geometry/cubic.rs` with free functions on the four points:

- evaluation and derivative;
- axis-extremum parameters, moved from `sketch_export/src/bounds.rs`;
- de Casteljau split;
- arc length on `[u0, u1]`;
- parameter at length;
- the Hermite builder (decision 4).

`sketch_export::bounds` calls these instead of its own copies (`cubic`, `extrema_cubic`). The quadratic helpers stay in `bounds.rs`, because only label outlines have quadratics.

*Alternative:* a separate crate. Not worth it for a few functions that the core shape type itself needs.

### 3. The shape query takes a tolerance

```rust
fn Geometry::shape(c: Self::Init, a: &Self::Attrs, tolerance: f64) -> Vec<ShapePiece>
fn Document::curve_shape(&self, id: GeometryId, p: &Vector, tolerance: f64) -> Option<Vec<ShapePiece>>
```

- The tolerance is in world units and must be > 0 (debug assertion).
- Segment and arc ignore it.
- Callers:
  - **export:** `SHAPE_TOLERANCE_MM / scale` with `SHAPE_TOLERANCE_MM = 0.5e-3`. That is half the spec's 1 µm, leaving room for rounding and for the estimate (decision 4). It is the same margin as `ARC_TARGET_MM`.
  - **`drawing_box`:** the same `export_tolerance(settings)`. (Changed during implementation; the first version used the current view's mesh tolerance.) `drawing_box` uses shapes instead of the Mesh precisely because the Mesh's tolerance follows the view this box is about to replace. A view-dependent shape tolerance would bring that dependency back.

*Alternatives:*
- **A fixed tolerance per kind:** the export's 1 µm on paper depends on the document scale, which kinds do not know.
- **Exporters approximate the curve themselves,** as PDF does for arcs: they would need to evaluate the curve, which only the kind can do, and the shape vocabulary would no longer be kind-independent.

### 4. Hermite builder with a pluggable deviation check

```rust
pub fn hermite_shape(
    point: impl Fn(f64) -> Point2,            // γ(t)
    tangent: impl Fn(f64) -> Vec2,            // γ′(t) = dγ/dt
    knots: &[f64],                            // ascending, from 0 to 1
    tolerance: f64,
    deviation: impl Fn(f64, f64, &ShapePiece) -> f64,  // (t0, t1, piece) -> deviation
) -> Vec<ShapePiece>
```

- **Piece on `[t0, t1]`** with `h = t1 − t0`: `start = γ(t0)`, `c1 = γ(t0) + γ′(t0)·h/3`, `c2 = γ(t1) − γ′(t1)·h/3`, `end = γ(t1)`. That is cubic Hermite interpolation in Bézier form.
  - Ends and joints lie exactly on the curve, and the tangent is continuous across joints. This satisfies the spec's "piece ends lie on the curve".
- **Knots:** the caller passes the knots, at least `[0, 1]`. #83 will pass branch points so pieces never straddle a corner.
  - Each knot interval starts as 4 equal pieces, so that a sampled estimate is not fooled by a single huge piece.
- **Refinement:** depth-first halving of every piece whose `deviation` exceeds `tolerance`. A shared cap `MAX_SHAPE_PIECES = 4096` applies, after which the remaining pieces are accepted as they are (documented, like the mesh cap).
- **Default check `sampled_deviation(point)`:** the largest distance `|γ(t0 + k·h/8) − B(k/8)|` for `k = 1…7`, times a safety factor 2.
  - The distance at matched parameters is at least the geometric distance at those samples, so it errs on the large side there.
  - Between samples it is an estimate. This is the user's choice for this change ("estimate now, proof later").
- **Length:** the spec's length agreement is not checked by the builder, which does not know the curve's length.
  - For Hermite pieces within δ, the length error is of higher order than δ.
  - Tests check it against a curve with a known length.
  - The follow-up issue covers proving it together with the deviation.
- **Determinism:** a pure function of its inputs, so exports stay byte-identical across runs.

*Alternatives:*
- **Sound bound now:** needs a fourth-derivative bound per piece (Hermite error ≤ h⁴/384 · max|γ⁗|), which #78 cannot provide. Deferred to the follow-up issue. The `deviation` parameter is the plug.
- **Least-squares or arc-length-matched fitting:** fewer pieces, but iterative and harder to bound. Not needed for 1 µm.

### 5. Export writes cubic pieces as they are

`scene.rs::path` maps `ShapePiece::Cubic` to `PathSeg::Cubic(c1, c2, end)`. The SVG and PDF writers already handle `PathSeg::Cubic`, so they need no change. `trim` works unchanged on top of decision 1. Arcs are still converted only in the PDF writer, so segment and arc exports keep their bytes.

### 6. Tight bounds for stroked cubics

For a butt-ended stroke of half-width `h` along a cubic `B`, the region's x-extent (and likewise y) is reached at one of these:

- **End corners:** `B(0) ± h·n(0)` and `B(1) ± h·n(1)`, with `n` the unit left normal.
- **Interior critical points:** `f(u) = Bₓ(u) ± h·nₓ(u)` has `f′ = B′ₓ·(1 ∓ hκ)`. Critical points are therefore at:
  - the axis-extremum parameters of `B`, where `B′ₓ = 0` (closed form, decision 2);
  - parameters where `hκ = 1` (offset cusps).

`stroked()` adds the end corners and `B(u) ± h·n(u)` at every axis-extremum parameter. That is exact as long as `hκ < 1` on the piece.

If the curvature, sampled at 32 parameters, reaches `0.5/h` somewhere on the piece, the piece additionally gets a numeric maximization of `f` for each of the four directions: golden-section search on each sample bracket around the largest sample. This keeps the bounds tight in practice where offset cusps can occur. Such curvature is rare for stroke widths of tenths of a point, and #83 splits at corners.

Stroked label outlines (`Part::Outline`) go through the same code, so their bounds become tight too. The previous control-polygon box was larger, so no test can lose coverage, but some expected numbers in existing tests may tighten.

*Alternative:* keep the conservative control-polygon box. Rejected: it violates "Export bounds are tight" visibly for curved strokes (the spec scenario's control points reach 2 while the curve only reaches 1.5).

### 7. Follow-up issue for a provable bound

#99 tracks: a sound `deviation` for function curves from interval bounds on `γ⁗`, needing a fourth-order AD type over `Interval` in `expression`; a sound length bound; then removing the safety factor and the initial 4-way split. It links #80, #81 and #78.

## Risks / Trade-offs

- **[The sampled estimate misses a deviation between samples]** → safety factor 2, 4 initial pieces per knot interval, tolerance set to half the spec value. The deviation can only hide in features narrower than 1/8 of a piece that also escape halving. The follow-up issue replaces the estimate with a bound.
- **[The cap of 4096 pieces is reached]** → the remaining pieces may exceed the tolerance. This needs a curve with over 4000 features at 1 µm on paper, far beyond teaching figures. It is documented like `MAX_MESH_EDGES`.
- **[Signature change of `shape` / `curve_shape` touches every caller]** → there are only three callers (two kinds, `scene.rs`, `app.rs`). Tests that call `curve_shape` gain a tolerance argument.
- **[Quadrature and inversion cost in `trim`]** → only at export time and only for the two pieces cut at arrow heads.
- **[Tighter stroked-outline bounds change expected values in label export tests]** → expected values move toward the true bounds. Review any changed expected value for that direction.
