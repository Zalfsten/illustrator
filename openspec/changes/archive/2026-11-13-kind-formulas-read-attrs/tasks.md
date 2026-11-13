# Tasks

Implementation: impls/rust, crate `constraint_runtime` (see design.md). Run tests from `impls/rust` with `cargo test`.

## 1. Contract and kinds

- [x] 1.1 In `geometry/formula.rs` add `type Attrs: Debug + PartialEq + 'static` and `fn attrs_of(a: &KindAttrs) -> &Self::Attrs` to `Geometry`. Add `a: &Self::Attrs` after `c` to every chart-dependent method (values and Jacobians of point, vec, length, heading, turn, plus `shape`, `curvature`, `canonical`, including their panicking defaults). Forward it through `Formula::eval` / `Formula::jacobian` and `formula_family!`. Leave `*_params`, the constants and `default_attrs` unchanged, and update the module doc ("Adding a kind"). Verify that `cargo build -p constraint_runtime` reports errors only in the kinds and dispatch sites handled in 1.2–1.3.
- [x] 1.2 Implement the new contract in `geometry_kinds/segment.rs` and `arc.rs` (`type Attrs = ()`, `attrs_of` returns `&()` for `KindAttrs::None`), and in `label.rs` (`type Attrs = LabelAttrs`, returns the `KindAttrs::Label` payload). Formulas ignore `a`; `Arc::shape` passes it to `canonical`. Verify that these files compile.
- [x] 1.3 Add `GeometryInst::attrs_of::<K>() -> &K::Attrs` in `geometry_kinds/catalog.rs`, beside `unpack::<K>` with the same kind `debug_assert`. Pass the attributes at every dispatch site:
  - `expr.rs`: `evaluate`, `accumulate_jacobian`, `VecExpr::jet`;
  - `document.rs`: `curve_shape`, `curve_curvature`, `add_geometry_with` (project the incoming `attrs` with `K::attrs_of` before `canonical`), `canonicalize`.

  Verify that `cargo build --workspace` succeeds (non-test targets).

## 2. Tests

- [x] 2.1 Update the direct formula calls in the kinds' unit tests (`geometry_kinds/tests_arc.rs`, `tests_segment.rs`, `tests_label.rs`, `tests_catalog.rs`) with `&()` or the label's attributes, without changing any assertion. Verify that `cargo test -p constraint_runtime` passes and that `git diff` of those files shows only added attribute arguments.
- [x] 2.2 Add tests for the spec scenarios and the projection, in the sibling `tests_*.rs` files:
  - **`tests_label.rs`:** a label's origin expression has the same `param_indices` and Jacobian before and after `set_label_attrs` with different source, size and alignment. Its origin and direction values are unchanged.
  - **`tests_catalog.rs`:** for every kind in `ALL_KINDS`, `GeometryInst::attrs_of::<K>()` of a document geometry equals `K::attrs_of(&K::default_attrs())`, and for a label equals the attributes set with `set_label_attrs`.

  Verify that the new tests pass.

## 3. Documentation

- [x] 3.1 Update the **Point formula**, **GeometryInst** and **GeometryKind** rows in `docs/constraint-mathematics.md` (formulas receive the kind's typed attributes; attributes are not solved and add no Jacobian columns) and the `formula.rs` / `geometry_kinds/` lines in the project context of `openspec/config.yaml`. Verify by reading both against design decisions 1–2.

## 4. Integration

- [x] 4.1 Verify the whole workspace: `cargo test` (including `tests/alloc_free.rs`) and `cargo clippy --all-targets` in `impls/rust` pass, and `impls/rust/scripts/check-wasm.sh` succeeds.
- [x] 4.2 Validate the change: `openspec validate kind-formulas-read-attrs --strict` passes.

## Workflow follow-up

- Archive the change after review (`/opsx:archive`), then close #79.
