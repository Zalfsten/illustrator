# Tasks

Implementation: impls/rust, crates `constraint_runtime`, `sketch_export`, `sketch_ui` (see design.md). Run tests from `impls/rust`, e.g. `cargo test -p constraint_runtime`. New unit tests go in `src/tests_<module>.rs` next to the module.

## 1. Cubic helpers (design decision 2)

- [x] 1.1 Add `constraint_runtime/src/geometry/cubic.rs` with free functions on four points (`start`, `c1`, `c2`, `end` as `Point2`): evaluation (`Point2`), derivative (`Vec2`), axis-extremum parameters in (0, 1), de Casteljau split at `u`, arc length on `[u0, u1]` (adaptive 8-point Gauss–Legendre, depth ≤ 20, 1e-13 relative) and parameter at arc length (safeguarded Newton, 1e-13 relative, the quadrature's accuracy). Register it in `geometry/mod.rs`. Verify with tests in `geometry/tests_cubic.rs`:
  - a cubic with collinear, evenly spaced control points has the chord as its length, and parameter at length is linear;
  - the length of a quarter-circle approximation (`k = 4/3·tan(π/8)`) matches π/2 to 3e-4 relative (its known radial error), and matches a 10⁵-edge polyline to 1e-9;
  - splitting at `u` and evaluating both halves reproduces the original curve at matched parameters;
  - the extrema of (0, 0), (1, 2), (2, 2), (3, 0) are at the known parameter, with height 1.5;
  - parameter at length is the inverse of length on a strongly non-uniform cubic.
- [x] 1.2 Make `sketch_export/src/bounds.rs` use the shared cubic evaluation and extrema instead of its own `cubic` / `extrema_cubic`; keep the quadratic helpers local. Verify that `cargo test -p sketch_export` passes unchanged.

## 2. Cubic shape piece (geometry-ir, design decision 1)

- [x] 2.1 Add `ShapePiece::Cubic { start, c1, c2, end }` in `geometry/shape.rs`, and implement for it `length`, `point_at(s)` and `sub(s0, s1)` with `s` as the arc-length fraction, plus `bounding_points` (ends and axis extrema). Update the module and type docs: the vocabulary is now three pieces, and only closed-form kinds are exact. Verify with tests in `geometry/tests_shape.rs`:
  - `point_at(0)` / `point_at(1)` are the ends;
  - `point_at(0.5)` halves the arc length;
  - `sub(0.2, 0.7)` has 0.5 of the length and the same points as the original;
  - the bounding points span the curve's exact box for the (0, 0), (1, 2), (2, 2), (3, 0) cubic.

## 3. Shape query with a tolerance (geometry-ir, design decision 3)

- [x] 3.1 Change `Geometry::shape(c, a)` to `shape(c, a, tolerance)` and `Document::curve_shape(id, p)` to `curve_shape(id, p, tolerance)` (world units, > 0, debug assertion). Segment and arc ignore the tolerance. Update the docs and `geometry_kinds/tests_catalog.rs`. Verify with a new test in `geometry/tests_shape.rs` for the spec scenario "Closed-form shapes ignore the tolerance" (tolerances 1 and 1e-9 give identical pieces), and that `cargo test -p constraint_runtime` passes.
- [x] 3.2 Update the callers:
  - `sketch_export/src/scene.rs`: `SHAPE_TOLERANCE_MM = 0.5e-3` and a function `export_tolerance(settings) = SHAPE_TOLERANCE_MM / scale`, passed to `curve_shape`;
  - `sketch_ui/src/app.rs` `drawing_box`: the same export tolerance (view-independent, see design decision 3).

  Verify with a test in `sketch_export/src/tests_scene.rs` for the spec scenario "Tolerance follows the document scale" (scale 10 → at most 1e-4), and that `cargo test --workspace` passes with unchanged segment and arc export bytes (existing SVG/PDF tests).

## 4. Hermite builder (geometry-ir, design decision 4)

- [x] 4.1 Add `hermite_shape(point, tangent, knots, tolerance, deviation)` and `sampled_deviation(point)` to `geometry/cubic.rs`, per design decision 4:
  - Hermite control points `γ ± γ′·h/3`;
  - 4 initial pieces per knot interval;
  - depth-first halving while `deviation > tolerance`;
  - `MAX_SHAPE_PIECES = 4096`;
  - 7 matched samples × safety factor 2.

  Document the estimate and point to #99. Verify with tests in `geometry/tests_cubic.rs` against curves with known geometry:
  - **quarter and full circle `γ(t) = c + R(cos 2πt, sin 2πt)`:** every point of the shape, densely sampled, lies within δ of the circle (distance = | |q − c| − R |); every curve sample lies within δ of the shape (closest point on the covering piece by Newton projection); the length differs from 2πR (or πR/2) by at most δ; for δ = 1e-3, 1e-6 and 1e-9 relative to R;
  - **sine graph `γ(t) = (4πt, sin 4πt)`:** the same two-sided distance check, with distances to the graph by Newton projection (a 10⁵-edge reference polyline was planned first, but its own chord error of about 2e-9 is too coarse for δ = 1e-9);
  - the first piece starts at `γ(0)`, the last ends at `γ(1)`, every joint equals `γ` at its knot, and given knots are joints;
  - a custom `deviation` closure is used (counting closure; a closure returning 0 gives exactly 4 pieces per knot interval);
  - a closure that never passes stops at `MAX_SHAPE_PIECES`;
  - two calls give identical pieces.
- [x] 4.2 Extend the spec-scenario coverage: for the circle case, test that the shape consists only of cubic pieces and that the length agrees within δ ("Length agrees within the tolerance"). Verify that `cargo test -p constraint_runtime` passes.

## 5. Export of cubic pieces (drawing-export, design decisions 5 and 6)

- [x] 5.1 Map `ShapePiece::Cubic` to `PathSeg::Cubic` in `scene.rs::path`, and make sure `trim` cuts cubic pieces at arc length. Verify with tests in `tests_scene.rs`: trimming L from both ends of a one-cubic shape leaves pieces whose length is the total minus 2L (1e-12 relative), and whose new ends lie on the original curve.
- [x] 5.2 Test the spec scenarios "Cubic piece in SVG" and "Cubic piece in PDF" with a scene built from one cubic piece, in `tests_svg.rs` and `tests_pdf.rs`: one move plus one `C` / `c` with the exact control points in output units. Verify that the tests pass.
- [x] 5.3 Implement tight stroked bounds for cubics in `bounds.rs::stroked` per design decision 6:
  - end corners `B ± h·n`;
  - offset points at the axis-extremum parameters;
  - a numeric maximization of each direction when the sampled curvature reaches `0.5/h`.

  Remove `widened_box` if nothing uses it any more. Verify with tests in `tests_bounds.rs`:
  - the spec scenario "Cubic bounds use the curve, not its control points" (top = 1.5 + w/2, below 2);
  - a cubic with butt ends whose extent is set by an end corner;
  - a cubic whose curvature passes 1/h (found by a random search; the closed-form candidates alone miss its right side by about 0.66), whose bounds match a dense sampling of the stroke outline to 1e-9, checked to fail without the numeric maximization;
  - updated expectations of any label outline test, each checked to move toward the true bounds.

## 6. Documentation

- [x] 6.1 Update `openspec/config.yaml`:
  - the `shape.rs` line: three pieces, arc-length fraction, tolerance;
  - a new `cubic.rs` line: helpers and Hermite builder with the pluggable deviation and #99;
  - the `sketch_export` lines: cubic paths, export tolerance, tight cubic stroke bounds.

  Verify that `openspec validate cubic-shape-pieces --strict` passes.

## 7. Integration checks

- [x] 7.1 Run `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings` and `scripts/check-wasm.sh` from `impls/rust`. Verify that all three succeed.

## Workflow follow-up

- Archive the change after review; sync the `geometry-ir` and `drawing-export` deltas into the main specs.
- Mark #80 done and check it off in #98 and #74.

## Review follow-ups (PR #100)

- [x] R.1 `hermite_shape` refines by priority (largest deviation first) and returns `HermiteShape { pieces, capped }`; tests `hermite_spreads_the_error_at_the_cap`, `hermite_reports_whether_the_cap_stopped_it`.
- [x] R.2 Spec: geometry-ir states that the deviation check may estimate, makes the length agreement a SHOULD and adds the cap scenario; drawing-export qualifies the 1 µm statement.
- [x] R.3 `axis_extrema` uses the stable quadratic roots; test `axis_extremum_with_a_tiny_quadratic_term`.
- [x] R.4 `param_at_length` computes the length scale once instead of per Newton step.
