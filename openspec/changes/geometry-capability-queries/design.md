# Design

## Context

Implementation: `impls/rust` (Rust, egui front-end, native and WASM builds).

Today a kind opts into capabilities with marker trait impls next to its chart (`impl CurveExprs for Arc {}`, `impl EndpointExprs for Arc {}`, `impl CircularExprs for Arc {}`). The factory methods on those traits are kind-agnostic: they only wrap `(GeometryId, formula tag)` via the crate-private `Expr::new`. The type parameter only serves as compile-time proof that the kind supports the tag.

Code that works on dynamic ids bypasses that proof in two ways:

- `GeometryKind::has_line_exprs` / `has_circular_exprs` are hand-written `matches!` lists in `geometry_kinds/catalog.rs`. `sketch_ui` checks `has_line_exprs()` and then calls `Segment::direction_expr(id)` for any picked id.
- `Mesh::from_document` dispatches `with_kind!(… => append_open_curve_mesh::<K>(…))`, and that helper is bounded `C: CurveExprs + EndpointExprs`. Because `with_kind!` monomorphizes its body for every kind, every kind must implement `EndpointExprs`, so a kind without endpoints would not compile.

Rust has no specialization, so "does `K` implement trait `T`?" cannot be asked inside a generic body. Runtime capability information must therefore be a value the kind provides.

## Goals / Non-Goals

**Goals:**

- One declaration per kind produces both the capability marker impls and the runtime capability set.
- Dynamic callers (GUI tools, Mesh) need no kind names and no trait bounds beyond `CurveExprs`.
- No change to expression evaluation, assembly or the solver.

**Non-Goals:**

