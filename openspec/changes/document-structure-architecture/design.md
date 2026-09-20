# Design

## Context

See proposal.md for motivation. Today `impls/linux-rust` keeps `Document { p, segments, constraints }` as one struct; Interact clones `p` for drag. Specs already describe Model E, assembly, and soft drag, but not the App/Canvas/Document/Interact layering or an explicit structure/state split. No `docs/` math glossary exists yet.

## Goals / Non-Goals

**Goals:**

- Make DocumentStructure + State first-class in the Rust API and in shared specs.
- Capture the discussed component architecture in `system-architecture`.
- Add `docs/constraint-mathematics.md` (glossary + residual/assembly/interact formulation) linked from the root README.
- Keep egui/`sketch_ui` as a thin App; math and assembly stay in `constraint_runtime`.

**Non-Goals:**

- Implementing a Mesh/View cache or incremental tessellation.
- Changing the constraint catalog or swapping the LM backend.
- Multi-document or file I/O.
- Formal proof of solver convergence.

## Decisions

### 1. Types in `constraint_runtime`

- Introduce `DocumentStructure` (segments, constraints, id allocation, connectivity helpers) and `State` as `Vec<f64>` (or a newtype `State(Vec<f64>)`).
- `Document` becomes `{ structure: DocumentStructure, state: State }` with accessors; preserve a thin compatibility surface where practical (`doc.state()`, `doc.segments()` via structure).
- Eval/assembly APIs take `&DocumentStructure` + `&[f64]` (or `&State`) rather than only `&Document`.

**Alternatives:** Keep a single struct and only document the split in specs — rejected; user asked for explicit split. Put State only in the UI — rejected by architecture discussion.

### 2. Structural edit policy

- Add: append parameters to State; record `param_offset` on the geometry.
- Remove (when added): rebuild State by concatenating remaining geometries' parameter blocks; remap offsets; drop constraints that reference removed ids (or reject remove while referenced — prefer drop orphan constraints and document in tasks).

**Alternatives:** Freelist / sparse slots — deferred until scale demands it.

### 3. Math documentation location

- Path: `docs/constraint-mathematics.md`.
- Language: English technical prose with LaTeX math (`\( \)` / `\[ \]`), glossary of Structure, State, Point expression, Residual, Local Jacobian, Connectivity, Assembly, Soft contribution, Working copy, Commit.
- Specs stay behavioral; the doc is normative vocabulary for humans, not a second OpenSpec capability.

### 4. App naming in linux-rust

- Rename or alias the egui app type toward `App` where low-cost; Canvas remains the paint/input surface inside that app. No new crate.

## Risks / Trade-offs

- [API churn in rust impl] → Keep `Document` facade; migrate call sites in one change; acceptance tests first.
- [Remove-geometry remap bugs] → Unit tests for add/remove index integrity before UI hooks.
- [Docs drift from specs] → Short “see also” links between `docs/constraint-mathematics.md` and capability names; update docs in the same change as spec wording.

## Migration Plan

1. Land specs + `docs/` + README link (can ship before full API split).
2. Refactor `constraint_runtime` types and tests.
3. Update `interact` and `sketch_ui`.
4. Archive change with sync so main specs carry `system-architecture` and updated `state-vector`.

## Open Questions

- Exact remove semantics when constraints still reference a deleted geometry (auto-drop vs. refuse) — default to auto-drop constraints that reference the removed id unless tests show a better UX need.
