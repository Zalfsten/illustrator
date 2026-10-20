# Proposal

> **Archive note (2026-10-20):** Abandoned (0/5 tasks). The implementation moved the other way, to a closed geometry kind sum (`GeometryKind` + `with_kind!`), which the main `geometry-ir` spec describes. Delta specs were **not synced**.

## Why

Document geometry storage in the shared specs still describes a **closed sum of kinds** (kind discriminant + kind-specific payload on each record). The proven linux-rust design instead uses a **shared shell** (identity, role, State block) plus an **object-safe instance contract**, with kinds owning charts and formula catalogs. Specs must catch up so new kinds and parallel implementations follow the same contracts—without treating one language’s trait objects as the product.

## What Changes

- **BREAKING (spec language):** Document SHALL store geometries as instances under a uniform instance contract over shared shell metadata, not as a mandatory closed kind-sum enum on the evaluate/solve path.
- Clarify that **shared shell** (stable id, role, contiguous parameter window) is kind-agnostic; **chart pack/unpack and formula catalogs** remain kind-owned.
- Keep **opaque point/vector expressions** (closed formula catalog behind a facade) as the evaluate/solve surface; kind matching for Mesh tessellation remains allowed where a discriminant is required.
- Record that structural add goes through Document with kind chart init → State block → shell-backed instance registration (`add_geometry`-style), not a privileged segment-only store API.

## Non-goals

- New curve kinds (circle, spline, graphs), styles, LaTeX, or export.
- Replacing opaque expressions with trait-object formulas (Model B stays).
- Changing State contiguous-block layout, constraint catalog semantics, or Interact/SolverPort contracts.
- Mandating Rust trait objects in shared specs—only the domain contracts (uniform instance API + shared shell + kind-owned charts).

## Capabilities

### New Capabilities

_(none)_

### Modified Capabilities

- `geometry-ir`: Replace “sum of kinds / closed sum payload” storage requirements with shared-shell + uniform instance contract; keep curve/role/expression contracts otherwise.

## Impact

- Main specs under `openspec/specs/geometry-ir/` after sync.
- linux-rust already matches (`GeometryShell`, `Geometry` trait, `HashMap<GeometryId, Box<dyn Geometry>>`, `Document::add_geometry`); apply phase is primarily **spec sync + verify**, not a greenfield rewrite.
- Docs (`docs/constraint-mathematics.md`) and `openspec/config.yaml` context already describe this layout; keep them aligned when archiving.
