# Proposal

## Why

The product now has working constraint-runtime specs and a Rust spike, but the overarching component model (Document, App/Canvas, Interact, Solver) and the mathematics of residuals/assembly are only implicit. We need an explicit structure-vs-state split and shared vocabulary so later geometries, views, and implementations stay aligned.

## What Changes

- Introduce a shared **system-architecture** capability: Document, App, Canvas, Interact, SolverPort relationships; View/Mesh deferred until non-segment geometry needs tessellation.
- **BREAKING** (conceptual / API for the Rust impl): split committed sketch state into **DocumentStructure** (geometries, constraints, connectivity schema) and **State** \(p\); Document is the composite of both. Eval and Interact use an explicit \(p\) (committed or working copy).
- Modify **state-vector** requirements to match that split and structural add/delete rebuild rules for \(p\).
- Add repository documentation under `docs/` with a mathematical description and glossary (not a runtime capability).
- Align the linux-rust `Document` / Interact APIs with the new contracts (implementation tasks).

## Capabilities

### New Capabilities

- `system-architecture`: Top-level component contracts — Document (structure + committed state), App, Canvas, Interact, SolverPort; role of display tessellation (Mesh/View) as deferred view layer.

### Modified Capabilities

- `state-vector`: Explicit DocumentStructure vs State \(p\); Document owns the composite; connectivity belongs to structure; structural edits rebuild/remap \(p\); evaluation always takes an explicit \(p\).

## Impact

- Main specs after archive: new `system-architecture`, updated `state-vector`.
- New `docs/` math/glossary document referenced from README.
- Rust `constraint_runtime` Document API and possibly Interact call sites; tests updated for structure/state split.
- No change to constraint catalog semantics or solver algorithm choice.
