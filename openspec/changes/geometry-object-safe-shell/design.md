# Design

## Context

See proposal.md — Why. linux-rust already implements shared shell + object-safe geometry storage; this change aligns shared OpenSpec language and records the impl layout so later kinds and other stacks do not reintroduce a document-wide kind enum as the evaluate/solve storage model.

## Goals / Non-Goals

**Goals:**

- Spec language: uniform instance contract + kind-agnostic shell; kind-owned charts/formulas.
- linux-rust design of record: `GeometryShell`, `Geometry` trait, `Chart`, `Box<dyn Geometry>`, `Document::add_geometry`.
- Keep Model B opaque expressions (closed private formula catalog).

**Non-Goals:**

- New geometry kinds or Mesh tessellation strategies beyond the current segment path.
- Changing SolverPort / Interact / State contiguous-block contracts.
- Mandating Rust trait objects in shared specs.

## Decisions

### 1. Shared shell + object-safe instance (document storage)

**Choice (linux-rust):** Each kind instance embeds/wraps `GeometryShell` (`id`, `role`, `param_offset`) and implements `Chart`; `geometry_shell_instance!` expands local `AsShell` + `From<GeometryShell>` (orphan rules block blankets over `Deref`/`From`). Blankets supply marker `GeometryKind` and object-safe `Geometry`. `Document::add_geometry` is bounded on `GeometryKind`. `Document` holds `HashMap<GeometryId, Box<dyn Geometry>>`.

**Why:** Evaluate/solve and Document ops (role, param block, reindex, parameter count) need a uniform API without a growing `GeometryBody` enum. Kind knowledge stays in chart pack/unpack and formula factories.

**Alternatives considered:**

| Approach | Rejected because |
|----------|------------------|
| Closed `GeometryBody` / kind enum in Document | Every new kind touches Document storage and central matches |
| Store only shell + separate kind tables | Splits identity from chart instance; harder atomic add/remove |
| Trait-object formulas instead of Model B exprs | Already rejected for Eq/Debug/serde and sparse Jacobian clarity |

### 2. Structural add via chart init

**Choice:** `Document::add_geometry::<C>(role, init)` packs `C::pack(init)`, extends State, registers `C::from(GeometryShell::new(...))`. Remove compacts State and reindexes shells.

**Why:** Matches state-vector atomic structural edits; kinds never mutate State alone.

### 3. Expressions stay Model B

**Choice:** Opaque `PointExpr` / `VecExpr` with private per-kind formula enums and an outer kind sum for dispatch. Document storage openness does **not** imply expression openness.

**Why:** Constraints must not match formula kinds; data-driven catalogs remain preferable for this slice.

### 4. Mesh may still discriminate kinds

**Choice:** Mesh tessellation may downcast or match kind-specific factories (segment endpoints today). Specs already allow kind matching where a discriminant is required.

## Risks / Trade-offs

- **[Risk] Spec language too Rust-specific** → Mitigation: requirements speak of “uniform instance contract” and “shared shell metadata,” not `dyn` / `Box`.
- **[Risk] Mesh still kind-specialized while Document is open** → Mitigation: acceptable until multi-kind tessellation; document in Mesh code, not as a Document kind enum.
- **[Trade-off] Cloning Document** → requires `clone_box` on the instance contract (already present).

## Migration Plan

1. Sync delta into main `geometry-ir` spec (`/opsx-sync` or archive).
2. Confirm linux-rust tests still green (code already matches).
3. Archive this change; no runtime migration.

## Open Questions

None for this slice.
