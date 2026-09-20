# Proposal

## Why

The owned dense LM backend proves the SolverPort contract, but interactive sketches benefit from a battle-tested nonlinear least-squares library. Binding Google Ceres behind the existing port validates the FFI evaluation path and improves default solve quality without changing interaction semantics.

## What Changes

- Add a **Ceres** adapter implementing `SolverPort`, calling residual/Jacobian fills through the existing C-ABI trampolines (or an equivalent cxx bridge that uses the same `SystemEval` contract).
- Install/link Ceres (and its deps) in the Dev Container / build for `impls/linux-rust`.
- Make **Ceres the default** backend for interactive solve (`InteractiveSolve` / `sketch_ui`).
- Keep **`LmSolver`** as an owned fallback for tests and comparison.
- **Non-goals:** runtime config UI or env switch between backends (deferred); sparse Jacobians; replacing soft/hard Interact semantics; naming Ceres in shared OpenSpec requirements.

## Capabilities

### New Capabilities

- (none — `skip_specs: true`)

### Modified Capabilities

- (none — existing `solver-port` / `solve-interaction` already require a replaceable backend and FFI-friendly eval; this change only implements them with Ceres as the linux-rust default)

## Impact

- `impls/linux-rust`: new Ceres binding module/crate pieces, Dockerfile/apt deps, default `InteractiveSolve` backend.
- Build becomes dependent on Ceres + Eigen (typical Debian packages or vendored build).
- Acceptance/UI behavior remains under the same residual/commit contracts; tests should cover Ceres path and retain LM coverage where useful.
- Shared main specs unchanged at archive (no delta sync).
