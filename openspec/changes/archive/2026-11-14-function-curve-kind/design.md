# Design: The function curve kind (smooth cases)

## Context

See proposal.md for motivation and the spec deltas (`geometry-ir`, `sketch-commands`, `sketch-file`) for the contract. Target implementation: impls/rust, crates `constraint_runtime`, `sketch_session`, plus tests in `sketch_export` / `sketch_ui`. Planned on top of #80 (cubic shape pieces, PR #100) and #62 (split specs, PR #101).

- **Kind infrastructure:**
  - The closed kind catalog (`GeometryKind`, `with_kind!`, `ALL_KINDS`) and `KindAttrs { None, Label }`.
  - Kinds implement `Geometry` (formulas, Jacobians, connectivity, `shape(c, a, tolerance)`, `curvature`, `piece_bound`, `canonical`) and declare capabilities with `geometry_caps!`. Label is the closest template (origin + heading, attributes).
  - Formulas receive only the chart values and `&Attrs`. `Document::set_geometry_attrs` canonicalizes the block after an attribute change (#79).
- **Expressions** (`expression` crate, #78):
  - `Expression::parse(source, "x")`, `eval_in(x, &mut scratch) -> Jet { value, d1, d2 }` (allocation-free with a reused scratch), `eval_with::<N: Number>` (allocates), `check_domain(x0, x1)` (proves f, f′, f″ finite; `NotProven { range }` or `TooComplex { range }`).
  - `constraint_runtime` does not depend on `expression` yet.
- **Shapes and Mesh:**
  - `cubic::hermite_shape(point, tangent, knots, tolerance, deviation)` and `sampled_deviation` (#80).
  - The Mesh samples non-uniform curves from `piece_bound` (length and curvature upper bounds, #86).
  - Export reads `curve_shape` generically.
- **Session:**
  - `GeometryCommand` (`TAG`, `Params`, `init`, `attrs`, `params`) and `AttributeCommand` (`apply`) per kind.
  - `Command::Set` never solves today.
  - Constraint commands use `check_not_frozen` and `interact.structural_scoped`, and `apply` restores the snapshot on error.
  - The file is version 3; `migrate` and per-version field checks exist (version 2 rejects style fields).

## Goals / Non-Goals

**Goals:**

- **A function curve that behaves like any curve:** join, parallel, drag, Mesh, export, save/load.
- **Fast solving:** a solve moves only `O`, `θ`; no tabulation and no allocation in the residual/Jacobian path.
- **A chart that lies exactly on the graph** and is the arc-length fraction within 1e-9 L (spec), with consistent analytical derivatives.

**Non-Goals:** abscissa addressing (#82), singularities (#83), app tool/panel (#84), a provable shape bound (#99).

## Decisions

### 1. Attributes carry the derived table

```rust
pub struct FunctionAttrs {
    expression: Expression,   // source kept verbatim, variable "x"
    range: (f64, f64),        // x0 < x1
    scale: (f64, f64),        // su, sv > 0
    table: Arc<ArcTable>,     // derived; rebuilt by every constructor
}
impl FunctionAttrs {
    pub fn new(source: &str, range: (f64, f64), scale: (f64, f64)) -> Result<Self, FunctionAttrsError>;
}
```

- `FunctionAttrsError` has the variants `Parse(ParseError)`, `Domain(DomainCheckError)`, `Range`, `Scale` and `TooComplex`.
- `PartialEq` compares source, range and scale only. `Clone` shares the table through `Arc`.
- The constructor is the only way to change attributes, so the table can never be stale.
- `KindAttrs::Function(FunctionAttrs)` is added.
- `default_attrs()` is `x` on [0, 1] with scale (1, 1): a valid diagonal, as `Geometry` requires an infallible default.

*Alternative:* tabulate lazily on first use (interior mutability, `Sync` issues), or recompute in every formula call (far too slow for the solver). Both rejected.

### 2. Arc-length table and chart

- **Table:** nodes `x_i` on [x0, x1] with cumulative arc length `s_i` of the scaled graph and speed `σ_i = √(su² + sv²·f′(x_i)²)` (= ds/dx). Interval lengths come from adaptive 8-point Gauss–Legendre to 1e-14 relative.
- **Inverse `x(s)`:** on [s_i, s_{i+1}], cubic Hermite with end slopes `1/σ_i`, `1/σ_{i+1}`, clamped to [x_i, x_{i+1}].
- **Building:** start with 16 equal intervals, halve every interval whose Hermite inverse misses the exact arc length at its quarter points by more than 1e-11·L, cap 65 536 intervals. Reaching the cap is `TooComplex`, which the command layer reports as `invalid expression` ("exceeded its budget").
- **Chart at `t`:** `s = t·L` → binary search for the interval → `x = H(s)` → **exact** `f(x), f′(x)`.
  - Point: `γ(t) = O + R(θ)·(su·x, sv·f(x))`.
  - Tangent: `γ′(t) = R(θ)·(su, sv·f′(x))·L·H′(s)`, the derivative of this very chart.

  Points lie exactly on the graph; `t` matches the arc-length fraction within the table accuracy (≪ 1e-9 L).
- **`Start` / `End`:** use `x0` / `x1` directly, without the table.
- **`Length`:** `L`. Its formula depends on no State entry (empty connectivity). EqualLength with a function curve then fixes the other length; a test covers zero-column expressions in assembly.

*Alternative:* the chart `t ∝ x`, the affine "canonical" chart. Rejected: #74 decided on the arc-length chart, so that #83 can keep vertical tangents finite. The geometry-ir delta opens the requirement.

*Alternative:* interpolate the local points from the table, without evaluating `f`. Rejected: points would lie on an interpolant, not on the graph, and the export error budget would have to share that error.

### 3. Allocation-free evaluation via a thread-local scratch

Formulas evaluate `f` with `Expression::eval_in` and a `thread_local!` `RefCell<Vec<Dual2<f64>>>` in `function.rs`. After the first call per thread it never allocates. The wasm build is single-threaded. The domain check guarantees that evaluation inside [x0, x1] cannot fail, so the formula uses `expect` with that reason.

*Alternative:* a scratch passed through the evaluation context changes every formula signature for one kind. A fixed stack array would need a node limit in the expression language. Both rejected.

### 4. Jacobians: only `O` and `θ`

Chart block `(Ox, Oy, θ)`. With `p = R(θ)·local` and `R′(θ)·local = (−p_y, p_x)`:

| formula | columns | Jacobian |
|---|---|---|
| point (on-curve, start, end) | Ox, Oy, θ | `[[1, 0, −p_y], [0, 1, p_x]]` |
| tangent / normal at t | θ | `[[−v_y], [v_x]]` for the rotated vector `v` |
| origin | Ox, Oy | identity |
| direction `(cos θ, sin θ)` | θ | `[[−sin θ], [cos θ]]` |
| length | none | empty |

Curvature and the second derivative of `f` never enter a Jacobian, as decided in #74.

### 5. Curvature, piece bounds and shape

- **`curvature(t)`:** `|su·sv·f″(x)| / (su² + sv²·f′(x)²)^{3/2}` at `x = x(t)`.
- **`piece_bound(t0, t1)`:** upper bounds, never estimates (review of #89):
  - the x interval is widened to the enclosing table nodes, so the interpolation error cannot hide anything;
  - **length:** arc length of the piece by adaptive Gauss–Legendre plus a margin of 1e-9·L;
  - **curvature:** `f′`, `f″` over that x interval with `eval_with::<Dual2<Interval>>`; bound `su·sv·sup|f″| / (su² + sv²·inf f′²)^{3/2}`.

  This path is the Mesh, not the solver, so it may allocate.
- **`shape(c, a, tolerance)`:** `hermite_shape` on the chart's `γ` and `γ′` with knots `[0, 1]` and `sampled_deviation`.
- **`UNIFORM_CURVE`:** false, so the Mesh uses the general sampler.
- **`canonical`:** identity, because no chart value has a sign convention.

### 6. Capabilities

`geometry_caps!(Function: curve, endpoints, anchor, line)` with name prefix `fn`. Address suffixes (`origin`, `start`, `end`, `at(t)`, `tangent(t)`, `normal(t)`, `length`, `direction`) come from the capabilities, so `address.rs` needs no change.

### 7. Commands and errors

- **`kinds/function.rs`:**
  - `FunctionParams { origin, heading, expression, range, scale: Option<[f64; 2]> }` with `TAG = "function"`;
  - `FunctionSet { expression, range, scale }`, each optional.
- **`CommandError::InvalidExpression { message }`** with class `invalid expression`:
  - parse errors with their byte position;
  - `NotProven` with its sub-range;
  - `TooComplex` with "exceeded its budget" and no function name (review of #88).

  A reversed or non-finite range and non-positive scales stay `InvalidValue`.
- **Solving `set`:** `Command::Set` asks the attribute patch whether it changes a curve's geometry (true for every non-empty function patch).
  - If so: `check_not_frozen` before applying, then `structural_scoped` on the component; a failure is `Unsatisfiable`, and `apply` restores the snapshot.
  - Label `set` keeps its non-solving path unchanged.
- **Declarative export:** writes `expression`, `range` and `scale` always. `scale` is written even when it is (1, 1), to keep every attribute explicit, as for labels.
- The JSON Schema gains the function variants.

### 8. Sketch file version 4

- `VERSION = 4`. `migrate` passes version 3 envelopes unchanged.
- The per-version check rejects a `function` create in versions below 4 ("the kind `function` needs version 4"), next to the existing style-field check for version 2.

## Risks / Trade-offs

- **[Table build cost on every attribute change]** → done only on create, set and load, never in a solve. With an analytic `f` and 1e-11 L accuracy, a few hundred to a few thousand intervals are expected. Measured: `sin(x)` on [0, 100π] needs 11 696 intervals (about 230 per period); a test bounds it below 16 000. A quintic inverse would need fewer, if build time ever matters.
- **[Thread-local scratch hides state]** → it holds only scratch values, and the allocation test covers a function curve after warm-up.
- **[Zero-column length expression]** → covered by a test (EqualLength between a function curve and a segment). If assembly mishandles empty blocks, it gets a fix in this change.
- **[Mesh budget near steep parts]** → piece bounds are sound but may be pessimistic near large `f″`. The Mesh cap applies (#86 review); a test with `exp(x)` on [0, 5] checks that it stays under the cap.
- **[`set` now may move geometry]** → only for function curves. Undo restores the snapshot. Label behaviour and tests are unchanged.
- **[Sampled shape estimate]** → inherited from #80; #99 tracks the proof.
