# Design

## Context

See proposal.md — Why. linux-rust already has object-safe `Geometry`, `GeometryShell`, `GeometryStore::unpack_chart(id, p)`, and `DocumentStructure.next_id`. This change separates identity allocation and chart mapping without ECS side-maps or splitting the shell.

Related in-flight change: `geometry-object-safe-shell` (spec language for shell storage). Prefer not conflicting edits to the same requirement text; this change's geometry-ir delta focuses on id allocation and resolve-then-unpack.

## Goals / Non-Goals

**Goals:**

- Flat SoC: `IdGenerator`, `Mapper` (or equivalent store helpers), Document remains sync authority.
- `unpack_chart` / param-block helpers take `&dyn Geometry` (resolved), not `GeometryId`.
- Keep `GeometryShell { id, role, param_offset }`; integer ids; Document assigns on add.

**Non-Goals:**

- UUID ids; geometry constructors calling a generator.
- Side-map-only layout; shell companion split.
- Deep generic nesting.

## Decisions

### 1. IdGenerator (linux-rust)

```text
IdGenerator { next: usize }
  allocate() -> GeometryId
  observe(GeometryId)  // next = max(next, id.0 + 1)
```

- Owned by `DocumentStructure` (or Document) — replace raw `next_id` field.
- `push_geometry` / import calls `observe` when an instance already carries an id.
- `Document::add_geometry` calls `allocate` (or allocate then build shell).
- No process-global singleton; kinds never import the generator.

**Alternatives:** UUID (rejected — size/debug noise); geometry asks generator (rejected — ambient coupling).

### 2. Mapper view (linux-rust)

```text
Mapper<'a> { store: &'a dyn GeometryStore, state: &'a State }
  // or simply free helpers / GeometryStore methods without holding store if redundant

fn param_block(g: &dyn Geometry) -> Range<usize>  // already on Geometry
fn unpack_chart<C: Chart>(g: &dyn Geometry, p: &State) -> C::Init
```

Preferred shape for minimal syntax: keep helpers on `GeometryStore` as **associated functions or default methods that take `&dyn Geometry`**, plus an optional thin `Mapper<'a>` that bundles `&impl GeometryStore` + `&State` for call sites that always have both:

```text
mapper.map::<Segment>(g) -> Segment::Init
```

Expressions:

```text
evaluate:
  g = store.expect_geometry(self.geometry)  // one lookup
  init = store.unpack_chart::<Segment>(g, p)
```

**Alternatives:** Mapper owns State (rejected — evaluate must pass working copies); unpack keeps id (rejected — duplicate lookup).

### 3. Retain GeometryShell

Single companion bag remains. Document writes id+offset(+role) on register; Mapper/Store reads offset via `g.param_block()`. Reindex-on-remove stays on shell fields via `Geometry::reindex_after_removal`.

### 4. Stability note (documentation only)

`Box<dyn Geometry>` keeps heap pointees address-stable across `Vec` growth, but safe `&dyn Geometry` must not outlive structural mutation. Durable handles remain `GeometryId`.

## Risks / Trade-offs

- **[Risk] Conflict with `geometry-object-safe-shell` deltas on geometry-ir** → Mitigation: different requirements; sync/archive carefully.
- **[Trade-off] Mapper as type vs methods only** → Prefer methods + optional view struct; avoid forcing Mapper through every call if a one-liner method suffices.
- **[Risk] Over-abstracting IdGenerator** → Keep it a tiny struct in `shell.rs` or `document.rs`; no trait unless a second allocator appears.

## Migration Plan

1. Add `IdGenerator`; wire DocumentStructure.
2. Change `unpack_chart` / `chart_param_block` signatures to `&dyn Geometry`; fix `expr.rs`.
3. Optionally add `Mapper` view if it clarifies call sites; otherwise stop at store methods.
4. Update docs/config; run `cargo test -p constraint_runtime`.

## Open Questions

None blocking — Mapper-as-struct vs methods-only can be chosen at apply time favoring fewer types.
