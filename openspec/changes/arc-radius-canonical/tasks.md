# Tasks

Paths relative to `impls/linux-rust/crates/constraint_runtime`. Tests: `cargo test -p constraint_runtime` from `impls/linux-rust`.

## 1. Canonical chart form

- [x] 1.1 Add the provided method `fn canonical(init: Self::Init) -> Self::Init { init }` to the `Geometry` trait in `src/geometry/formula.rs` (rustdoc: same curve, identity by default) and override it in `src/geometry_kinds/arc.rs` (`R < 0` → `(C, −R, θ₀ + π, Δθ)`). Verify with `cargo check -p constraint_runtime`.
- [x] 1.2 Unit tests in `src/geometry_kinds/tests_arc.rs` (written by the local model, reviewed; `Segment` identity test in `tests_segment.rs`): `canonical` of a negative-radius arc yields `R ≥ 0` and `θ₀ + π`; start / end / on-curve points (t = 0, 0.25, 0.5, 1), tangent, normal and length are equal before and after within 1e-12; positive radius and `R = 0` are unchanged; `Segment::canonical` is identity. Verify that the tests pass.

## 2. Radius expression yields the magnitude

- [x] 2.1 In `src/geometry_kinds/arc.rs` make `LengthFormula::Radius` evaluate to `|R|` with Jacobian `signum(R)` on the radius column (0 at `R = 0`); update the `radius_expr` and kind docs (non-negative in committed State, iterates may be negative, start heading is the polar angle of the start point only for `R ≥ 0`). Add a finite-difference test for the radius expression at `R > 0` and `R < 0` in `src/geometry_kinds/tests_arc.rs`. Verify that the tests pass.

## 3. Canonicalize in Document and Interact

- [x] 3.1 Add `Document::canonicalize(&self, p: &mut Vector)` (per geometry: `with_kind!` → unpack block → `canonical` → pack back) and make `Document::add_geometry` store `K::canonical(init)`. Tests in `src/tests_document.rs`: adding an arc with negative radius stores the canonical form and evaluates to the same start / end points; `canonicalize` on a Vector with a negative-radius arc rewrites only that arc's block. Verify that the tests pass.
- [x] 3.2 In `InteractiveSolve::solve_commit_soft` (`src/interact.rs`) canonicalize `working` before the structural gate at both commit points. Test in `src/tests_interact.rs`: with `StubSolver { succeed: true }` and a working State whose arc radius was negated (with heading shifted to describe the same curve), the committed State has `R ≥ 0` and unchanged endpoints. Verify that the test and `tests/acceptance.rs` pass.

## 4. Docs and integration

- [x] 4.1 Update `docs/constraint-mathematics.md` (arc chart: radius non-negative in committed State, canonicalization on commit) and the `openspec/config.yaml` context ("signed Length radius" → radius `≥ 0` when committed). Verify with `grep -rni "signed radius\|signed Length radius" docs openspec/config.yaml` that no stale wording remains (historical archive excluded).
- [x] 4.2 Run `cargo test --workspace`, `cargo clippy --workspace --all-targets` (0 warnings), `cargo doc -p constraint_runtime --no-deps` (0 warnings) and `openspec validate arc-radius-canonical --strict`. Verify that all pass.

## Workflow follow-up

- Archive the change and sync the `geometry-ir` delta.
- Then propose `curve-invariants-and-relations` (its `CircularExprs::radius_expr` builds on the magnitude semantics from task 2.1).
