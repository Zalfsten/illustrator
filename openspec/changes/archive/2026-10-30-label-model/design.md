# Design

## Context

See proposal.md for the motivation and the specs for the requirements. This design targets `impls/rust`. Four facts about the current code shape it:

- **Every kind is a curve today.**
  - `geometry_caps!` always implies `CurveExprs`.
  - `LineExprs`, `EndpointExprs` and `CircularExprs` have `CurveExprs` as a supertrait.
  - Three places monomorphize curve factories for every kind through `with_kind!`:
    - `Mesh::from_document` → `append_curve_mesh::<K>`;
    - `interact::curve_length_exprs` → `K::length_expr`;
    - `session::displacement`, which already uses the id-based `doc.on_curve_expr`.
  - A kind without `CurveExprs` would not compile in the first two.
- **`GeometryInst` is `Copy`** and holds only `shell` + `kind`. Geometries have no non-parameter data.
- **Undo and atomicity snapshot the whole `Document`** (`Session::apply`, `undo`, `drag_start`). Anything that must be undone has to live in `Document`.
- **`sketch_session` defines the per-kind command layer.** `GeometryCommand` (`TAG`, `Params`, `init`, `params`) and `command_catalog!` generate `GeometrySpec` and all dispatch. `json.rs` reads ops by hand from a JSON map. `schema.rs` composes one `oneOf` branch per fixed op and per kind. `file.rs` has a `migrate` stub.

## Goals / Non-Goals

**Goals:**

- Admit a kind without the curve capability, without adding kind matches outside the catalogs.
- Keep label attributes and document settings inside `Document`, so the existing snapshot undo and atomicity cover them unchanged.
- Add `set` / `set_document` within the existing interchange machinery: strict reading, generated schemas, lossless round trip.

**Non-Goals:**

- No GUI for labels. `sketch_ui` only has to keep working with drawings that contain labels.
- No typesetting types or metrics in the core. `LabelAttrs` is plain data.

## Decisions

### 1. `curve` becomes an explicit capability

- `geometry_caps!(Segment: curve, endpoints, line)`. The macro no longer implies `CurveExprs`, and `Caps::CURVE` is no longer the base; `Caps::NONE` is.
- `LineExprs` and `EndpointExprs`/`CircularExprs` change their supertrait from `CurveExprs` to `Geometry`. This is safe because their factories don't use curve formulas.
- **New** `AnchorExprs: Geometry` with `origin_expr`, a new `PointFormula::Origin`, and `Caps.anchor`.
- In `Geometry`, `point` / `vec` / `length` (and their Jacobians) get the same panicking defaults that `heading` / `turn` already have. A label implements only `point` (Origin) and `vec` (Direction).
- **Curve paths are gated by capability, not by kind:**
  - Mesh tessellation is no longer generic over the kind. It takes the geometry's `Caps`, returns nothing without `caps.curve`, and otherwise builds the on-curve and tangent expressions directly from the formula tags. This is the same pattern it already used for endpoints with `caps.endpoints`, and it needs no lookup per sample.
  - `curve_length_exprs` uses `filter_map(|id| doc.length_expr(id))`.
  - This removes `with_kind!` from both.

*Alternative:* a second dispatch macro `with_curve_kind!` that lists only curve kinds and keeps the monomorphized mesh path. Rejected: it is a second kind list that must be kept in sync, and the spec already asks callers outside the kinds to go through capability queries.

*Alternative:* treat the label as a degenerate curve (its baseline as a ray). Rejected: `at(t)`, `tangent`, `length` and drag length floors would all get meanings nobody wants. A zero-length floor would even fight the label.

### 2. Label chart

```
chart_atoms! { Label { origin: Point2, heading: HeadingAngle } }
geometry_caps!(Label: anchor, line);
NAME_PREFIX = "lbl"
```

