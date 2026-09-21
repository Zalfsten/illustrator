# Tasks

## 1. Mesh core (constraint_runtime)

- [ ] 1.1 Add `src/mesh/{mod.rs,tests.rs}` with `Mesh` / `MeshEdge` (`geometry: GeometryId`, endpoints `Point2`); verify module compiles and is re-exported from `lib.rs`
- [ ] 1.2 Implement `Mesh::from_structure(structure, p)` mapping each `Geometry::Segment` to one edge (start/end from state); verify unit test: one segment → one edge with matching coordinates and id
- [ ] 1.3 Implement `hit_endpoint(query, radius) -> Option<(GeometryId, EndpointKind)>` (nearest Mesh endpoint within radius; start=`a`, end=`b` for segment edges); verify unit tests for hit near start, near end, and miss
- [ ] 1.4 Implement `hit_geometry(query, radius) -> Option<GeometryId>` (nearest edge by distance to finite segment); verify unit test hit near edge midpoint and miss

## 2. sketch_ui rewire

- [ ] 2.1 Paint Canvas strokes from `Mesh::from_structure` edges instead of `doc.segments()`; verify `cargo build -p sketch_ui`
- [ ] 2.2 Route Join endpoint pick through Mesh `hit_endpoint`; Parallel/Orthogonal geometry pick through `hit_geometry`; verify the sample right-angle sketch still runs (manual or existing flow)
- [ ] 2.3 Remove paint/hit uses of `Document::segments()`; keep `add_segment` / construction tools; verify `cargo test -p constraint_runtime` and `cargo build -p sketch_ui`

## 3. Docs / glossary (light)

- [ ] 3.1 Add a short Mesh glossary note to `docs/constraint-mathematics.md` (derived view, not authoritative State); verify terms align with `display-mesh` / architecture deltas
