# Design: Kind formulas may read kind attributes

## Context

See proposal.md for motivation. Target implementation: impls/rust, crate `constraint_runtime`.

- **`Geometry` trait** (`geometry/formula.rs`): every formula is a function of the unpacked chart `c: Self::Init` only. This covers values and Jacobians of point, vector, length, heading and turn, plus `shape(c)`, `curvature(t, c)` and `canonical(c)`. Connectivity (`*_params(f)`) depends on the formula alone.
- **`Formula` trait:** the macro-generated `Formula` impls forward `eval::<K>(c)` / `jacobian::<K>(c, j, scale)` to these methods. They are the only path from opaque expressions to a kind.
- **Instances** (`geometry_kinds/catalog.rs`): `GeometryInst` stores `attrs: KindAttrs`, an enum `None | Label(LabelAttrs)`. `set_attrs` already refuses attributes of another variant, so an instance's attributes always match its kind. `Geometry::default_attrs() -> KindAttrs` supplies the initial value. `GeometryInst::unpack::<K>(p)` is the typed access to the State block.
- **Dispatch sites** that unpack a chart and call a kind formula:
  - `expr.rs`: `evaluate`, `accumulate_jacobian`, `VecExpr::jet`.
  - `document.rs`: `curve_shape`, `curve_curvature`, `add_geometry_with` (canonical before packing), `canonicalize`.
  - Inside the kinds: `Arc::shape` calls `canonical`.
- **Direct calls in tests:** about 40, mostly in `geometry_kinds/tests_arc.rs`, plus `tests_segment.rs`, `tests_label.rs` and `tests_catalog.rs`.
- **Hot path:** the solver path is allocation-free and guarded by `tests/alloc_free.rs`.

## Goals / Non-Goals

**Goals:**

- A kind can use its own attributes in every formula, typed (no matching on `KindAttrs` inside formulas).
- No cost on the evaluate / Jacobian path beyond passing a reference; no allocation.
- Segment, arc and label behave exactly as before.

**Non-Goals:**

- Attribute types or caches for function curves (#81).
- Changing how attributes are stored, edited, serialized or compared.

## Decisions

### 1. Typed attributes per kind via an associated type

`Geometry` gains `type Attrs: Debug + PartialEq + 'static` and a projection

    fn attrs_of(a: &KindAttrs) -> &Self::Attrs;

- **Segment and arc:** `type Attrs = ()` and return `&()` for `KindAttrs::None`.
- **Label:** `type Attrs = LabelAttrs` and returns the payload of `KindAttrs::Label`.
- **Wrong variant:** this is unreachable (`set_attrs` keeps variants consistent) and panics like the other unsupported paths.

`GeometryInst::attrs_of::<K>() -> &K::Attrs` sits next to `unpack::<K>` and has the same `debug_assert` on the kind. `default_attrs() -> KindAttrs` stays as it is.

*Alternatives:*

- **Pass `&KindAttrs` untyped:** every kind with attributes would match the enum inside every formula, and the function kind would unwrap in a dozen places.
- **Put attributes into `Self::Init`:** `Init` is a `Copy` tuple produced by `Chart::unpack` from State. Attributes hold strings and, later, expression trees, and they are not State. That would break the chart's role as the State layout.
- **Side channel, such as a global table keyed by geometry id:** excluded by #79, and it would hide a data dependency from undo and from tests.

### 2. One extra argument on every chart-dependent formula; connectivity unchanged

Each formula that takes `c: Self::Init` also takes `a: &Self::Attrs`, directly after `c`:

- `point(f, c, a)`, `point_jacobian(f, c, a, j, scale)`;
- the same for `vec`, `length`, `heading` and `turn`;
- `shape(c, a)`, `curvature(t, c, a)`, `canonical(c, a)`.

`Formula::eval::<K>(self, c, a)` and `Formula::jacobian::<K>(self, c, a, j, scale)` forward it.

`*_params(f)` keeps its signature, so connectivity cannot depend on attributes, as the spec requires. Constants (`KIND`, `NAME_PREFIX`, `UNIFORM_CURVE`) and `default_attrs` are unchanged.

`canonical` takes attributes too, because a function curve's canonical chart may depend on its x range. `add_geometry_with` projects the incoming `attrs` with `K::attrs_of` before packing; `canonicalize` uses the instance's attributes.

*Alternative:* a context struct `ChartArgs<K> { c, a }` as the single argument. It would make future additions cheaper, but every existing destructuring pattern `(center, radius, start, sweep): Self::Init` would become a nested pattern or an extra `let`, which is more churn now for a speculative benefit. Rejected.

### 3. Attributes are source data; caches are derived

`KindAttrs` keeps deriving `PartialEq` on the stored source values (label text today; expression text, range and scale for #81), and Document snapshots for undo keep carrying them. If #81 compiles an expression, the compiled form must not take part in equality. It can live in a wrapper whose `PartialEq` ignores it, or outside the attributes. This change adds no cache. The rule is recorded here so #81 does not have to revisit the contract.

### 4. No behaviour change, checked by tests

- **Existing tests:** they keep their assertions, and only direct formula calls gain `&()` or the label's attributes.
- **New tests for the two added spec scenarios:**
  - a label's origin Jacobian refers to the same State columns before and after its text attributes are replaced;
  - its origin and direction values are unchanged.
- **Catalog test:** `tests_catalog.rs` gains a check that `GeometryInst::attrs_of::<K>()` returns the stored attributes for every kind.

## Risks / Trade-offs

- [Signature churn in about 40 test call sites] → Mechanical: `&()` for segment and arc, `&LabelAttrs::default()` or the stored attributes for label. The compiler finds every site.
- [An extra `match` per formula call for the projection from `KindAttrs`] → One enum discriminant check next to the existing `with_kind!` match. No allocation; `alloc_free.rs` keeps guarding it.
- [Formulas could start depending on attributes in ways the solver cannot see] → Attributes are fixed during a solve (proposal non-goal), and connectivity stays attribute-independent (decision 2), so the solver's sparsity and Jacobian columns are unaffected.

## Migration Plan

This is an internal API change inside `constraint_runtime`. No other crate calls kind formulas directly, and there is no file-format, command or export change. Rollback is a revert.
