# Tasks

## 1. Chart write path

- [ ] 1.1 Add `Chart` trait (init, body, pack) under `geometries/` and implement it for `Segment`; verify the crate compiles with the trait in place
- [ ] 1.2 Add `Document::add_chart` (and optional `add_segment` sugar); demote public `add_geometry` / `push_geometry` / `into_geometry` per design; verify document unit tests for add still pass with the new API
- [ ] 1.3 Migrate `constraint_runtime` tests, acceptance, and `sketch_ui` call sites to `add_chart`/`add_segment`; verify `cargo test -p constraint_runtime -p sketch_ui` compiles and existing add/remove tests pass

## 2. Parameter-window read path

- [ ] 2.1 Add `Geometry::param_block()` and derive connectivity (and any kept index helpers) from it; verify geom unit tests pass
- [ ] 2.2 Switch segment formulas/helpers from ad-hoc `segment_offset` to the structure `param_block` window; verify segment and constraint Jacobian/FD tests pass

## 3. Docs and verification

- [ ] 3.1 Update `docs/constraint-mathematics.md` and `openspec/config.yaml` with Document-as-sync / Chart-pack / param-window wording; verify the docs render and config still parses
- [ ] 3.2 Run full `cargo test -p constraint_runtime -p sketch_ui` and confirm all tests pass (including acceptance)
