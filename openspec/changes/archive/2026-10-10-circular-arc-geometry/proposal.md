# Proposal

## Why

Scientific figures need circular arcs (Kreissegmente) under the same constraint/eval model as line segments. The heading/turn foundation is in place; the next step is to admit an open circular-arc curve kind with chart `(center, radius, start, sweep)` so constructions can join, drag, and solve against arc endpoints and on-curve samples.

## What Changes

- Admit a second curve geometry kind: **circular arc** (open arc of a circle), stored in the document kind sum alongside line segment.
- Chart parameters: center (`Point2`), radius (`Length`), start heading (`HeadingAngle`), sweep (`TurnAngle`). Position at normalized \(t\): \(\gamma(t) = C + R\,(\cos\theta(t),\sin\theta(t))\) with \(\theta(t)=\theta_0 + t\,\Delta\theta\).
- Expose curve + endpoints capabilities (start/end at \(t=0,1\)), length = \(R\,|\Delta\theta|\), and opaque heading/turn expressions for start and sweep.
- Tessellate arcs into the display Mesh as polylines (kind discriminant at Mesh only).
- Analytical on-curve / tangent / normal / length / endpoint Jacobians on the production path.

## Non-goals

- Full closed circle as a separate kind (periodic, no join endpoints).
- New angle constraints (equal heading, fixed sweep, …) beyond what existing catalog can already do via expressions once factories exist.
- UI tools for drawing arcs, styles, LaTeX, export.
- Variable-radius or elliptical arcs.

## Capabilities

### New Capabilities

_(none)_

### Modified Capabilities

- `geometry-ir`: Admit circular arc as a curve kind with endpoints; chart/init semantics for center/radius/start/sweep; heading/turn expression factories on that kind; geometry-set language no longer “segment only”.
- `geometry-eval`: Evaluation and analytical derivatives for arc curve/endpoint/length/heading/turn expressions.
- `display-mesh`: Mesh construction tessellates arcs (not only one edge per segment).

## Impact

- Shared specs: deltas under `geometry-ir`, `geometry-eval`, `display-mesh`.
- linux-rust: new `geometry_kinds/arc.rs`, catalog + `register_expr_kinds!`, Mesh match on `GeometryInst`, tests; docs/config kind list.
- Specs remain shared across implementations.
