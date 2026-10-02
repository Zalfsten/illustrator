# Tasks

## 1. Sync shared specs

- [ ] 1.1 Sync this change's `geometry-ir` delta into main `openspec/specs/geometry-ir/spec.md` (via `/opsx-sync` or equivalent merge) and verify the two MODIFIED requirements no longer require a document-wide sum-of-kinds enum
- [ ] 1.2 Run `openspec validate --specs` and verify it succeeds after the sync

## 2. Verify linux-rust already matches

- [ ] 2.1 Confirm DocumentStructure stores `Box<dyn Geometry>` / shell-backed instances as in design.md (spot-check `document.rs` + `geometries/geometry.rs`)
- [ ] 2.2 Run `cargo test -p constraint_runtime` from `impls/linux-rust` and verify the suite passes

## 3. Close out

- [ ] 3.1 Archive this change after sync (`/opsx-archive`) and verify no active change remains for this name
