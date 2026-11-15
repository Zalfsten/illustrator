# Tasks

Implementation: impls/rust (see design.md). Run tests from `impls/rust`. New unit tests go in `src/tests_<module>.rs` next to the module.

## 1. Curve parameter and capability (geometry-ir, design decisions 1 and 3)

- [x] 1.1 Add `CurveParam::Abscissa(f64)` (`chart_t` is `unreachable!` for it, with a doc comment), `Caps.abscissa` with `with_abscissa()`, and the keyword `abscissa` in `geometry_caps!`. Declare it for `Function`. Verify with tests:
  - `tests_catalog.rs`: only the function kind reports abscissa;
  - `tests_geom.rs`: `chart_t` still serves `Normalized` / `Uniform`.

  `cargo test -p constraint_runtime` passes.

## 2. Function curve abscissa evaluation (geometry-ir, design decision 2)

- [x] 2.1 In `function.rs`:
  - add `FunctionAttrs::abscissa_jet(x)` and `abscissa_valid(x)`, evaluated without clamping;
  - add the `Abscissa` arms for point, tangent and normal, and their Jacobians.

  Verify in `tests_function.rs`:
  - the spec scenarios "Point and tangent at x = 1", "Same direction as the normalized tangent", "Abscissa stays at its x when the range changes", "Point beyond the drawn range";
  - Jacobians of abscissa points and vectors against finite differences;
  - `abscissa_valid` is false for `sqrt(x)` at 0, `log(x)` at −1 and `exp(x)` at 1000.

## 3. Document factories and addresses (geometry-ir, object-naming, design decisions 3 and 4)

- [x] 3.1 Gate the `Document` curve factories for `Abscissa`: abscissa capability plus `abscissa_valid` (new `Document::abscissa_valid(id, x)`). Verify in `tests_document.rs` / `tests_function.rs`:
  - "No abscissa on a segment" (and arc, label);
  - "Vertical tangent" and "Undefined function" give `None`;
  - a valid abscissa gives the expression.
- [x] 3.2 Extend `address.rs`:
  - the `x=` argument form;
  - resolution with `MissingCapability { "abscissa" }` and `InvalidAbscissa { name, x }`;
  - printing `at(x=…)` / `tangent(x=…)` / `normal(x=…)`;
  - updated module docs.

  Verify in `tests_address.rs`:
  - the object-naming scenarios "Tangent at x", "No keyword means normalized", "Round trip of an abscissa address", "Abscissa on a segment", "Invalid abscissa";
  - `fn1.at(u=1)` and `fn1.at(x=)` are malformed;
  - `fn1.at(x=inf)` is rejected.

## 4. Edit rule (sketch-commands, design decision 5)

- [x] 4.1 Add `ConstraintKind::curve_params` (default empty) and `Expr::curve_param()`; implement it for Congruence, FixPoint, VectorsParallel and VectorsOrthogonal. Add `Document::invalid_abscissa_users(id)`. Verify in `tests_constraint.rs` / `tests_document.rs`:
  - each kind reports its operands' parameters;
  - `invalid_abscissa_users` lists exactly the constraints with an invalid abscissa on that geometry.
- [x] 4.2 In the function `set`, fail with `InvalidExpression` naming the affected constraints before solving. Verify in `tests_session.rs` with the spec scenarios "Undefined after the change", "Still valid after the change" and "Range change is never checked". The drawing must be unchanged after the failure.

## 5. End-to-end and documentation

- [x] 5.1 Extend `sketch_session/tests/function_curve.rs`: a tangent segment parallel to `fn1.tangent(x=1)` and joined to `fn1.at(x=1)` survives:
  - a range change: the joint stays at x = 1, checked by the joint's x in the curve's frame;
  - save → load → save, byte-identical, with the address text `fn1.at(x=1)` in the file;
  - a drag.

  Verify that the test passes.
- [x] 5.2 Update `openspec/config.yaml`: address grammar, capability list, `CurveParam::Abscissa`, `curve_params` / `invalid_abscissa_users`. Verify that `openspec validate abscissa-addressing --strict` passes.
- [x] 5.3 Run `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings` and `scripts/check-wasm.sh` from `impls/rust`. Verify that all three succeed.

## Workflow follow-up

- Archive the change after review; sync the `geometry-ir`, `object-naming` and `sketch-commands` deltas.
- Check off #82 in #98 and #74; #84 (change G) can then use abscissa picks.
