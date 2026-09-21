# Proposal

## Why

Scientific figures need more than line segments: constructions such as midpoints, equal arc splits, tangents, and (later) circles and function graphs all rest on a shared notion of a parameterized curve. Today the IR only admits segments and endpoint `PointExpr`s, so those constructions cannot be expressed without inventing ad-hoc geometry. Specifying a curve model now keeps Model E coherent before new primitives land.

## What Changes

- Introduce a shared **Curve** contract: every curve exposes position, tangent, and normal at a **normalized** chart parameter \(t \in [0,1]\), evaluating to point / free-vector expressions (not snapshots).
- Define length policy: `tangent_at` is the chart derivative \(\gamma'(t)\) (parametric speed as length); `normal_at` is a planar \(\pm 90^\circ\) rotate of that tangent (same length). Neither is required to be unit length. Curvature is **not** encoded as normal length (deferred as a separate quantity).
- Distinguish **capabilities**: curves with geometric endpoints (`HasEndpoints`) versus intrinsically periodic curves (e.g. full circle) that have a chart origin but no join endpoints.
- Address points on a curve via `CurveParam`: `Normalized(t)` (canonical) and `Uniform(u)` (constant-speed / equal arc-length fraction on \([0,1]\)). Physical arc-length domain \(s \in [0,L]\) is deferred.
- Extend **point expressions** beyond segment ends (e.g. on-curve samples); introduce **vector expressions** for tangents, normals, and directions used by constraints and construction.
- Reserve cross-geometry expression forms (intersection, and later tangency as constraints) as expression / catalog concerns—not methods on a single curve type.
- **BREAKING** (spec): lift the “segments only” geometry-set requirement so further primitives can be specified; this change’s implementation still proves the model primarily on segments.
- Document that a first-class `Point` geometry entity and `Face` remain out of scope; Rust traits / a document enum need no common OOP base class for a future `Point`.

## Capabilities

### New Capabilities

- *(none)* — curve contracts extend existing geometry capabilities rather than adding a parallel ontology.

### Modified Capabilities

- `geometry-ir`: Curve model (position/tangent/normal), parameterization (`Normalized` / `Uniform`), endpoint/periodic capabilities, expanded point/vector expressions; replace segments-only gate with a curve-centered geometry set for this slice.
- `geometry-eval`: Evaluation and analytical derivatives for curve position/tangent/normal and for new expression forms (at least segment-backed on-curve / uniform, which coincide for segments).

## Non-goals

- New primitive kinds in the runtime (circle, arc, spline, function graph)—only the contracts they will satisfy.
- First-class `Point` geometry entity; `Face` / filled regions; polycurve arc-length domain \(s \in [0,L]\).
- Unit-length-only tangent/normal APIs; curvature scalar or curvature-weighted normal as the primary `normal_at`.
- Full numerical arc-length inversion for general curves; intersection evaluation; new catalog constraints (Incidence, Tangency)—may be named as future hooks only.
- Illustration styling, LaTeX, export.

## Impact

- Shared specs: `geometry-ir`, `geometry-eval`; glossary alignment in `docs/constraint-mathematics.md` as needed.
- Linux Rust `constraint_runtime`: `geom` module (Curve-oriented API on `Segment`, `PointExpr` / `VecExpr` expansion), callers in document/constraints/UI that assume endpoint-only expressions.
- `sketch_ui` may keep drawing segments; optional use of on-curve expressions for handles is not required in this slice.
- Other implementations (none yet) consume the same specs with their own `design.md`.
