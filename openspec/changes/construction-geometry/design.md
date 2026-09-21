# Design

## Context

Agreed in exploration: construction helpers need a Document-level role; GUI toggles result-only vs include-construction; when not visible, not pickable; no layers in this slice. Mesh already owns paint + edge/endpoint hits.

Target: Linux Rust `constraint_runtime` + `sketch_ui`. Specs: deltas to `geometry-ir`, `display-mesh`, `system-architecture`.

## Goals / Non-Goals

**Goals:**

- `GeometryRole::{Result, Construction}` (or equivalent) on each geometry / segment in structure.
- `Mesh::from_structure(..., filter)` or `VisibilityFilter` excluding construction when requested.
- App toggle; paint + Mesh hits + logical_points use the same filter.
- Default new segments: Result (construction creation can be a follow-up tool option).

**Non-Goals:** Layers; export pipelines; styled construction strokes beyond a simple visual distinction when shown; auto-delete construction.

## Decisions

### 1. Role on geometry, not a parallel store

**Choice:** Field on `Segment` / available via `Geometry` match.  
**Why:** One identity space; constraints unchanged.  
**Alternative:** Side table `HashSet<GeometryId>` of construction — rejected (easy to desync on remove).

### 2. Single filter for Mesh and handles

**Choice:** `enum PresentationMode { ResultOnly, IncludeConstruction }` in App → `fn include_construction(mode) -> bool` → Mesh filter + logical_points rebuild.  
**Why:** Guarantees invisible ⇒ not pickable.  
**Alternative:** Separate pick allowlist — rejected (can diverge).

### 3. Mesh API takes filter

**Choice:** `Mesh::from_structure(structure, p, filter: MeshFilter)` with `MeshFilter { include_construction: bool }`. Hits only see built edges.  
**Why:** Toolkit-free; tests can assert omission.  
**Alternative:** Post-filter Mesh edges in UI — rejected (easy to forget on one call site).

### 4. No layers

**Choice:** Defer layers until a second grouping need appears.  
**Why:** Binary role is enough; layers would be speculative.

## Risks / Trade-offs

- [Join to hidden construction] → Impossible in result-only mode by design; user must enable construction mode.
- [Constraints involving only construction] → Still solved; result view just hides them — OK.
- [Old docs saying UI may iterate segments] → Already superseded by Mesh; this change filters Mesh instead.

## Migration Plan

1. Land specs/design/tasks.  
2. Apply: role field + Mesh filter + UI toggle + tests.  
3. Archive → merge main specs.

## Open Questions

- Whether `add_segment` gains an optional `role` parameter in this slice or only a later tool; default **Result** for all adds unless tasks say otherwise.
