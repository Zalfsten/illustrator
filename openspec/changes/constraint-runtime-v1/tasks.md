# Tasks

## 1. Project scaffold

- [ ] 1.1 Add Rust workspace under `impls/linux-rust/` with a library crate (`constraint_runtime` or similar) and a binary crate for the UI; verify `cargo build` succeeds in that workspace
- [ ] 1.2 Ensure the Linux/devcontainer environment can compile the workspace (install or document Rust toolchain as needed) and verify `cargo test` runs with zero tests OK

## 2. Geometry IR (Model E)

- [ ] 2.1 Implement segment geometry with endpoint parameterization \((x_1,y_1,x_2,y_2)\) and `Endpoints` point expressions; verify unit tests that changing parameters changes evaluated endpoint locations
- [ ] 2.2 Implement document/parameter vector collection over geometries; verify a multi-segment document exposes a contiguous parameter slice suitable for solving
- [ ] 2.3 Implement UI shared-point desugaring that inserts Congruence between endpoint expressions; verify a join of two ends yields a congruence relationship in the core model (no authoritative core point entity required)

## 3. Constraint catalog residuals

- [ ] 3.1 Implement Congruence residuals (2 components); verify a unit test that congruent endpoints agree after a trivial solve or residual-only check at a satisfying configuration
- [ ] 3.2 Implement Parallel residual (1 component, 2D cross product); verify residual ≈ 0 for known parallel segments and ≠ 0 otherwise
- [ ] 3.3 Implement Orthogonal residual (1 component, dot product); verify residual ≈ 0 for known perpendicular segments and ≠ 0 otherwise
- [ ] 3.4 Implement FixPoint residuals (2 components); verify residual ≈ 0 only at the target location
- [ ] 3.5 Implement residual assembly into a global \(r(p)\); verify stacking arities match the catalog (2+1+1+2 for one of each) on a fixture document

## 4. Solver backend and interaction encoding

- [ ] 4.1 Define a replaceable solver backend trait/interface (`solve` over parameters + assembled system); verify a stub backend can be swapped in tests
- [ ] 4.2 Implement a CPU damped least-squares (LM-style) backend with finite-difference Jacobian; verify it drives a simple Congruence-only or FixPoint-only system to tolerance
- [ ] 4.3 Encode soft mouse-follow and stay-near-previous as temporary terms used only during interaction; verify they are not persisted on the document after a completed drag commit helper is called
- [ ] 4.4 Implement commit-on-success / keep-last-consistent-on-failure; verify a forced failing solve leaves document parameters unchanged and reports failure observably (Result/error flag)
- [ ] 4.5 Implement atomic drag session (snapshot on pointer-down, single undo restore); verify one undo restores pre-drag state after a successful drag commit

## 5. Headless acceptance scenarios

- [ ] 5.1 Test: two segments, Congruence at one end, Orthogonal, FixPoint at shared corner; soft-drag free end; verify Orthogonal and Congruence remain within tolerance after solve
- [ ] 5.2 Test: rectangle from four segments with shared corners (Congruence), opposite sides Parallel, one Orthogonal, one FixPoint; drag a free corner; verify structural residuals remain within tolerance and configuration stays near the previous state for a small move

## 6. Minimal desktop UI

- [ ] 6.1 Choose and wire a minimal canvas UI (egui or equivalent per design open question) that draws segments and logical UI points; verify the app window launches and shows an empty or sample sketch
- [ ] 6.2 Add interactions to create segments, join ends into shared UI points, and assign Parallel / Orthogonal / FixPoint; verify each action updates the core document as expected (manual check or debug overlay)
- [ ] 6.3 Implement pointer drag on logical UI points calling the interactive solve loop each move and committing on pointer-up; verify manually that a constrained right angle / rectangle does not fall apart while dragging
- [ ] 6.4 Surface solve failure in the UI (status text or log); verify a deliberately overconstrained sketch shows failure and keeps the last good figure
