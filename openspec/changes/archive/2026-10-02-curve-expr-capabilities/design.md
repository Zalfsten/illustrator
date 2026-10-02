# Design

## Context

See proposal.md. Specs already separate Curve vs Endpoints; linux-rust still exposes a flat `Segment` factory list including `direction_expr`.

## Goals / Non-Goals

**Goals:**

- `CurveExprs` + `EndpointExprs` capability factory traits.
- `Segment` implements both; closed curves later implement only `CurveExprs`.
- Remove `direction_expr` / `VecFormula::Direction`; Parallel/Orthogonal use `tangent_expr`.

**Non-Goals:** Circle kind; object-safe factory methods on `dyn Geometry`.

## Decisions

### 1. Capability traits (associated functions)

```text
trait CurveExprs {
  fn on_curve_expr(id, param) -> PointExpr
  fn tangent_expr(id, param) -> VecExpr
  fn normal_expr(id, param) -> VecExpr
}
trait EndpointExprs {
  fn start_expr(id) -> PointExpr
  fn end_expr(id) -> PointExpr
}
```

Call sites import the traits and use `Segment::…` via the trait methods (no inherent wrappers).

### 2. Replace direction with tangent

`Segment::tangent_expr(id, CurveParam::Normalized(0.0))` (any `t` — constant γ'). Call sites updated accordingly.

### 3. Shared formula catalogs + factory macros

Catalogs split with the traits: `CurvePointFormula` / `VecFormula` vs `EndpointPointFormula`.
Closed kinds never mention endpoint tags (no `circle_endpoint` constructor).
`impl_curve_exprs!($ty, $curve_wrap, $vec_wrap)` / `impl_endpoint_exprs!($ty, $endpoint_wrap)`
build exprs via kind-keyed constructors. Kinds own evaluate / param_indices / Jacobians.

## Risks / Trade-offs

- **BREAKING** API removal of `direction_expr` → update all call sites in-repo.
- Parallel/Orthogonal semantics unchanged (same residual on same vectors for segments).

## Migration Plan

Implement traits; delete Direction; retarget call sites; tests; docs.

## Open Questions

None.
