# Tasks

## 1. Identity allocator

- [ ] 1.1 Add `IdGenerator` (`allocate` / `observe`) and replace `DocumentStructure.next_id` usage; verify add/import paths still assign non-colliding ids (unit test or existing document tests)
- [ ] 1.2 Ensure `Document::add_geometry` assigns id via the generator when building `GeometryShell`; verify chart pack is not given an id

## 2. Resolve-then-unpack API

- [ ] 2.1 Change `GeometryStore::chart_param_block` / `unpack_chart` to take `&dyn Geometry` (drop id lookup inside unpack); update `expr.rs` to `expect_geometry` then unpack; verify `cargo test -p constraint_runtime` passes
- [ ] 2.2 Optionally add a thin `Mapper<'a>` view bundling store+State if it clarifies call sites without extra nesting; otherwise document that store methods suffice and skip the type

## 3. Docs and validation

- [ ] 3.1 Align `docs/constraint-mathematics.md` and `openspec/config.yaml` context with IdGenerator + resolve-then-unpack (shell unchanged)
- [ ] 3.2 Run `openspec validate geometry-id-mapper-soc` and `cargo test -p constraint_runtime` (and sketch_ui check if API exports change)
