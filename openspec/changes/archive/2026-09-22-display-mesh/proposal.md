# Proposal

## Why

The Canvas currently paints and picks by iterating line segments on the Document. That couples presentation to one geometry kind and blocks future curves without rewriting the UI. A display Mesh derived from DocumentStructure plus State gives the Canvas kind-agnostic edges and—when tagged—WYSIWYG hit-testing against the same primitives that are drawn.

## What Changes

- Introduce a **display Mesh**: a view-layer tessellation of the sketch into drawable edge primitives, built only from DocumentStructure + a concrete State \(p\). The Mesh is not a second source of truth for parameters or constraints.
- Make the Mesh **hit-testing capable**: edges and endpoints carry stable geometry identity (and endpoint kind where applicable) so pick queries return the same identities tools already use (`GeometryId`, endpoint refs / point expressions).
- **MODIFIED** architecture: Canvas SHALL paint from the Mesh (or an equivalent derived edge list), not by matching geometry kinds. Lift the “Mesh deferred until non-segment geometry” gate so Mesh is required for display decoupling even while only segments exist.
- Linux Rust: add Mesh build + hit helpers in `constraint_runtime`; rewire `sketch_ui` paint and Join/Parallel/endpoint picks through Mesh. Construction tools that *create* segments (`add_segment`) may remain segment-shaped for this slice.

## Capabilities

### New Capabilities

- `display-mesh`: Display tessellation of DocumentStructure+State into edge primitives; Mesh rebuild rules; hit queries that return geometry/endpoint identities consistent with paint.

### Modified Capabilities

- `system-architecture`: Display tessellation is no longer deferred; Canvas derives appearance (and may derive picks) from a Mesh/view layer rather than kind-specific Document iteration. Mesh remains outside Document/Interact as authoritative state.

## Non-goals

- Circle/spline tessellation quality, adaptive flatness, GPU buffers, style/stroke attributes on Mesh.
- A full View cache with invalidation graph; spatial indexes (BVH) beyond linear scan.
- Replacing logical-point / congruence-aware drag targets solely with Mesh vertices (drag may still use point expressions).
- New construction tools beyond segment; illustration styling; LaTeX; export.
- Putting Mesh inside DocumentStructure or making Interact depend on Mesh.

## Impact

- Shared specs: new `display-mesh`; update `system-architecture` tessellation requirement.
- Linux Rust `constraint_runtime`: new mesh module (build + hit); exports for UI.
- `sketch_ui`: paint and geometry/endpoint hit paths use Mesh; drop `doc.segments()` from those paths.
- Other implementations consume the same specs with their own design.
