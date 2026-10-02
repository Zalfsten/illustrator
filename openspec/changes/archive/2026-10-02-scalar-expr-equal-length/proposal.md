# Proposal

## Why

Constraints today only consume opaque point and free-vector expressions. Equal curve lengths (and later radius, fixed length, distances) need a first-class opaque **scalar** expression with analytical derivatives. Without it, length equality would hard-wire kinds or invent one-off catalog special cases.

## What Changes

- Add opaque **scalar expressions** (evaluate to a real number; connectivity + 1×n analytical Jacobian) parallel to point/vector expressions (Model B).
- Extend the curve capability with a **curve length** scalar expression factory (`length_expr`), implemented per kind (segment: Euclidean end−start length).
- Add structural catalog constraint **EqualLength** between two opaque scalar expressions (one residual: \(a - b\)).
- Out of scope: public compose algebra (`norm`, point−point), `FixLength`, circle kind, general numerical arc-length for arbitrary charts.

## Capabilities

### New Capabilities

None (scalar expressions extend existing geometry expression contracts; EqualLength extends the constraint catalog).

### Modified Capabilities

- `geometry-ir`: Opaque scalar expressions; curve length via capability-scoped factory on curves (not endpoints).
- `geometry-eval`: Evaluate scalar expressions and their analytical Jacobians; segment length path.
- `constraint-catalog`: EqualLength structural constraint over two scalar expressions; structural-invariants list includes EqualLength.

## Impact

- linux-rust `constraint_runtime`: `ScalarExpr` + scalar Jacobian carrier; `CurveExprs::length_expr`; segment length formula; `Constraint::EqualLength`; assembly/tests/docs.
- sketch_ui: optional later tool; not required in this change.
- Spec sync on archive for the three modified capabilities.