- Verifying that a kind's formula `match` arms actually implement the tags it claims (for example `VecFormula::Direction` in `Segment::vec`). That is the job of the generic conformance suite (#23).
- A registration list that generates `GeometryKind` / `with_kind!` (#25).

## Decisions

### D1: `Caps` is a plain `Copy` struct of booleans

```rust
pub struct Caps { pub curve: bool, pub endpoints: bool, pub line: bool, pub circular: bool }
```

It has `const` constructors and helpers (`Caps::CURVE`, `.with_endpoints()`, …) so it can be built in `const` context.

- *Alternative:* the `bitflags` crate. It adds a dependency for four flags, so we don't use it.
- *Alternative:* a hand-rolled `u8` bitset. It is more compact but less readable in `Debug` output and at call sites. Size is irrelevant, since the set is never stored per geometry; it is a per-kind constant.

`curve` is always `true` for today's kinds. It is kept so that later non-curve kinds (regions, labels) fit without changing the type.

### D2: A declaration macro is the single source of truth

```rust
geometry_caps!(Segment: endpoints, line);
geometry_caps!(Arc: endpoints, circular);
```

The macro expands to:

- `impl CurveExprs for K {}` (always), plus `impl EndpointExprs for K {}`, `impl LineExprs for K {}` and `impl CircularExprs for K {}` for each listed capability;
- `impl GeometryCaps for K { const CAPS: Caps = … }`.

`GeometryCaps` becomes a supertrait of `Geometry`, so every kind must invoke the macro. It is documented as "implemented only via `geometry_caps!`". A sealed-trait trick is not worth it for an internal crate; review and the macro's doc comment are enough.

This fits the existing style (`chart_atoms!`, `formula_family!`). The kind's `impl Geometry` stays hand-written.

- *Alternative:* a hand-written `const CAPS` on `Geometry`. It can drift from the marker impls, which is exactly the problem being fixed.
- *Alternative:* keep marker impls and check consistency at test time by evaluating tags and catching `unreachable!` panics (`catch_unwind`). That is fragile, fails only at test time, and still keeps two sources of truth.
- *Alternative:* derive `CAPS` from the trait impls. This is not possible without specialization.

### D3: Runtime queries on `GeometryKind` and `Document`

- `GeometryKind::caps(self) -> Caps` comes via `with_kind!(self, K => K::CAPS)`. It is the only use of the kind tag for capabilities.
- `Document::caps(&self, id) -> Option<Caps>` returns `None` for an unknown id.
- `has_line_exprs` and `has_circular_exprs` are removed.

### D4: Identity-based factories return `Option`

`Document` gains:

```rust
fn start_expr(&self, id) -> Option<PointExpr>
fn end_expr(&self, id) -> Option<PointExpr>
fn direction_expr(&self, id) -> Option<VecExpr>
fn center_expr(&self, id) -> Option<PointExpr>
fn radius_expr(&self, id) -> Option<LengthExpr>
```

Each one checks `caps(id)` and then builds the expression with the same crate-private `Expr::new(id, tag)` the static factories use. The result is therefore identical to `K::direction_expr(id)` by construction.

Curve-capability factories (`on_curve_expr`, `tangent_expr`, `normal_expr`, `length_expr`) get the same treatment for symmetry. They return `None` only for unknown ids or kinds without `curve`.

- *Alternative:* `Result<_, CapabilityError { UnknownGeometry, Missing(Capability) }>`. That would be more informative, but today's only callers (GUI tools) produce their own user message. `Option` keeps call sites short, and switching to `Result` later is a local change. The spec only requires "reports not available".
- *Alternative:* free functions in `geometry::capabilities` that take `&Document`. Methods on `Document` are more discoverable, and `Document` already resolves ids.

Static, typed factories (`Segment::start_expr`, …) stay. Statically typed code such as the demo scene, tests and benches keeps compile-time guarantees.

### D5: Mesh tessellation needs only `CurveExprs`

`append_open_curve_mesh<C: CurveExprs + EndpointExprs>` becomes `append_curve_mesh<C: CurveExprs>`:

- The first and last vertices come from the endpoint expressions when `C::CAPS.endpoints`, and from `on_curve(Normalized(0 / 1))` otherwise. For a closed curve both coincide, so the polyline closes.
- Endpoint hit targets (`endpoints` vector) are pushed only when `C::CAPS.endpoints`.

`C::CAPS` is a constant, so the branch folds away per monomorphized kind.

### D6: Testing the endpoint-free path without a new kind

There is no admitted kind without endpoints. Two options:

- (a) a `#[cfg(test)]` full-circle kind, with its own variant and `with_kind!` arm;
- (b) let the mesh helper take its capability set as a parameter (`append_curve_mesh::<C>(caps, …)`, called with `K::CAPS` in production) and test it with `Segment` and a caps value that has `endpoints` cleared.

We choose **(b)**. It covers the code path and both spec scenarios (edges present, no endpoint hit) with a few lines. (a) would put a test-only variant into the closed enum and require a complete chart for a kind that does not exist yet. It fits better with #23/#25, when a real closed curve arrives.

The compile-level guarantee ("`with_kind!` body no longer requires `EndpointExprs`") is covered by the signature change itself.

### D7: GUI tools

- The Parallel/Orthogonal tool checks `doc.caps(id).is_some_and(|c| c.line)` and builds the relation from `doc.direction_expr(id)`, with no `Segment::` reference.
- The Join tool already picks through Mesh endpoint hits, which are now capability-gated (D5). No further change is needed.
- Tool feedback texts stay the same.

## Risks / Trade-offs

- [Macro hides the marker impls; a reader searching for `impl LineExprs for Segment` won't find it] → The macro's doc comment lists its expansion, and each kind module has a one-line comment at the invocation.
- [`curve: true` everywhere looks redundant] → It is accepted as a cheap placeholder for non-curve kinds. The macro sets it implicitly, so no kind declares it.
- [Option factories duplicate the static factory list] → Both build the expression via the same `Expr::new(id, tag)`. A unit test per factory asserts `doc.x_expr(id) == Some(K::x_expr(id))` for each kind that has the capability, and `None` otherwise.
- [Claimed capability but missing formula arm] → Not caught here. It still panics (`unreachable!`) on first evaluation, as today, and #23 will cover it.

## Migration Plan

This is an internal API only. `sketch_ui` is the only consumer of `has_line_exprs` and is updated in the same change. There is no persisted data and nothing to roll back beyond a revert.
