# Tasks

## 1. Components and consistency (`constraint_runtime`)

- [x] 1.1 Add `src/components.rs` with `Component`, `Document::components` and `Document::component_of` (D1: graph walk over `constraints_on` / `referenced_geometries`, id-ordered lists, components ordered by smallest geometry id); verify with `src/tests_components.rs` covering every scenario of "Components of the constraint graph are enumerable" (joined segments, an unconstrained geometry on its own, removing the only link splits, coincident unconstrained segments stay separate)
- [x] 1.2 Add `COMMIT_TOL`, `Document::component_residual` and `Document::is_consistent` (D2), and remove the unused `assemble::is_structurally_satisfied`; verify with `tests_components.rs`: a FixPoint off its target beyond `COMMIT_TOL` makes its component inconsistent while a satisfied component next to it stays consistent, and the residual equals the global structural norm when the drawing has one component

## 2. Scoped solve and drag (`constraint_runtime`)

- [x] 2.1 Add the constraint scope to `AssembledSystem` (D3; `None` = all constraints, unchanged behaviour); verify with `tests_assemble.rs` that a scoped system has the residual length of the scoped constraints only and that the unscoped tests still pass
- [x] 2.2 Add `InteractiveSolve::solve_scoped` / `structural_scoped` taking a `&Component` (the whole-document entry points delegate with `Document::whole()`), assemble with its constraints, gate with `component_residual` against `COMMIT_TOL`, and copy only its geometries' parameter windows on success or from the fallback on failure (D3); verify with `tests_interact.rs`: in a drawing with a conflicted and a consistent component, a structural solve of the consistent one succeeds, and the conflicted component's values are bit-identical afterwards, both on success and on a failing solve
- [x] 2.3 Make `DragSession::pointer_down` store the dragged point's component, scope pointer moves and pointer-up to it, and limit `soft_for_drag` stay-near terms to that component; verify with `tests_interact.rs` covering "Drag next to a conflicted component" (the endpoint moves, the conflicted component is bit-identical) and that the existing drag tests pass
- [x] 2.4 Update the `constraint_runtime` section of the `openspec/config.yaml` context (components module, `COMMIT_TOL`, scoped solve and drag); verify that the text names `components.rs` and the per-component commit gate

## 3. Session rules (`sketch_session`)

- [x] 3.1 Add `CommandError::Frozen { name }` with `Display`, and the frozen-operand check in the constrain path, after `build` and before `add_constraint`; solve the merged component (D4); verify with `tests_session.rs` from a conflicted document passed to `Session::new`: "Constraint inside a frozen component is rejected", "Joining to a frozen component is rejected", "Contradiction elsewhere does not block commands", "Rename inside a frozen component succeeds", and the existing "Contradicting constraint is rolled back" still fails with `unsatisfiable`
- [x] 3.2 Implement the delete rule (D4: note whether the target's component is consistent; after removal, run one solve per surviving piece of a conflicted component; no solve for a consistent one); verify with `tests_session.rs`: "Deleting the contradicting constraint unfreezes the component", "Deleting an unrelated constraint keeps the component frozen" (values bit-identical) and "Undo restores the conflict"
- [x] 3.3 Make `drag_begin` fail with `Frozen` on a point in a conflicted component; verify with `tests_session.rs` covering "Pointer-down on a frozen point" (no drag active, drawing and undo stack unchanged) and that a drag in a consistent component next to it works
- [x] 3.4 Make `Session::new` run one structural solve per inconsistent component, and add `Session::conflicts()` returning `Conflict { geometries, constraints, residual }` (D4); verify with `tests_session.rs` covering "Solvable and contradicting components at start" (no undo available) and "One conflicted component is listed", and an empty list for the sample-style consistent drawing
- [x] 3.5 Update the `sketch_session` paragraph of the `openspec/config.yaml` context (frozen components, `Frozen`, `conflicts()`, solve at session start); verify that the text names the `frozen` class and `Session::conflicts`

## 4. Integration

- [x] 4.1 Run `cargo test --workspace`, `cargo clippy --workspace --all-targets` and `cargo fmt --check -p sketch_session` from `impls/rust`, and `scripts/check-wasm.sh`; verify that all pass and that `sketch_ui` builds without source changes
- [x] 4.2 Run `openspec validate conflicted-components --strict`; verify it reports the change as valid

## Workflow follow-up

- After merge: close #51, note in #8 that conflicted files can now be held (`Session::new` start-up solve), and note in #52 that `Session::conflicts()` and the `frozen` class are ready for the red display.
- Archive the change with `/opsx:archive`.
