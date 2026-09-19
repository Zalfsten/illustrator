# Tasks

## 1. State vector and connectivity

- [x] 1.1 Introduce authoritative `StateVector` / `p: Vec<f64>` and change segments to store param offsets (stride 4); verify unit tests that editing `p` changes endpoint evaluation
- [x] 1.2 Represent explicit local→global connectivity per geometry and per constraint contribution; verify a fixture prints expected index maps for a two-segment join

## 2. Geometry eval and local Jacobians

- [x] 2.1 Implement analytical local residuals and local Jacobians for Congruence, Parallel, Orthogonal, FixPoint w.r.t. involved local params; verify against finite-difference on a fixture within tight tolerance
- [x] 2.2 Expose geometry quantity eval (endpoints, direction) from `p` via connectivity; verify existing geom unit tests adapted to state-vector storage

## 3. System assembly

- [x] 3.1 Assemble global residual by scattering local residuals; verify arity and values on the catalog fixture (2+1+1+2)
- [x] 3.2 Assemble global Jacobian by scattering local \(J_N\); verify FD of global `f(p)` matches assembled \(J\) on a small sketch
- [x] 3.3 Assemble soft mouse/stay contributions the same way without persisting them; verify drag commit leaves only structural constraints

## 4. Solver port

- [x] 4.1 Define `SystemEval` + `SolverPort` + `SolverOptions` (tol, max_iters, LM-related knobs); verify stub backend swaps in tests
- [x] 4.2 Reimplement owned LM against `SystemEval` using assembled analytical \(J\); verify congruence/fixpoint and acceptance drag tests still pass
- [x] 4.3 Shape eval callbacks so an `extern "C"` trampoline could call them (opaque pointer + fill buffers); verify with a thin Rust test double that mimics the C calling pattern

## 5. UI and cleanup

- [x] 5.1 Update `sketch_ui` to the state-vector document API; verify `cargo build -p sketch_ui` and manual drag still keeps right angles
- [x] 5.2 Remove or quarantine document-owned parameter pack/unpack and FD-as-primary paths; verify `cargo test -p constraint_runtime` is green
