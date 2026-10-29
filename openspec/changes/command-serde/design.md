# Design

## Context

Target implementation: `impls/rust`, crates `sketch_session` (main work), `sketch_ui` (call sites). See proposal.md for motivation and specs/sketch-commands for the observable contract.

- `sketch_session::Command` ([command.rs](../../../impls/rust/crates/sketch_session/src/command.rs)) has one variant per geometry and constraint kind (`CreateSegment`, `CreateArc`, `Join`, `Parallel`, `Perpendicular`, `Fix`, `EqualLength`) plus `Delete`, `Rename`, `SetRole`. `Session::execute` matches all of them centrally and builds the core values (`Segment` init tuple, `Congruence { a, b }`, …). The `AlreadyJoined` check is inline in the `Join` arm.
- The core has two kind catalogs: `GeometryKind` with the single dispatch macro `with_kind!`, and `constraint_catalog!`, which generates `ConstraintInst` (one variant per kind type, same identifier) and its dispatch. Kinds carry `NAME_PREFIX` (`seg`, `arc`, `join`, `par`, `orth`, `fix`, `eqlen`).
- `Document` already prints expressions back as addresses (`point_address`, `vec_address`, `length_address`); these return `None` for a `Uniform` curve parameter or an unnamed geometry.
- `Document::add_geometry` canonicalizes an arc with negative radius; geometry-ir specifies this, and it stays.
- `Session::new(doc)` accepts any `Document`; the GUI's sample scene is built directly on `Document`, not through commands.
- No crate depends on serde today; `serde` / `serde_json` already appear in `Cargo.lock` through eframe. `scripts/check-wasm.sh` guards the wasm32 build.

## Goals / Non-Goals

**Goals:**
- One definition per kind of its tag, parameters, schema fragment, forward mapping (command → core value) and reverse mapping (drawing → command), with generated dispatch and no hand-written per-kind `match` in the session.
- Exact float round trip, strict reading, and generated schemas whose accidental change fails a test.

