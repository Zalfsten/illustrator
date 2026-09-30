# Design

## Context

See proposal.md — Why. Today linux-rust callers use `Document::add_geometry(role, body, &Segment::params(...))`, tests can `push_geometry` without touching State, and segment formulas resolve `param_offset` via `segment_offset`. Structure and State remain correctly paired only by convention. Specs (`state-vector`, `geometry-ir` deltas) require atomic Document edits and evaluation through the geometry's recorded parameter window.

## Goals / Non-Goals

**Goals**

- One write authority: Document composite performs add/remove so Structure and State stay length-matched.
- Kinds own chart packing (`Chart`) and formula catalogs; shared shell exposes the contiguous parameter window.
- Evaluate path reads that window from the geometry record (no parallel layout table).

**Non-Goals (design-level)**

- New solver/Interact semantics; FFI trampoline shape.
- New geometry kinds beyond making Segment the first `Chart` implementor.
- Merging DocumentStructure and State into one type.

## Decisions

### 1. `Chart` trait on kinds; Document owns mutation

- Introduce `Chart` (linux-rust) with associated init type, body discriminant, and `pack(init) ->` fixed/contiguous initial scalars.
- `Document::add_chart<C: Chart>(role, init) -> GeometryId` packs, extends State, assigns `param_offset`, registers geometry.
- Optional thin `Document::add_segment(role, a, b)` for call-site ergonomics.
- **Why over keeping `add_geometry(body, &[f64])` as the public API:** the slice+body pair is exactly the split that desyncs; Chart makes the kind the pack authority and Document the register authority in one call.
- **Alternative considered:** keep public `add_geometry` and only document the invariant — rejected; does not fix call-site footguns.
- Demote `DocumentStructure::push_geometry` and `Segment::into_geometry` to `pub(crate)` (or test-only helpers that still go through Document).

### 2. Parameter window on the shared shell

- Add `Geometry::param_block() -> Range<usize>` (offset..offset+n).
- Segment formulas replace `segment_offset` with a block lookup (`expect_geometry` + `param_block` + body assert); chart helpers take block start (or range) consistently.
- Keep `connectivity()` derived from the same window; drop or derive redundant public index helpers (`all_param_indices`) from the block.
- **Why over putting sampling methods on `Geometry`:** shell stays kind-agnostic; kinds keep formulas (Model B / config).

### 3. Docs / config

- One paragraph in `docs/constraint-mathematics.md` and a short `openspec/config.yaml` context note: Document is the structural sync point; kinds pack and own formulas; evaluate uses the structure window.

## Risks / Trade-offs

- [Breaking rust API for add] → Update all crate call sites in the same change; acceptance + sketch_ui covered by tasks.
- [Tests that built Structure+Vec by hand] → Migrate to Document/`add_chart` or crate-private helpers that preserve the invariant.
- [Chart trait vs per-kind Document methods] → Trait scales to future kinds; Segment-only `add_segment` may coexist as sugar.

## Migration Plan

1. Land Chart + `add_chart` + `param_block`; migrate call sites; restrict `push_geometry`.
2. Switch segment evaluate helpers to `param_block`.
3. Update docs/config; run `cargo test -p constraint_runtime -p sketch_ui`.
4. Archive/sync specs when the change is complete (separate apply/archive workflow).

Rollback: revert the change branch; no on-disk document format yet.

## Open Questions

_(none — Chart+Document write path and param_block read path are fixed by this design.)_
