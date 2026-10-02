# Tasks

## 1. Identity allocator

- [x] 1.1 Add `IdGenerator` (`allocate` / `observe`) and replace `DocumentStructure.next_id` usage; verify add/import paths still assign non-colliding ids (unit test or existing document tests)
- [x] 1.2 Ensure `Document::add_geometry` assigns id via the generator when building `GeometryShell`; verify chart pack is not given an id

## 2. Resolve-then-unpack API

- [x] 2.1 Change `GeometryStore::chart_param_block` / `unpack_chart` to take `&dyn Geometry` (drop id lookup inside unpack); update `expr.rs` to `expect_geometry` then unpack; verify `cargo test -p constraint_runtime` passes
- [x] 2.2 Add zero-sized `Mapper` (param block, unpack_chart, globalize_indices); `GeometryStore` stays id→geometry only; `expr` uses Mapper for layout lifts

## 3. Docs and validation

- [x] 3.1 Align `docs/constraint-mathematics.md` and `openspec/config.yaml` context with IdGenerator + resolve-then-unpack (shell unchanged)
- [x] 3.2 Run `openspec validate geometry-id-mapper-soc` and `cargo test -p constraint_runtime` (and sketch_ui check if API exports change)
