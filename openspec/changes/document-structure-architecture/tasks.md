# Tasks

## 1. Documentation

- [x] 1.1 Create `docs/constraint-mathematics.md` with glossary (DocumentStructure, State \(p\), point expression, residual, local Jacobian, connectivity, assembly, soft contribution, working copy, commit) and the mathematical formulation of local maps, constraints, assembled \(r(p)\)/\(J(p)\), and Interact soft+hard solve; verify the file renders as valid Markdown with math delimiters
- [x] 1.2 Link `docs/constraint-mathematics.md` from the root `README.md` and verify the relative link path is correct

## 2. Runtime structure/state split

- [x] 2.1 Introduce `DocumentStructure` and `State` (or equivalent newtype) in `constraint_runtime` and make `Document` a composite; verify existing unit tests compile against the new types
- [x] 2.2 Change geometry eval and assembly entry points to take explicit structure + `&[f64]` / State; verify `cargo test -p constraint_runtime` passes
- [x] 2.3 Update Interact to use a working State copy and commit only on success; verify acceptance tests for drag/fail-keep-state still pass
- [x] 2.4 Add unit tests for add-segment (State grows, connectivity valid) and remove-segment (State compacted, offsets remapped, orphan constraints dropped); verify those tests pass

## 3. UI alignment

- [x] 3.1 Update `sketch_ui` to the Document composite API and keep naming aligned with App/Canvas roles where low-cost; verify `cargo build -p sketch_ui` succeeds

## 4. Spec hygiene

- [x] 4.1 Run `openspec validate --change document-structure-architecture` and fix any reported issues in change artifacts
