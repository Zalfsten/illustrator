# Proposal: Kind formulas may read kind attributes

## Why

Today a kind formula is a pure function of the geometry's State block. A label's text attributes never reach a formula, because text does not move geometry. Function curves (#74, change D #81) are the first kind whose geometry depends on a non-State attribute: the expression, its x range and its scale decide where the curve lies, but they are not degrees of freedom. Before that kind can exist, formulas need access to their geometry's attributes. This change is change B of the function-curve split (#79). It changes no behaviour and can land independently of A (#78) and C (#80).

This advances the product goal's figure types (function graphs for teaching figures) by preparing the geometry contract. It adds no figure type itself.

## What Changes

- Every kind formula family receives the geometry's own kind attributes next to its State block. This covers values and Jacobians of point, vector, length, heading and turn, plus exact shape, curvature and canonicalization. Attributes are passed explicitly, not looked up through a side channel such as a global table keyed by id.
- Each kind declares the type of its attributes. Segment and arc have none and ignore them. The label's text attributes become visible to its formulas, and its formulas ignore them.
- Attributes are not degrees of freedom. They add no Jacobian columns, and a formula's connectivity (which State entries it reads) still depends only on the kind and the formula.
- Attribute equality, which undo relies on, stays on the stored source data. Anything a later kind compiles from its attributes is a derived cache.
- No observable behaviour changes. All existing tests pass with only mechanical updates to direct formula calls.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `geometry-ir`: a kind formula may read its own geometry's non-parameter attributes as well as its State block window. Attributes are not State and contribute no derivatives.

## Non-goals

- The function curve kind itself (#81), its attributes and any compiled-expression cache. This change only opens the contract.
- The expression language (#78) and Bézier shape pieces (#80).
- Formulas reading another geometry's attributes or State. Cross-geometry relations stay expressions and constraints.
- Attributes that vary during a solve. Attributes stay fixed for the duration of a solve; changing them is a structural edit.
- Changes to label typesetting, export, commands or the sketch file.

The specs stay shared across implementations. How attributes are typed and passed is a design decision of each implementation.

## Impact

- **impls/rust `constraint_runtime`:**
  - `geometry/formula.rs` (the `Geometry` trait gains an attribute type and an attribute argument on the formula families; the `Formula` dispatch passes it through).
  - `geometry_kinds/catalog.rs` (typed access to a geometry's attributes).
  - `geometry_kinds/{segment,arc,label}.rs` (new signatures, attributes ignored).
  - `expr.rs` and `document.rs` (pass the attributes at evaluation, Jacobian, shape, curvature and canonicalization).
  - Direct formula calls in the kinds' unit tests.
- No change to other crates, the file format, commands, export or the solver.
