# Linux Rust implementation

First implementation of the shared OpenSpec capabilities for the constraint
runtime (geometry IR, catalog, solve interaction, solver port).

## Requirements

- Rust stable (`rustc`, `cargo`) — installed in the Dev Container image, or via [rustup](https://rustup.rs/).
  The default build is pure Rust (sparse linear algebra via [`faer`](https://codeberg.org/sarah-quinones/faer), MIT).
- **Optional: Ceres Solver** for the reference backend (`--features ceres`): Debian
  package `libceres-dev` (pulls Eigen, glog, gflags). The Dev Container still
  installs it via `.devcontainer/Dockerfile` (`libceres-dev`, `pkg-config`, `g++`)
  so reference tests run there. Ceres is licensed under **BSD-3-Clause**
  (compatible with this crate’s MIT OR Apache-2.0).

## Solvers

| Backend | Role |
|---------|------|
| `LmSolver` | Default for `InteractiveSolve` / `sketch_ui`: owned sparse Levenberg–Marquardt (normal equations from the contribution-blocked Jacobian, sparse Cholesky) |
| `CeresSolver` | Optional reference backend behind `--features ceres` (dense, via C++ bridge) |
| `StubSolver` | Controllable success/failure for Interact tests |

`SolverOptions::lm_lambda0` is `LmSolver`'s initial damping relative to
`diag(JᵀJ)`; Ceres ignores it and maps `tol` / `max_iters` to function
tolerance and `max_num_iterations`.

## Build & test

```bash
cd impls/linux-rust
cargo test -p constraint_runtime
cargo build -p sketch_ui
```

With the Ceres reference backend (comparison tests in `tests/ceres_reference.rs`):

```bash
cargo test -p constraint_runtime --features ceres
```

Benchmarks (results in [`BENCHMARKS.md`](BENCHMARKS.md)):

```bash
cargo bench -p constraint_runtime --bench solve                  # sparse LM
cargo bench -p constraint_runtime --bench solve --features ceres # + Ceres
```

After Dockerfile dependency changes, rebuild the Dev Container so `libceres-dev` is present.

## Run UI

Needs the host display. The Dev Container mounts `/tmp/.X11-unix` and forwards `DISPLAY`
(see `.devcontainer/devcontainer.json`) and installs X11/EGL libs in the image. On the Linux
host, allow local Docker clients once per session:

```bash
xhost +local:docker
```

Then:

```bash
cargo run -p sketch_ui
```

Tools: Select/Drag, Segment (two clicks), Join, Parallel, Orthogonal, FixPoint.
