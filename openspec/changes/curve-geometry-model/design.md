# Design

## Context

See proposal.md for motivation. Today `impls/linux-rust` stores only `Segment` with endpoint `PointExpr::SegmentEnd`. Specs `geometry-ir` / `geometry-eval` gate the geometry set to segments. This change updates those contracts and refactors the Rust geom layer to a curve-shaped API without adding new primitive kinds yet.

Target implementation: Linux Rust `constraint_runtime` (+ tests); `sketch_ui` only if call sites break. Solver backends (LM / Ceres) unchanged.

## Goals / Non-Goals

**Goals:**

- Encode Curve + `CurveParam` { Normalized, Uniform } + endpoints-vs-periodic in shared specs.
- In Rust: trait(s) / API so `Segment` is a curve with `position_at` / `tangent_at` / `normal_at`; expand `PointExpr` / add `VecExpr`; evaluate OnCurve for segments (Normalized ≡ Uniform).
- Keep Model E: no mandatory Point entity; cross-geometry ops as future expressions/constraints.
- Update glossary docs lightly if terms (`Uniform`, vector expression, tangent/normal length) need alignment.

**Non-Goals:**

- Implement Circle/Spline/Face/Point entity or intersection evaluation.
- General arc-length inversion; physical \(s \in [0,L]\) domain.
- Unit-only frames; curvature as `normal_at` length.
- New constraint catalog entries (Incidence, Tangency).
- GUI construction tools for midpoints.

## Decisions

### 1. Specs extend `geometry-ir` / `geometry-eval` (no new capability id)

**Choice:** Delta existing capabilities.  
**Why:** Curve is the next layer of Model E geometry IR, not a parallel product area.  
**Alternative:** New `curve-geometry` capability — rejected as a near-duplicate of `geometry-ir`.

### 2. Rust: Curve trait + document `enum Geometry`

**Choice:** `Curve`-like trait with `position_at` / `tangent_at` / `normal_at` on normalized \(t\); separate `HasEndpoints` (or inherent endpoints on `Segment` only for now). DocumentStructure stores `Vec<Geometry>` with `Geometry::Segment(Segment)` today; further kinds become new enum variants. Shared ops live on `Geometry` (`parameter_count`, `connectivity`, `reindex_after_removal`, `curve()` → `&dyn Curve`). Callers use `curve()` / `match` for kind-specific work—not a growing set of `as_xxx` helpers. Convenience `Document::segments()` iterates segment geometries.  
**Why:** Makes the storage sum type explicit; scales by capability (Curve) and shared Geometry methods rather than per-variant downcasts.  
**Alternative:** Keep `Vec<Segment>` until circle exists — rejected once we want the structure visible now. Per-type `as_segment`/`as_circle` — rejected as unscalable.

### 2b. Tangent / normal length and orientation

**Choice:** `tangent_at(t)` = chart derivative \(\gamma'(t)\) (free vector; length = parametric speed). `normal_at(t)` = leftward planar rotate \(R_{90}(v)=(-v_y, v_x)\) of that tangent (same length). Not unit-length by default. Curvature is a separate future quantity—not the magnitude of `normal_at`. Degenerate only when \(\|\gamma'\|=0\) (e.g. zero-length segment).  
**Why:** Geometric perpendicular exists on line segments (κ = 0); solvers keep chart derivatives; matches Parallel/Orthogonal use of non-unit directions.  
**Alternative:** Unit tangent/normal only — rejected for this slice (loses speed; extra degeneracy rules). Curvature-weighted normal as `normal_at` — rejected (vanishes on segments).

### 3. Canonical domain is Normalized; Uniform is a parameter kind on expressions

**Choice:** `CurveParam::Normalized(t) | Uniform(u)`. Curve trait methods take normalized \(t\). `Uniform` resolves via per-kind logic (segment: identity).  
**Why:** Solver-friendly chart derivatives; construction midpoints use Uniform without a second trait hierarchy.  
**Alternative:** Only Uniform everywhere — rejected (cost/Jacobian for general curves; chart-native constraints).

### 4. Expression shapes

**Choice:** Extend `PointExpr` with `OnCurve { curve: SegmentId (for now), param: CurveParam }`; add `VecExpr` with `TangentAt` / `NormalAt` (and reuse segment direction where useful). Endpoint variants remain.  
**Why:** Congruence/FixPoint already consume `PointExpr`; midpoints become bindable without new constraints.  
**Alternative:** Separate `SampleId` table — heavier, defer.

### 5. Naming in code/docs

**Choice:** Spec language uses `Normalized` and `Uniform`. Method names `position_at` / `tangent_at` / `normal_at`. Avoid `ArcFraction` in public API (synonym of Uniform only if mentioned in design notes).  
**Why:** Matches agreed vocabulary from exploration.

### 6. Cross-geometry ops

**Choice:** Spec reserves intersection as expression/constraint; no Rust API in this change beyond a brief module comment or stub type if useful for compilation of reserved variants — prefer **not** stubbing unevaluable variants.  
**Why:** Avoid dead enum arms that panic at runtime.

### 7. Docs

**Choice:** Short glossary additions in `docs/constraint-mathematics.md` for normalized vs uniform, tangent/normal (chart derivative vs perpendicular, not curvature), and vector expressions.  
**Why:** Specs stay implementation-agnostic; math doc carries shared vocabulary.

## Risks / Trade-offs

- [SegmentId-only OnCurve] → Migration to a geometry id enum when circles arrive; keep `SegmentId` for this slice and note in tasks.  
- [Uniform ≡ Normalized only proven on segments] → Spec scenario already states they diverge for future Béziers; tests lock segment equality.  
- [Trait abstraction with one implementor] → Accept thin trait or inherent methods + trait for test doubles; prefer trait if it clarifies the contract without ceremony.  
- [BREAKING spec removal of segments-only] → Archive updates main specs; existing docs that say “v1 segments only” need a one-line refresh.

## Migration Plan

1. Land spec deltas + design + tasks (this change).  
2. Apply: geom refactor behind existing Segment documents; keep public endpoint API working.  
3. Add OnCurve/Uniform evaluation tests; run `constraint_runtime` + acceptance.  
4. On archive: merge deltas into main `geometry-ir` / `geometry-eval`.  
5. Rollback: revert change branch; no on-disk document format version yet beyond source.

## Open Questions

- Exact Rust naming: `Curve` trait vs `ParametricCurve`; settle during apply without changing specs.  
- Whether `sketch_ui` exposes midpoint handles in a follow-up (out of this task list).
