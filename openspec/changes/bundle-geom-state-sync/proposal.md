# Proposal

## Why

Geometry kinds, DocumentStructure registration, and State layout currently stay in sync only by convention: callers pack initial parameters, Document extends State and sets offsets, and evaluate re-resolves those offsets through ad-hoc helpers. That scattered discipline is fragile as more kinds and structural edits land. Bundling structural mutation and parameter-window access keeps Model E (structure ⊥ values) while making the Document composite the single authority that keeps them matched — a prerequisite for reliable multi-geometry sketches under constraints.

## What Changes

- **BREAKING** (linux-rust public API): adding a geometry goes through a kind-owned chart init API on the Document composite (pack + register + State extend in one step). Callers no longer pass a raw parameter slice beside a body discriminant as the supported path; low-level structure-only registration is not a public edit path.
- Structural add/remove of geometries SHALL keep DocumentStructure and State length/connectivity matched atomically via the Document composite (spec clarification of existing intent).
- Evaluation of kind formulas SHALL obtain each geometry's contiguous parameter window from the geometry record in DocumentStructure (offset + count), not from a parallel caller-managed layout table.
- Docs/config note the Document composite as the structural sync point; kinds own chart packing and formula catalogs only.

## Capabilities

### New Capabilities

_(none)_

### Modified Capabilities

- `state-vector`: Require atomic structural edits through the Document composite; clarify that evaluation reads the geometry's parameter window from the structure record.
- `geometry-ir`: Require that kind chart evaluation resolves parameters via the geometry's stored contiguous block (window from DocumentStructure), not a separate layout authority.

## Impact

- linux-rust `constraint_runtime`: Document add API, Chart trait on kinds, `Geometry` parameter-window helper, segment evaluate helpers; tighten visibility of `push_geometry` / kind `into_geometry`.
- Call sites: unit/acceptance tests, `sketch_ui` geometry creation.
- Specs: deltas under `state-vector` and `geometry-ir`; short docs/`openspec/config.yaml` wording.
- Shared specs remain implementation-agnostic; Chart/Document naming in design is linux-rust detail.

## Non-goals

- Redesigning opaque point/vector expression catalogs or constraint residual math.
- Changing SolverPort / Interact working-copy commit semantics.
- New geometry kinds, illustration styling, LaTeX, or export.
- Relaxing contiguous-block layout (already deferred in `state-vector`).