**Non-Goals:**
- A format-agnostic serde contract. The interchange form is JSON; other front-ends (Rhai, #18) build `Command` values directly.
- serde on core types; `constraint_runtime` gets no new dependency.
- Exact value preservation on re-execution (that is #8's load mode without eager solve).

## Decisions

### D1 Layer traits live in `sketch_session`, implemented on the core kind types

```rust
pub trait GeometryCommand: Geometry {
    const TAG: &'static str;                       // "segment"
    type Params: Clone + Debug + PartialEq + Serialize + DeserializeOwned + JsonSchema;
    fn init(p: &Self::Params) -> Result<Self::Init, CommandError>;   // finite + range checks
    fn params(init: Self::Init) -> Self::Params;                     // reverse (export)
}

pub trait ConstraintCommand: ConstraintKind + Into<ConstraintInst> + Sized {
    const TAG: &'static str;                       // "parallel"
    type Params: Clone + Debug + PartialEq + Serialize + DeserializeOwned + JsonSchema;
    fn resolve(p: &Self::Params, r: &Resolver<'_>) -> Result<Self, CommandError>;
    fn check(&self, doc: &Document) -> Result<(), CommandError> { Ok(()) }
    fn params(&self, doc: &Document) -> Result<Self::Params, ExportError>;
}
```

Implemented in `sketch_session/src/kinds/<kind>.rs` (`kinds.rs` declares the submodules) for `Segment`, `Arc`, `Congruence`, `VectorsParallel`, `VectorsOrthogonal`, `FixPoint`, `EqualLength`. A local trait on a foreign type satisfies the orphan rule. `Congruence::check` holds the `AlreadyJoined` test that is in the `Join` arm today. `Resolver` wraps `&Document` and holds the existing `point` / `vector` / `length` helpers (address → expression, wrong-family error), moved out of `Session`.

`TAG` and `NAME_PREFIX` are separate constants on purpose (`parallel` vs `par`).

*Alternative:* traits in `constraint_runtime` behind a `serde` feature. Rejected: #24 makes each layer own its trait so the core never depends on IO frontends, and the core has no notion of names in commands.

### D2 `command_catalog!` generates the spec enums and all dispatch

```rust
command_catalog! {
    create:    Segment, Arc;
    constrain: Congruence, VectorsParallel, VectorsOrthogonal, FixPoint, EqualLength;
}
```

generates
- `GeometrySpec { Segment(<Segment as GeometryCommand>::Params), Arc(..) }` and `ConstraintSpec { Congruence(..), … }`, with `tag()`;
- `fn create(doc, role, &GeometrySpec) -> Result<GeometryId, CommandError>` (`K::init` then `doc.add_geometry::<K>`) and `fn build(&ConstraintSpec, &Resolver) -> Result<ConstraintInst, CommandError>` (`K::resolve` then `check`);
- the reverse: `match GeometryKind::$k` / `match ConstraintInst::$k(c)` to `GeometrySpec` / `ConstraintSpec`. A core kind missing from the command catalog makes this match non-exhaustive, so the compiler catches it;
- serde and `JsonSchema` impls of D4 and D6.

The pattern mirrors `constraint_catalog!` in the core. Variant identifiers are the core type names; wire tags come only from `TAG`.

### D3 `Command` and the declarative `DeclCommand`

```rust
pub enum Command {
    Create    { name: Option<String>, role: GeometryRole, geometry: GeometrySpec },
    Constrain { name: Option<String>, constraint: ConstraintSpec },
    Delete    { name: String },
    Rename    { name: String, new_name: String },
    SetRole   { name: String, role: GeometryRole },
}
pub enum DeclCommand {
    Create    { name: String, role: GeometryRole, geometry: GeometrySpec },
    Constrain { name: String, constraint: ConstraintSpec },
}
impl From<DeclCommand> for Command { .. }
```

The per-kind payloads are shared, so the subset costs one small wrapper, not a second copy of every kind. The type guarantees what the spec requires of the subset (names present, no other operations), and the subset schema falls out of the same generator.

`Session::execute` shrinks to five arms. `Outcome` is unchanged.

*Alternatives:* a predicate on one `Command` (no exact subset schema, so MCP clients could not validate files), or `Command<N>` generic over the name type (serde and schemars bounds become noisy for little gain).

### D4 Two-step reading through `serde_json::Value`; strict at both levels

`Deserialize for Command` reads a `serde_json::Map`, removes `op`, then (for `create` / `constrain`) `kind`, `name`, `role`, and hands the remaining map to `serde_json::from_value::<K::Params>`. Every `Params` type derives with `#[serde(deny_unknown_fields)]`, and the fixed operations (`delete`, `rename`, `set_role`) are plain structs with the same attribute, so a misspelled field fails at whichever level owns it. Writing does the reverse: serialize `Params` to a map, insert the reserved fields, and assert in debug builds that no `Params` field uses a reserved name. A unit test checks the reserved names against every kind's schema.

The buffer goes through `serde_json::Value`, so the impls are tied to JSON. That is accepted (see Non-Goals). `serde_json` gets the `float_roundtrip` feature: without it, the parser may be off by one ULP for some inputs, and the round-trip requirement would fail on them.

*Alternative:* `#[serde(flatten)]` over an internally tagged inner enum. Rejected: `flatten` disables `deny_unknown_fields`, so `"rol": "construction"` would silently create a result geometry.

### D5 Wire adapters: points, finite numbers, typed addresses

- `wire::point` (`#[serde(with = …)]`): `Point2` ↔ `[x, y]`; exactly two numbers; schema is an array with `minItems = maxItems = 2`.
- `wire::finite` for plain `f64` fields and inside `wire::point`: serializing a non-finite value returns an error, so a NaN can never be written as `null`. serde_json's own default would write it as `null`, which then fails to read back. Reading a number out of range (`1e999`) already fails in serde_json; a test pins that behaviour.
- `PointAddr`, `VecAddr`, `LenAddr`: `#[serde(transparent)]` string newtypes whose schemas carry a description and example per family. `Resolver::point(&PointAddr)` etc. resolve them. The family check stays at execution, because the family of an address depends on the drawing.
- Range checks (arc `radius >= 0`) and finiteness checks for values built in Rust (the GUI builds commands directly) are done in `GeometryCommand::init` / `ConstraintCommand::resolve` and report `CommandError::InvalidValue { field, reason }` with `reason` ∈ {`NonFinite`, `OutOfRange`}. Schemas state the same range (`minimum: 0`).

### D6 Schemas by schemars 1.x, composed by the catalog, checked in

`Params` types derive `JsonSchema`. The catalog builds the command schemas by hand: a `oneOf` with one branch per fixed operation and per (`op`, `kind`) pair. Each branch merges `op` (const), `kind` (const), `name`, `role` and the kind's `properties` / `required`, with `additionalProperties: false`. `DeclCommand`'s schema is the `create` / `constrain` branches with `name` required.

Both schemas are written to `impls/rust/crates/sketch_session/schema/{commands,declarative}.schema.json`. An integration test (`tests/schema.rs`) regenerates them and compares; with `UPDATE_SCHEMA=1` it rewrites the files. Any change to tags or fields therefore shows up as a reviewed diff. If a second implementation appears, the files can move to a shared location.

### D7 Export: `Session::declarative_commands`

`Session::declarative_commands() -> Result<Vec<DeclCommand>, ExportError>` walks geometries, then constraints, in id order. Ids are allocated increasing and never reused, so id order is creation order. Geometry params come from `GeometryInst::unpack::<K>` and `K::params`; constraint params come from `ConstraintCommand::params`. `ExportError::NotAddressable { constraint }` covers an expression without an address (`Uniform` parameter, unnamed geometry). Documents built through commands cannot produce that; a document passed to `Session::new` can. Committed arcs are canonical (radius ≥ 0), so their export always passes `init` again.

The spec and tests require the re-executed drawing to export the same structure with values within solve tolerance. Each `constrain` on replay runs the eager structural solve from values that already satisfy all constraints.

### D8 Reading errors stay separate from `CommandError`

`Command::from_json(&str)` / `DeclCommand::from_json` / `to_json` return `ReadError` / `WriteError`, newtypes over the serde_json error (message, line, column). Reading is not executing; `CommandError` keeps describing why an apply failed. `#33` will give both structured shapes.

## Risks / Trade-offs

- [Hand-composed command schema can drift from the reader] → Generated from the same catalog list, snapshot-tested, and a test validates sample JSON (valid and invalid objects from the spec scenarios) against the generated schema with `jsonschema` as a dev-dependency.
- [Buffering through `Value` costs an allocation per command] → Negligible at drawing sizes; files are read once.
- [Replay changes values by a few ULPs because each `constrain` solves] → Spec allows tolerance; exact preservation is #8's load mode. If the solver already returns early for a converged start, the test will show exact equality, which is fine.
- [Restructured `Command` breaks every call site] → Few call sites (5 in `sketch_ui/src/app.rs`, the session tests); no compatibility shim is kept.
- [JSON-bound serde impls] → Accepted; Rhai maps to `Command` values, not to serde.

## Migration Plan

No persisted data exists yet, so there is nothing to migrate. Update the GUI and the tests to the new `Command` shape in the same change. Rollback is a revert.

## Open Questions

- Whether the generated schema files should eventually move out of `impls/rust` to a shared location next to the specs. This can wait until a second implementation exists.
