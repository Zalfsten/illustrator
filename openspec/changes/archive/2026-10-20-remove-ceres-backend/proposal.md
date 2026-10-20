# Proposal

## Why

Since `sparse-lm-solver`, the owned pure-Rust sparse Levenberg–Marquardt solver is the default backend and is much faster than Ceres (drag step at N = 100: 1.5 ms vs 323 ms). Ceres is now only an optional reference behind a Cargo feature. It still costs a C++ bridge, a build script, unsafe FFI glue, a large native package in the Dev Container, and a feature path nobody builds by default, so it can break without anyone noticing. It also stands in the way of the planned browser (WebAssembly) build, which keeps the illustration tool usable in teaching settings without installation.

With Ceres gone, the foreign-function evaluation layer has no consumer left. The shared spec requirement that the evaluation contract be bindable to foreign-language solvers no longer serves the product direction: pure-Rust in-process backends and WebAssembly.

## What Changes

- Remove the Ceres backend from linux-rust: adapter, C++ bridge, build script and `cc` build dependency, the `ceres` feature, its tests and benchmark group, and `libceres-dev` / `g++` from the Dev Container.
- Remove the foreign-function evaluation layer (opaque handle + C trampolines) from linux-rust.
- **BREAKING (shared spec, solver-port):** drop the requirement "FFI-friendly evaluation contract". Implementations still may adapt external solver libraries behind the solver port ("Owned or external backend" stays), but the evaluation contract no longer has to be expressible at a C ABI. The C-ABI wording in "Evaluation fills first-class residual and Jacobian buffers" is removed accordingly.
- Clean up documentation, comments and project context that mention Ceres or FFI.
- Kept on purpose (user decision): the `SolverPort` trait with `StubSolver` and the generic `InteractiveSolve<S>`, and the `SystemEval` trait. No other abstraction is removed.

## Capabilities

### New Capabilities

- (none)

### Modified Capabilities

- `solver-port`: remove the "FFI-friendly evaluation contract" requirement. Remove the C-ABI and FFI-adapter wording from "Evaluation fills first-class residual and Jacobian buffers".

## Non-goals

- WebAssembly build (later change; this change only removes the last native dependency on the solver path).
- Changes to solver algorithms, interaction semantics or the block-sparse Jacobian.
- Removing `SolverPort`, `StubSolver`, `InteractiveSolve<S>` or `SystemEval`.

Specs stay shared across implementations. Another implementation may still bind a foreign solver behind the solver port. It just no longer has to via a C-ABI evaluation contract.

## Impact

- `impls/linux-rust/crates/constraint_runtime`: delete `src/ceres_solver.rs`, `src/tests_ceres_solver.rs`, `cpp/`, `build.rs`, `src/solver/ffi.rs`, `src/solver/tests_ffi.rs`, `tests/ceres_reference.rs`. Edit `Cargo.toml`, `src/lib.rs`, `src/solver.rs` (docs), `benches/solve.rs`.
- `.devcontainer/Dockerfile` (drop `libceres-dev`, `g++`; keep `pkg-config`, which `sketch_ui` needs via `wayland-sys`), `.devcontainer/devcontainer.json` (comment).
- Docs: `impls/linux-rust/README.md`, `docs/constraint-mathematics.md`, `openspec/config.yaml` context. `impls/linux-rust/BENCHMARKS.md` keeps the historical Ceres numbers.
- `openspec/specs/solver-port/spec.md` via the delta.
