# Proposal: Abscissa addressing on function curves

## Why

The function curve (#81) uses an arc-length chart, so the normalized `t` of a given `x` shifts whenever its expression, range or scale changes. A tangent constrained at `fn1.tangent(0.4)` therefore moves silently along the graph on every edit. The product-goal figure needs "the tangent at x = 1" to stay at x = 1. This is **change E** of #74 (#82); the app UI (change G, #84) uses it to pick points and tangents by `x`.

## What Changes

- **A third curve parameter, `Abscissa(x)`**, next to `Normalized` and `Uniform`. It is stored with each operand, inside the point or vector expression, not with the constraint.
  - Point: `O + R(θ)·(su·x, sv·f(x))`, in closed form.
  - Tangent: the derivative with respect to `x`, `R(θ)·(su, sv·f′(x))`. It has the same direction as the normalized tangent at that point, but a different length.
  - Normal: the tangent turned a quarter to the left, as for every curve.
- **An abscissa capability**, reported only by function curves. Building an abscissa expression on any other kind fails with a missing capability.
- **Address syntax:** `fn1.at(x=1)`, `fn1.tangent(x=1)`, `fn1.normal(x=1)`.
  - Without a keyword the normalized parameter applies (`fn1.at(0.4)`), so existing addresses and files stay valid.
  - Expressions with an abscissa parameter print in that form.
- **Continuation:** an abscissa may lie outside the drawn range `[x0, x1]`, on the undrawn continuation of `f`. A range change never invalidates an abscissa address.
- **Validity:** at the abscissa, `f` must be defined and `f′` finite.
  - Otherwise address resolution fails with its own reason.
  - At a vertical tangent the normalized parameter has to be used instead.
- **Edit rule:** changing a function curve's expression so that an abscissa used by a constraint becomes invalid is rejected with `invalid expression`, naming the affected constraints. Range and scale changes are not affected.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `geometry-ir`:
  - the curve parameter kinds gain `Abscissa`;
  - the capability set gains abscissa, reported only by function curves;
  - abscissa evaluation, continuation and validity.
- `object-naming`:
  - the address forms `at(x=…)`, `tangent(x=…)`, `normal(x=…)`;
  - the new resolution failure for an invalid abscissa;
  - printing abscissa expressions as addresses.
- `sketch-commands`: an expression change that invalidates an abscissa address in use is rejected and names the constraints.

## Non-goals

- **Abscissa addresses for other kinds** (segments, arcs). Only user-entered expression curves get the capability.
- **A `Uniform` address form** (`u=`). The keyword syntax leaves room for it.
- **Singularities** (change F, #83), the **app UI** for picking by `x` (change G, #84) and **branches across poles** (#76).
- **Moving an abscissa along the curve:** the abscissa is a fixed value of the operand, not a degree of freedom (#3 covers free curve parameters).

The specs stay shared across implementations. How abscissa expressions are stored and evaluated, and how constraints report their operand parameters, are design decisions of each implementation.

## Impact

- **impls/rust `constraint_runtime`:**
  - `geom.rs`: `CurveParam::Abscissa`; `chart_t` no longer covers every variant;
  - `geometry/capabilities.rs`: the abscissa capability in `Caps` and `geometry_caps!`;
  - `geometry_kinds/function.rs`: abscissa formulas and Jacobians, evaluation without clamping to the range;
  - `document.rs`: the capability check for abscissa parameters in the identity-based factories, and the validity check;
  - `address.rs`: the grammar, the new error and printing;
  - `constraint.rs` / `constraint_kinds/*`: each constraint reports the curve parameters of its operands.
- **impls/rust `sketch_session`:** the function `set` checks abscissa addresses in use before changing the expression.
- **Docs:** `openspec/config.yaml` (address grammar, capability list, curve parameters).
- No change to the file format: addresses are text, so files with abscissa addresses are ordinary version 4 files.