- `Origin` → `origin`. Its Jacobian is I₂ over the `origin` slot (`point_params` = origin range).
- `Direction` → `(cos θ, sin θ)`. Its Jacobian is `(−sin θ, cos θ)ᵀ` over the `heading` slot only (`vec_params` = heading range), so a parallel constraint never touches the origin columns.
- `canonical` stays the identity: there is no wrap at type level, as for other heading angles.

The direction is a **unit** vector, unlike a segment's `end − start`. `VectorsParallel` / `VectorsOrthogonal` use the 2-D cross / dot product, which is homogeneous in each vector's length, so they work unchanged.

### 3. Attributes live in `GeometryInst`

- `GeometryInst` gains `attrs: KindAttrs`, an enum `{ None, Label(LabelAttrs) }` defined in the geometry catalog next to `GeometryKind`. `GeometryInst` becomes `Clone` instead of `Copy`.
- `Geometry::default_attrs() -> KindAttrs` (default `None`; `Label` returns `LabelAttrs::default()` with an empty source). `add_geometry::<K>` stores it, so the 93 existing `add_geometry` call sites stay unchanged.
- **New `Document` API:**
  - `add_geometry_with::<K>(role, init, attrs)`, which debug-asserts that the variant matches the kind;
  - `label_attrs(id) -> Option<&LabelAttrs>`;
  - `set_label_attrs(id, LabelAttrs) -> bool`;
  - `set_geometry_attrs(id, KindAttrs) -> bool`, a setter independent of the kind, used by `set`.
- `LabelAttrs { source: String, markup: Markup, halign: HAlign, valign: VAlign, offset: Vec2, size: Option<f64> }` with core enums. `offset` is a free vector in the label frame, in pt, never in world units, which its doc comment states. `size: None` means inherited.

*Alternative:* a side table `HashMap<GeometryId, LabelAttrs>` in `Document`. Rejected:
- removal and the "exactly the labels have an entry" invariant would need their own bookkeeping;
- `add_geometry::<Label>` would need a kind check outside the catalog.

Kept in the instance, attributes are cloned with the snapshot and dropped with the geometry for free.

### 4. Document settings live in `Document`

`DocumentSettings { scale: f64, font_size: f64, preamble: String }` with `Default` (10, 10, ""), plus `Document::settings()` and `set_settings()`. The core does not validate ranges; the session does, as it does for every command value. `Session::settings()` exposes them for the query requirement.

*Alternative:* keep settings in `Session` and snapshot `(Document, DocumentSettings)` pairs. Rejected: it touches every undo / redo / drag / atomicity path for no gain.

### 5. Command layer

- **`create`:**
  - `GeometryCommand` gains `fn attrs(p: &Self::Params) -> Result<KindAttrs, CommandError>` (default `Ok(KindAttrs::None)`).
  - `params(init)` becomes `params(init, attrs: &KindAttrs)`; segment and arc ignore `attrs`.
  - `GeometrySpec::add_to` calls `add_geometry_with`.
  - `LabelParams` = `origin`, `heading`, `source` (required), plus `markup`, `halign`, `valign`, `offset`, `size` (each `#[serde(default)]`). On export every field is written, except `size` when it is `None` (`skip_serializing_if`).
- **`set`:**
  - A new trait `AttributeCommand: GeometryCommand` with `type SetParams` (all fields optional, `deny_unknown_fields`) and `fn apply(set, &mut LabelAttrs-like) -> Result<(), CommandError>`, which validates values.
  - `command_catalog!` gets a third list, `set: Label;`. It generates `AttrSpec` with `TAGS`, `from_fields`, `serialize_tagged` and `kind_schemas`, as for create and constrain.
  - `Command::Set { name, attrs: AttrSpec }`.
  - Execution:
    1. look up the name; a constraint gives `WrongObjectType { expected: "geometry" }`;
    2. a kind mismatch gives `WrongObjectType { expected: <tag> }`;
    3. clone the attrs, apply, write back.
  - No frozen check and no solve.
