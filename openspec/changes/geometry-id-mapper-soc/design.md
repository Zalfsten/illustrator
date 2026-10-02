# Design

## Context

See proposal.md — Why. linux-rust has object-safe `Geometry`, `GeometryShell`,
`GeometryStore` (id resolution), `Mapper` (chart layout), and `IdGenerator`.
This change separates identity allocation and chart mapping without ECS
side-maps or splitting the shell.

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

### 2. Chart layout via `Mapper` (stateless); store resolves only

```text
g = store.expect_geometry(id)          // identity
Mapper::unpack_chart::<C>(g, p)       // layout + State window
Mapper::chart_param_block(g)
Mapper::globalize_indices(g, local)   // shell.param_offset + local
```

`GeometryStore` is id→geometry only. `Mapper` is a stateless namespace that reads
`GeometryShell` layout (contiguous block today). State is passed per unpack call.
Kinds expose formula-local indices; expressions lift via `globalize_indices`.

Ephemeral `(store, state)` views remain rejected (misleading ownership). A
long-lived Mapper holding Document is unnecessary while blocks stay contiguous
on the shell — the type still names the layout seam.

**Alternatives:** Methods only on GeometryStore (rejected — mixes resolution with
layout; `globalize_indices` belonged nowhere); unpack keeps id (rejected —
duplicate lookup).

### 3. Retain GeometryShell

Single companion bag remains. Document writes id+offset(+role) on register; Mapper reads offset via `g.param_block()` / `shell().param_offset`. Reindex-on-remove stays on shell fields via `Geometry::reindex_after_removal`.

### 4. Stability note (documentation only)

`Box<dyn Geometry>` keeps heap pointees address-stable across `Vec` growth, but safe `&dyn Geometry` must not outlive structural mutation. Durable handles remain `GeometryId`.

## Risks / Trade-offs

- **[Risk] Conflict with `geometry-object-safe-shell` deltas on geometry-ir** → Mitigation: different requirements; sync/archive carefully.
- **[Trade-off] Mapper as type vs methods only** → Chose a zero-sized `Mapper` namespace so layout lives beside store resolution without an ephemeral view.
- **[Risk] Over-abstracting IdGenerator** → Keep it a tiny struct in `shell.rs` or `document.rs`; no trait unless a second allocator appears.

## Migration Plan

1. Add `IdGenerator`; wire DocumentStructure.
2. Change `unpack_chart` / `chart_param_block` signatures to `&dyn Geometry`; fix `expr.rs`.
3. Introduce `Mapper` with unpack / param-block / `globalize_indices`; slim `GeometryStore` to resolution.
4. Update docs/config; run `cargo test -p constraint_runtime`.

## Open Questions

None.
