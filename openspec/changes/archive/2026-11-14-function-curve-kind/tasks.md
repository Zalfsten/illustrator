# Tasks

Implementation: impls/rust (see design.md). Run tests from `impls/rust`, e.g. `cargo test -p constraint_runtime`. New unit tests go in `src/tests_<module>.rs` next to the module.

## 1. Function attributes and arc-length table (design decisions 1–3)

- [x] 1.1 Add the `expression` dependency to `constraint_runtime` (check that `scripts/check-wasm.sh` still builds). Add `geometry_kinds/function.rs` with:
  - `FunctionAttrs::new(source, range, scale) -> Result<_, FunctionAttrsError>`: parse with variable `x`; check range (finite, x0 < x1) and scale (finite, > 0); `check_domain`; build the table;
  - accessors and `PartialEq` on source, range and scale only;
  - the thread-local `eval_in` scratch.

  Verify with tests in `geometry_kinds/tests_function.rs`:
  - each error variant (`2x` parse error with position; `1/x` on [−1, 1] and `sqrt(x)` on [0, 1] unproven; reversed range; zero scale);
  - equality ignores the table;
  - a clone shares the table.
- [x] 1.2 Implement the arc-length table and its inverse per design decision 2 (adaptive Gauss–Legendre intervals, Hermite `x(s)`, refinement to 1e-11·L, cap → `TooComplex`). Verify in `tests_function.rs`:
  - `sin(x)` on [0, 2π] has length 7.6403955780 within 1e-9;
  - `x^2` on [0, 1] with scales (2, 3) matches the closed-form length within 1e-9;
  - the arc length from x0 to `x(t·L)` equals `t·L` within 1e-9·L at 50 values of t;
  - the build for `sin(x)` on [0, 100π] stays under 16 000 intervals (planned: 2000; measured 11 696, about 230 per period at 1e-11·L with cubic Hermite inverses).

## 2. The function kind (geometry-ir, design decisions 2, 4–6)

- [x] 2.1 Implement `Geometry` for `Function`:
  - chart block `(O: Point2, θ: HeadingAngle)`, prefix `fn`, `geometry_caps!(Function: curve, endpoints, anchor, line)`;
  - point / vector / length formulas with the Jacobians and connectivity of decision 4;
  - `curvature`, `piece_bound` (decision 5) and `shape` via `hermite_shape`.

  Add `GeometryKind::Function`, `KindAttrs::Function`, the `with_kind!` arm and `ALL_KINDS`, and update every `match` on `KindAttrs`. Verify with tests in `tests_function.rs`:
  - every Jacobian against finite differences;
  - the spec scenarios "Sine over one period", "The frame rotates the graph", "Scales stretch the graph", "Midpoint of a sine period", "Arc length up to a sample", "Origin and direction", "No radius";
  - `Normalized` = `Uniform`.
