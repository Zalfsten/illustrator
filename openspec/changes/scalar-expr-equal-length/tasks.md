# Tasks

## 1. Scalar expression infra

- [ ] 1.1 Add `ScalarJacobian` (1×n) in `geom` and opaque `ScalarExpr` in `geometries/expr` (kind-keyed; segment wrap)
- [ ] 1.2 Add shared scalar formula tag for curve length in capabilities; wire `CurveExprs::length_expr`

## 2. Segment length + EqualLength

- [ ] 2.1 Implement segment length evaluate / param_indices / Jacobian; extend curve macro
- [ ] 2.2 Add `Constraint::EqualLength`; residual arity 1; connectivity + Jacobian; orphan handling
- [ ] 2.3 Unit tests (length value, Jacobian vs FD, EqualLength FD); acceptance or interact smoke for two equal-length segments
- [ ] 2.4 Update `docs/constraint-mathematics.md` glossary for scalar expressions / EqualLength; `cargo test -p constraint_runtime`
