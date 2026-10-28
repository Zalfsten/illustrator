# Proposal

## Why

Geometry kinds opt into capabilities (curve, endpoints, line, circular) statically. Code that holds only a geometry identity, such as GUI tools after a pick or the display Mesh iterating the document, cannot ask "what can this geometry do?" without a hand-maintained per-kind list in the central catalog. Even after that check it has to name a concrete kind to build an expression (the Parallel tool calls the segment's direction factory for any picked id). The Mesh also hard-requires the endpoints capability for every kind, so a full circle (admitted by the spec as a future curve kind) could not be tessellated.

This blocks the per-kind plugin structure (GitHub #21, this change implements #22): adding a kind should not mean editing central lists, and tools should depend on what a geometry can do, not on which kind it is. That keeps the product's constraint tools (parallel, concentric, equal radius, joins) applicable to every current and future curve kind that has the needed invariant.

## What Changes

- **Capability query by identity:** for a geometry identity, the system answers which capabilities its kind has (curve, endpoints, line, circular).
- **Identity-based capability factories:** the system builds capability-scoped expressions (start/end, direction, center, radius) from a geometry identity alone and reports "not available" when the kind lacks the capability, instead of requiring the caller to name the kind.
- **Capabilities are declared once, with the kind:** a kind's capability set and its capability factories come from a single declaration in the kind's own code; there is no separate central list to keep in sync.
- **Tools decide applicability by capability:** the Parallel/Orthogonal and Join tools (and later tools) check capabilities and use identity-based factories; they no longer name a concrete kind.
- **Mesh does not require endpoints:** tessellation samples the curve capability; endpoint hit targets are produced only for geometries with the endpoints capability. A curve kind without endpoints can be meshed and edge-picked.
- Removal of the per-kind runtime helpers on the kind tag (`has_line_exprs`, `has_circular_exprs` in the Rust implementation). Internal API only, no user-visible behavior change.

## Capabilities

### New Capabilities

- (none)

### Modified Capabilities

- `geometry-ir`: add a requirement for capability queries and identity-based capability factories (ADDED); rewrite the kind-dispatch sentence in "Geometry set includes curves with segment as proving kind" so callers query capabilities instead of matching on kind (MODIFIED).
- `display-mesh`: edges come from curve position expressions rather than requiring endpoint expressions; geometries without the endpoints capability contribute edges but no endpoint hit targets (MODIFIED "Mesh exposes drawable edges tagged with geometry identity", "Mesh supports endpoint hit queries").

## Non-goals

- New geometry kinds (full circle, splines). The change only removes the obstacle; no kind without endpoints is added, so the endpoint-free Mesh path is covered by a test-only kind or by unit tests on the capability-gated code path.
- Generic conformance tests across kinds (#23), per-layer kind traits for GUI/IO/command API (#24) and the per-kind directory layout with a single registration list (#25).
- Capability-aware picking UX (highlighting only applicable geometries). Tools keep today's feedback text.

Specs stay shared across implementations. How capabilities are declared (Rust macro, associated constant, option-returning factories) is a design decision of each implementation.

## Impact

- `impls/rust/crates/constraint_runtime`: `geometry/capabilities.rs` (capability set type, declaration macro), `geometry/formula.rs` (`Geometry` gains the capability set), `geometry_kinds/{catalog,segment,arc}.rs` (declarations replace hand-written marker impls and `has_*` helpers), `document.rs` (identity-based factories), `mesh.rs` (no `EndpointExprs` bound in the kind dispatch), tests.
- `impls/rust/crates/sketch_ui/src/app.rs`: tools use capability queries and identity-based factories.
- Solver hot path: unchanged (capability sets are compile-time constants; expression evaluation is untouched).
- Specs: deltas for `geometry-ir` and `display-mesh`.
