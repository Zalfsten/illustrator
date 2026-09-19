# Proposal

## Why

Illustration tools (xfig, tgif) lose geometric relationships on edit; CAD sketchers keep constraints but force rigid drafting conventions unsuitable for teaching figures. Project principles in `openspec/config.yaml` define the long-term product (constraint-backed scientific illustration; one shared spec, many implementations). This change is the first vertical slice under those principles: a constraint runtime where segments stay related under mouse drag — without labels, styles, export, or function graphs yet.

## What Changes

- Introduce **Model E** as the shared geometry ontology: geometries are first-class and parameterized; derived locations are `PointExpr` objects; relationships are equation factories — not a point-sharing document model at the core.
- Define a **v1 constraint catalog**: Congruence (UI-shared endpoints), Parallel, Orthogonal, and FixPoint, each with explicit residual arity and meaning.
- Define **interactive solve semantics**: structural constraints are hard invariants; drag is a soft mouse target plus preference to stay near the previous state; failed solves keep the last consistent document state; pointer-down…up is one edit.
- Specify a **minimal sketch session** sufficient to construct segments, apply those constraints, and drag logical UI points without the figure falling apart.
- Treat the numerical solver as an **owned, swappable backend** (algorithms and future GPU offload are design concerns, not shared requirements).
- Explicitly defer illustration features: function graphs, LaTeX labels, styles, export, circles/splines, equal-length, and third-party geometry kernels as the document API.

## Capabilities

### New Capabilities

- `geometry-ir`: Model E intermediate representation — parameterized geometries, `Endpoints` / `PointExpr`, equation objects, and UI shared-point desugaring to Congruence.
- `constraint-catalog`: Persistent v1 constraints (Congruence, Parallel, Orthogonal, FixPoint), residual semantics, and structural consistency.
- `solve-interaction`: Solve and drag contract (hard vs soft conditions, stay-near-previous, failure handling, atomic drag edits, logical UI points).

### Modified Capabilities

- (none — greenfield; no existing specs)

## Non-goals (relative to product vision)

- Full illustration surface (styles, LaTeX labels, export, function graphs, freeform bodies) — later changes under the same project principles.
- Choosing one language/OS as the only product — specs stay shared; stacks live in per-impl design.

## Impact

- Greenfield planning only: no application code exists yet beyond OpenSpec scaffolding and the devcontainer.
- Specs produced here become shared capability contracts per `openspec/config.yaml`; the first impl stack is recorded only in this change's `design.md`.
- Parallel implementations later should reuse the same IR and solve contract; solver backends may differ.
