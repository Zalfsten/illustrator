# Linux Rust implementation

First implementation of the shared OpenSpec capabilities for the constraint
runtime (geometry IR, catalog, solve interaction, solver port).

## Requirements

- Rust stable (`rustc`, `cargo`) — installed in the Dev Container image, or via [rustup](https://rustup.rs/).
- **Ceres Solver** (default interactive backend): Debian package `libceres-dev`
  (pulls Eigen, glog, gflags). In the Dev Container this is installed via
  `.devcontainer/Dockerfile` (`libceres-dev`, `pkg-config`, `g++`).
- Ceres is licensed under **BSD-3-Clause** (compatible with this crate’s MIT OR Apache-2.0).

## Solvers

| Backend | Role |
|---------|------|
| `CeresSolver` | Default for `InteractiveSolve` / `sketch_ui` (external NLS) |
| `LmSolver` | Owned dense Levenberg–Marquardt (tests / fallback) |
| `StubSolver` | Controllable success/failure for Interact tests |

`SolverOptions::lm_lambda0` applies only to `LmSolver`. Ceres maps `tol` and
`max_iters` to function/parameter tolerance and `max_num_iterations`.

## Build & test

```bash
cd impls/linux-rust
cargo test -p constraint_runtime
cargo build -p sketch_ui
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