- **`size: null`:** `SetParams.size: Option<Option<f64>>` through a small `wire::finite_or_null` module: absent → `None`, `null` → `Some(None)`. Its schema is `{"type": ["number", "null"], "exclusiveMinimum": 0}`.
- **No other `null`:** every other optional field of `set` / `set_document` goes through `wire::present`, so `null` there is a reading error rather than being read as absent. Its schema is the plain field type (`#[schemars(with = "T")]`).
- **Non-empty rule:** checked after deserializing in `from_fields` (a reading error, "no attribute given"). The schema expresses it as `minProperties` (reserved fields + 1).
- **`set_document`:** a fixed op with `SetDocumentFields { scale, font_size, preamble }`, all `Option`, `deny_unknown_fields`, non-empty. `Command::SetDocument { scale, font_size, preamble }`. Ranges are validated with `InvalidValue { field }`.
- **Wire enums:** `Markup` / `HAlign` / `VAlign` get lowercase serde mirrors in `wire.rs`, like `Role`, so `constraint_runtime` stays serde-free.
- **Rest of the interchange machinery:** `json.rs` adds `"set"` / `"set_document"` to `OPS` and to `command()`. `decl_command()` keeps rejecting them. `schema.rs` adds the set branches (op `set`, kind const) and the `set_document` fixed branch. The schemas under `sketch_session/schema/` are regenerated with `UPDATE_SCHEMA=1`.

*Alternative:* `set` without a `kind`, with fields checked against the named object at execution time. Rejected: strict reading and the JSON Schema could then only check the union of all kinds' attributes, and a typo would be caught only when executing.

### 6. File version 2

- `VERSION = 2`; `load` accepts 1 and 2.
- `migrate(1, env)`: first reject `scale` / `font_size` / `preamble` if present, as unknown fields (v1 has none), then insert the defaults. `migrate(2, env)` is the identity.
- After migration, `load` removes the three settings fields, which are now required, and checks them: finite, > 0, string. Errors become `LoadError::Envelope` ("missing field", "invalid value of `scale`"). The settings are applied to the fresh `Document` before the commands run.
- `save` writes `format`, `version`, `scale`, `font_size`, `preamble`, `commands`, each on its own line. Numbers and the string go through `serde_json`, which escapes line breaks in the preamble, so the output is deterministic and round-trips byte for byte.

### 7. Displacement of labels

`session::displacement` keeps its curve sampling. For a geometry with no on-curve expression, it compares `doc.origin_expr(g)` before and after. A label that only turns has displacement 0. This is accepted, because without typesetting the core does not know how far the text reaches.

### 8. GUI

No label UI.
- Labels have no mesh edges, so they are not painted or picked.
- `logical_points` already takes only endpoint classes, so a free label has no handle, while a label joined to an endpoint is moved along with that endpoint's handle.
- Tools already check `caps` (parallel / orthogonal need `line`, which a label has but cannot be picked through the mesh).

## Risks / Trade-offs

- **Labels flip upside down.** `VectorsParallel` allows anti-parallel solutions, so a label made parallel to a segment may come out at θ + π (#42 / #44). This is accepted for now; the spec states that there is no automatic flip. An "always upright" attribute can come later.
- **`GeometryInst` loses `Copy`.** Compile errors will point at every place that copied it implicitly. They are expected to be few (`catalog.rs`, tests).
- **Breaking interchange.**
  - Clients that hard-code the op list break.
  - Version 1 readers cannot read version 2 files. There are none outside this repository.
  - → The schema files and the version number make this explicit.
- **`Option<Option<f64>>` in the schema.** schemars may not render the double option the way we want. → Write the field schema by hand with `schema_with`, as `wire::point` already does.

## Migration Plan

Existing version 1 files load unchanged with default settings; saving writes version 2. There is no rollback path: an older build rejects version 2 as unsupported, which is the specified behaviour.
