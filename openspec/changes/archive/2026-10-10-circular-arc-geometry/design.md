# Design

## Context

Heading/turn foundation landed (`HeadingAngle` / `TurnAngle`, Expr families). linux-rust stores `GeometryInst` with `Segment` only; Mesh hard-codes segment start/end for every geometry. Specs already allow additional curve kinds; this change admits **circular arc**.

## Goals / Non-Goals

**Goals:**

- Kind `Arc` with chart atoms `center: Point2`, `radius: Length`, `start: HeadingAngle`, `sweep: TurnAngle` (5 State DOFs).
- Curve + endpoints + length; `start_heading_expr` / `sweep_expr`; analytical Jacobians.
- Catalog + `register_expr_kinds!` registration; Mesh polyline tessellation.
- Tests for layout, on-curve/endpoints/length, FD checks for Jacobians, Mesh multi-edge.

**Non-Goals:** Closed full-circle kind; UI draw tool; new constraint kinds; ellipse.

## Decisions

### 1. Kind name `Arc`

Short, parallel to `Segment`. Specs say “circular arc”; rust type `Arc` in `geometry_kinds/arc.rs`.

### 2. Chart and parameterization

```text
Init = (center, radius, start, sweep)
theta(t) = start.radians() + t * sweep.radians()
gamma(t) = center + |R_sign| * (cos theta, sin theta)   # use signed R in State;
                                                        # position uses R as stored
                                                        # (signed radius flips 180°)
```

**Convention:** Heading null ray = world \(+x\); increasing heading = CCW in y-up. Sweep sign selects orientation; length = `|R| * |sweep|`.

Normalized \(t \in [0,1]\) maps the full sweep; for constant \(|R|\), Uniform ≡ Normalized.

Endpoints: kind-efficient start/end formulas that match on-curve at \(t=0\) / \(t=1\) (no on-curve-at-endpoint default factory).

### 3. Expression registration

Extend `register_expr_kinds!` (or sibling) so `Arc` contributes:

- curve / endpoint / vec / length (like Segment)
- heading + turn keyed variants (replace empty uninhabited enums)

Factories: `CurveExprs` / `EndpointExprs` via existing blankets; add thin arc-specific helpers `start_heading_expr` / `sweep_expr` (associated fns on `Arc`, analogous to length).

### 4. Mesh tessellation

Match on `GeometryInst`:

- `Segment` → one edge (unchanged)
- `Arc` → polyline with \(n = \mathrm{clamp}(\lceil |\Delta\theta| / (\pi/8) \rceil, 2, 64)\) chords (design default; tunable constant in code)

Endpoint hit classification uses arc start/end expressions (first/last polyline vertices).

### 5. PackAtoms

`(Point2, Length, HeadingAngle, TurnAngle)` already covered by the 4-tuple `PackAtoms` impl (`PARAMETER_COUNT = 5`).

## Risks / Trade-offs

- **Signed radius:** Unusual for CAD but matches Length DOF policy; document in kind docs. Length expression uses magnitudes.
- **Empty angle catalogs today:** First consumer fills `HeadingAngleFormulaKeyed` / `TurnAngleFormulaKeyed` — replace uninhabited enums with real variants.
- **Mesh density:** Fixed angular step is simple; adaptive curvature later if needed.
- **sketch_ui:** May still assume segment-only handles; out of scope unless smoke breaks — note in tasks if a minimal match is required.

## Alternatives considered

- **start/end headings instead of start/sweep:** Rejected; exploration preferred start+sweep for Heading/Turn clarity.
- **Center + two endpoints (chord):** Overconstrained / different DOF story; not this chart.
