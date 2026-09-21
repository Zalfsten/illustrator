# Tasks

## 1. Document role (constraint_runtime)

- [ ] 1.1 Add `GeometryRole::{Result, Construction}` and store it on `Segment` (accessible via `Geometry`); default `Result` in `add_segment`; verify unit test round-trips role on add
- [ ] 1.2 Ensure `remove_geometry` / reindex unchanged; verify existing document tests still pass
- [ ] 1.3 Re-export `GeometryRole` from `lib.rs`; verify `cargo test -p constraint_runtime` compiles

## 2. Mesh filter

- [ ] 2.1 Add `MeshFilter { include_construction: bool }` and thread through `Mesh::from_structure`; omit construction geometries when false; verify unit test: construction edge absent in result-only mesh, present when included
- [ ] 2.2 Verify `hit_endpoint` / `hit_geometry` cannot return a construction id when meshed result-only (unit test)

## 3. sketch_ui

- [ ] 3.1 Add presentation mode toggle (result only vs include construction); rebuild Mesh and logical_points with the same filter; verify `cargo build -p sketch_ui`
- [ ] 3.2 When construction is shown, draw construction edges with a distinct stroke (e.g. lighter/dashed); verify build succeeds
- [ ] 3.3 Optional demo: mark one sample segment as construction OR add a way to set role; if no UI to set role yet, add a unit/acceptance-level fixture only — document which was chosen in the PR/commit message

## 4. Glossary

- [ ] 4.1 Add construction role / presentation mode notes to `docs/constraint-mathematics.md`; verify terms match the deltas
