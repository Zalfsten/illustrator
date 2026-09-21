# Design

## Context

`sketch_ui` paints and picks via `Document::segments()` / `Segment::{start,end}`. Specs previously deferred Mesh until non-segment tessellation. This change introduces Mesh now so the Canvas is kind-agnostic and hit-testing matches stroked primitives (WYSIWYG), tagged with `GeometryId`.

Target: Linux Rust `constraint_runtime` + `sketch_ui`. Shared specs: new `display-mesh`, update `system-architecture`.

## Goals / Non-Goals

**Goals:**

- Mesh build from `&DocumentStructure` + `&[f64]` (State).
- Edges: two `Point2` + `GeometryId`; for segments, also endpoint kinds on the two vertices for hit → `EndpointRef`.
- Hit: nearest endpoint within radius; nearest edge (distance to segment) within radius → `GeometryId`.
- `sketch_ui` paint + Join/Parallel/endpoint pick use Mesh; drop `segments()` from those paths.

**Non-Goals:**

- Adaptive tessellation, styles on Mesh, BVH, Mesh inside Document, Interact depending on Mesh.
- Replacing `logical_points` / congruence drag entirely with Mesh vertices.
- Renaming construction `add_segment` / `Tool::Segment`.

## Decisions

### 1. New capability `display-mesh` + architecture delta

**Choice:** New capability for Mesh contracts; `system-architecture` removes “deferred tessellation” and requires Canvas to paint from Mesh.  
**Why:** Mesh behavior is its own concern; architecture only places Mesh relative to Document/App/Canvas.  
**Alternative:** Only modify architecture — rejected (too little detail for hit semantics).

### 2. Mesh owns paint + edge/endpoint hits; logical points stay separate

**Choice:** Mesh hit covers geometry edges and geometric endpoints. Drag/FixPoint logical points remain expression-based (congruence collapsing).  
**Why:** WYSIWYG for stroked geometry; logical points are IR sugar, not Mesh vertices.  
**Alternative:** Mesh-only picks for everything — rejected for this slice (breaks congruence-aware handles).

### 3. Rust: `mesh` module in `constraint_runtime`

**Choice:** `Mesh { edges: Vec<MeshEdge> }` with `MeshEdge { geometry: GeometryId, a: Point2, b: Point2 }` (segment: `a`=start, `b`=end). `Mesh::from_document(structure, p)`, `hit_endpoint`, `hit_geometry`. Module layout `src/mesh/{mod.rs,tests.rs}`.  
**Why:** Toolkit-free; UI only consumes Mesh.  
**Alternative:** Mesh only in `sketch_ui` — rejected (other UIs would reimplement; specs are shared).

### 4. Rebuild policy

**Choice:** Rebuild Mesh each paint/frame (and before hit on that frame) from committed or displayed State; no persistent cache invalidation yet.  
**Why:** Segment count is tiny; avoids stale Mesh bugs.  
**Alternative:** Cached Mesh on App — defer until cost matters.

### 5. Segment tessellation = one edge

**Choice:** `Geometry::Segment` → one `MeshEdge`. Future curves append polylines with the same `GeometryId`.  
**Why:** Satisfies architecture with current geometry set.  
**Alternative:** Sample curves with \(N\) chords now — unnecessary without non-segment kinds.

## Risks / Trade-offs

- [Logical points vs Mesh endpoints] → Join uses Mesh endpoint hit; Select/FixPoint keep `logical_points`. Document in UI comments.
- [Linear hit scan] → Fine for v1; BVH later under same hit API.
- [Ordering of edges] → Unspecified; hits use distance, not order.

## Migration Plan

1. Land proposal/specs/design/tasks.  
2. Apply: mesh module + tests; rewire sketch_ui; keep `segments()` only if still needed for construction helpers.  
3. Archive → merge into main specs.  
4. Rollback: revert change; UI can temporarily restore segment iteration.

## Open Questions

- None blocking; endpoint hit returns `(GeometryId, EndpointKind)` matching `EndpointRef`.
