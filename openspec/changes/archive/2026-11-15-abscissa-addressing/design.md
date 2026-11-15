# Design: Abscissa addressing on function curves

## Context

See proposal.md for motivation and the spec deltas (`geometry-ir`, `object-naming`, `sketch-commands`) for the contract. Target implementation: impls/rust, crates `constraint_runtime` and `sketch_session`. Planned on top of #81 (function curve kind, PR #102).

- **`CurveParam`** (`geom.rs`) has `Normalized(t)` and `Uniform(t)`. Kinds read it through `CurveParam::chart_t()`, which today covers every variant.
- **Formula tags** carry the parameter: `PointFormula::OnCurve { param }`, `VecFormula::TangentAt { param }` and `NormalAt { param }`.
- **Identity-based factories** (`Document::on_curve_expr / tangent_expr / normal_expr`) check only the curve capability through `capability_expr`. `Caps` has curve, endpoints, line, circular and anchor; `geometry_caps!` maps keywords to `Caps` and the factory traits.
- **Addresses** (`address.rs`):
  - `parse_suffix` accepts `ident` or `ident(<f64>)`, and `normalized()` checks [0, 1];
  - `AddressError` has Malformed, UnknownName, NotAGeometry, ParamOutOfRange and MissingCapability;
  - printing returns `None` for `Uniform`.
- **Function curves** (`function.rs`): `FunctionAttrs::jet(x)` clamps `x` to the range before evaluating (thread-local scratch), so the continuation is not reachable yet.
- **Constraints** report `referenced_geometries()` but not their operand expressions or curve parameters.
- **Function `set`** (`session.rs`): applies the patch, then solves (`CHANGES_GEOMETRY`); `apply` restores the snapshot on any error.

## Goals / Non-Goals

**Goals:**

- `fn1.at(x=1)` / `tangent(x=1)` / `normal(x=1)` resolve, evaluate with exact analytical Jacobians, print back, and work in every constraint, in files and in the solver.
- An abscissa never becomes invalid behind the user's back: it is checked when it is built and whenever the expression changes.

**Non-Goals:** abscissa on other kinds; a `u=` form; abscissa as a degree of freedom (#3).

## Decisions

### 1. `CurveParam::Abscissa(f64)` in the existing formula tags

- The parameter travels in the same `OnCurve` / `TangentAt` / `NormalAt` tags. No new formula families are needed, so connectivity, assembly, `VecJet` and the constraint kinds stay unchanged.
- `chart_t()` keeps serving `Normalized` / `Uniform` and is `unreachable!` for `Abscissa`. Only kinds with the abscissa capability can receive one (decision 3), and the function kind matches `Abscissa` before calling `chart_t()`.

*Alternative:* separate formula variants such as `PointFormula::AtAbscissa`. Rejected: every kind's `match` would grow a dead arm, and the spec stores the parameter with the operand anyway.

### 2. Function kind: closed-form evaluation without clamping

- **New on `FunctionAttrs`:**
  - `abscissa_jet(x) -> Result<Jet, _>` evaluates at the unclamped `x`;
  - `abscissa_valid(x)` is true when the evaluation succeeds and value and `f′` are finite.
- **Formulas:**
  - point `O + R(θ)·(su·x, sv·f(x))`;
  - tangent `R(θ)·(su, sv·f′(x))`;
  - normal: its left normal.
- **Jacobians:** the same shapes as for `Normalized` (point: `[I | R′(θ)p]`; vectors: θ column only). The existing code paths only need the local point and vector for the abscissa case.
- Evaluation of an invalid `x` cannot occur, because every abscissa expression is checked when it is built (decision 3) and after every expression change (decision 5). It `expect`s with that reason.

### 3. The abscissa capability and the validity check at build time

- `Caps.abscissa` and the keyword `abscissa` in `geometry_caps!`; `Function` declares it.
  - It brings no factory trait of its own: it gates the existing curve factories for the `Abscissa` parameter.
- `Document::on_curve_expr / tangent_expr / normal_expr` with `CurveParam::Abscissa(x)` return `None` unless the geometry has the abscissa capability **and** the abscissa is valid there. `Document::abscissa_valid(id, x)` answers the latter.
- The static per-kind factories (`Function::on_curve_expr(id, param)`) stay unchecked, as for every capability today. They are for code that already knows the kind.

*Alternative:* a separate `AbscissaExprs` factory trait. Rejected: it would duplicate three factories, and the parameter value already tells which form is meant.

### 4. Address grammar, errors and printing

- **Grammar:** `parse_suffix` returns an argument enum `Arg::Normalized(t) | Arg::Abscissa(x)`. `ident(x=<f64>)` is the abscissa form, `ident(<f64>)` the normalized form; any other keyword is `Malformed`.
- **Resolution order** for an abscissa:
  1. the name;
  2. the abscissa capability (`MissingCapability { capability: "abscissa" }`);
  3. a finite `x`, then validity (`AddressError::InvalidAbscissa { name, x }`, "f is not defined or its slope is not finite at x = …").
- **Printing:** `at(x={x})`, `tangent(x={x})`, `normal(x={x})`. Rust's `f64` `Display` is the shortest round-trip form, so resolving the printed address yields an equal expression.

### 5. Edit rule: constraints report their operands' curve parameters

- **New:** `ConstraintKind::curve_params(&self) -> Vec<(GeometryId, CurveParam)>`.
  - The default is empty, kept for kinds without point/vector operands (EqualLength).
  - Congruence, FixPoint, VectorsParallel and VectorsOrthogonal implement it from their operands through a helper `Expr::curve_param()`, which reads the formula tag.
- **Document query:** `Document::invalid_abscissa_users(id) -> Vec<ConstraintId>` lists the constraints whose abscissa on geometry `id` is invalid under its **current** attributes.
- **Session:** in the function `set`, after applying the attributes and before solving, a non-empty list fails with `CommandError::InvalidExpression`, naming the constraints ("the abscissa addresses of join1, par2 would be invalid"). `apply` then restores the snapshot.
  - The check runs for every function `set`. A range or scale change cannot fail it, because validity depends only on the expression, so this is observably the spec's "not checked".

*Alternative:* scan the constraints' printed addresses for `(x=`. Rejected as text-based and fragile.

## Risks / Trade-offs

- **[`chart_t` panics if an abscissa reaches a kind without the capability]** → only the unchecked static factories can produce that. A catalog test asserts that abscissa factories through `Document` return `None` for every kind except `Function`.
- **[Continuation far outside the range is numerically extreme]** (`exp(x)` at `x = 1000`) → validity requires finite value and slope, so overflow is rejected like a domain error.
- **[Files with abscissa addresses opened by an older version-4 reader]** → that reader rejects the address as malformed with the command index. No format bump is needed, because the grammar extension only adds forms that were invalid before.
