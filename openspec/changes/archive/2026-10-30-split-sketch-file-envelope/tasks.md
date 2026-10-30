# Tasks

## 1. Split the envelope requirement

- [x] 1.1 Sync the delta into `openspec/specs/sketch-file/spec.md`. Verify:
  - every statement of the old envelope requirement appears in exactly one of the three requirements;
  - every scenario is unchanged;
  - `openspec validate sketch-file --type spec --strict` passes.
- [x] 1.2 Verify that the existing file tests still pass unchanged, since no behaviour changes: `cargo test -p sketch_session file::`.

## Workflow follow-up

- Archive the change once the spec is synced.