- [x] 2.2 Document-level attribute forwarding (review of #87): at fixed State, changing the expression, range or scale of a function curve through `set_geometry_attrs` changes `evaluate`, the Jacobian (`accumulate_jacobian` and `VecExpr::jet`), `curve_shape` and `curve_curvature`; the committed block stays unchanged (canonical is the identity). Verify with tests in `tests_document.rs` (or `tests_function.rs` if only public API is needed).
- [x] 2.3 Piece bounds and shape:
  - for `sin(x)` and `exp(x)` on [0, 5], the 64 piece lengths sum to at least the length, and every piece's curvature bound is ≥ `curvature(t)` at its ends and midpoint;
  - the shape stays within the tolerance of the chart (dense two-sided check as in `tests_cubic.rs`) and its length agrees within δ;
  - the Mesh of `exp(x)` on [0, 5] stays under `MAX_MESH_EDGES` and within the chord tolerance.

  Extend `tests_catalog.rs` if the existing curve-kind checks need a function-specific case. Verify that `cargo test -p constraint_runtime` passes.
- [x] 2.4 Solver integration: extend `tests/alloc_free.rs` with a drawing that has a function curve, joined at its start and end and parallel through its direction; after warm-up, residual and Jacobian evaluation allocate nothing. Add a test with EqualLength between a function curve's length (zero-column expression) and a segment's length that solves to the segment having the curve's length. Verify that both pass.

## 3. Commands and errors (sketch-commands, design decision 7)

- [x] 3.1 Add `kinds/function.rs` in `sketch_session`: `FunctionParams` / `FunctionSet` with serde and JSON Schema, `GeometryCommand` and `AttributeCommand`, registered in the catalog. Add `CommandError::InvalidExpression { message }` with class `invalid expression`, its wire/JSON form and messages (parse position, unproven sub-range, "exceeded its budget"). Verify with tests in `tests_kinds.rs` / `tests_json.rs`:
  - the spec scenarios "Function curve in interchange form", "Reversed range is rejected", "Zero scale is rejected", "Parse error with position", "Unproven domain with sub-range";
  - round trip of a function `create` and `set` through JSON;
  - `tests/schema.rs` still validates.
- [x] 3.2 Make `set` solve for function curves: `check_not_frozen` first, then `structural_scoped`; `Unsatisfiable` and `Frozen` as specified; label `set` unchanged. Verify with tests in `tests_session.rs`:
  - "New expression";
  - "Rejected expression keeps the curve";
  - "Joined end follows a longer range";
  - "Frozen component";
  - an unsatisfiable range change restores the drawing;
  - undo of a solving `set` restores attributes and values.
- [x] 3.3 Declarative export writes `expression`, `range` and `scale` for every function curve. Verify with a `tests_session.rs` test: export, re-execute in an empty session, export again → identical names, attributes and values within solve tolerance.

## 4. Sketch file version 4 (sketch-file, design decision 8)

- [x] 4.1 Raise `VERSION` to 4, migrate version 3 unchanged, and reject a `function` create in versions 1–3. Verify with tests in `tests_file.rs`:
  - "Version 3 file" (loads, saves as 4);
  - "Function curve in version 3 is rejected";
  - "Newer version" with version 5;
  - save → load → save is byte-identical for a drawing with a function curve;
  - update existing tests that expect version 3 on save.

## 5. End-to-end and documentation

- [x] 5.1 The "Done when" of #81 as an integration test in `sketch_session` (`tests/function_curve.rs`):
  - create `fn1 = sin(x)` on [0, 2π];
  - join `fn1.origin`, `fn1.start` and `fn1.end` to segments; make a segment parallel through `fn1.direction`;
  - solve, drag a joined segment end, save, load;
  - export SVG and PDF through `sketch_export` and check that the curve is written as cubic pieces within the export bounds.

  Verify that the test passes.
- [x] 5.2 Update `openspec/config.yaml`:
  - the kind list (function: prefix `fn`, attributes and table, thread-local scratch, caps);
  - the file version 4;
  - the `constraint_runtime → expression` dependency.

  Verify that `openspec validate function-curve-kind --strict` passes.
- [x] 5.3 Run `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings` and `scripts/check-wasm.sh` from `impls/rust`. Verify that all three succeed.

## Workflow follow-up

- Archive the change after review; sync the `geometry-ir`, `sketch-commands` and `sketch-file` deltas.
- Check off #81 in #98 and #74. Changes E (#82), F (#83) and G (#84) become unblocked.

## Review follow-ups (PR #102)

- [x] R.1 Rebased onto main (#100 and #101 merged); `Function::shape` takes `.pieces` of the new `HermiteShape`. Reporting `capped` to the export is #104.
- [x] R.2 Mesh: `place_samples` uses an adaptive grid (cells halved while they need more than one edge and halving lowers their demand, or while their bound is unbounded); `piece_bound` margins and quadrature are relative to the piece. A bell curve `1/((x-1)^2 + 0.01)` went from 1024 unchecked edges 0.215 off the graph to 95 edges within the tolerance. Tests `mesh_of_peaked_curves_stays_under_the_cap_and_within_tolerance`, `sharp_bend_in_a_long_cell_does_not_inflate_the_demand`.
- [x] R.3 Arc table: quadrature stops at its rounding noise (`sin(100*x)` on [0, 1000] ran for minutes), a budget of 15 million speed evaluations, incremental rounds (passed intervals are not re-checked) and a projected early rejection. Test `oscillating_graph_is_rejected_quickly`.
- [x] R.4 A NaN curve parameter evaluates at the range start instead of panicking; test `nan_parameter_does_not_panic`.
- [x] R.5 geometry-ir: attribute changes do not themselves move O and θ; the solve after a geometry-changing `set` may.
- [x] R.6 Parse errors report character positions; `FunctionAttrs::default()` is built once and cloned.
