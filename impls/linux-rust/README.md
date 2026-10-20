# Linux Rust implementation

First implementation of the shared OpenSpec capabilities for the constraint
runtime (geometry IR, catalog, solve interaction, solver port).

## Requirements

- Rust stable (`rustc`, `cargo`) — installed in the Dev Container image, or via [rustup](https://rustup.rs/).
  The solver is pure Rust (sparse linear algebra via [`faer`](https://codeberg.org/sarah-quinones/faer), MIT);
  no native solver library is needed.
- `sketch_ui` needs `pkg-config` and a C compiler for its windowing stack
  (`wayland-sys`, `wayland-backend`); both are in the Dev Container image.

## Solvers

| Backend | Role |
|---------|------|
| `LmSolver` | Default for `InteractiveSolve` / `sketch_ui`: owned sparse Levenberg–Marquardt (normal equations from the contribution-blocked Jacobian, sparse Cholesky) |
| `StubSolver` | Controllable success/failure for Interact tests |

`SolverOptions::lm_lambda0` is `LmSolver`'s initial damping relative to
`diag(JᵀJ)`. The former Ceres reference backend was removed (change
`remove-ceres-backend`); its measured numbers stay in
[`BENCHMARKS.md`](BENCHMARKS.md).

## Build & test

```bash
cd impls/linux-rust
cargo test -p constraint_runtime
cargo build -p sketch_ui
```

Benchmarks (results in [`BENCHMARKS.md`](BENCHMARKS.md)):

```bash
cargo bench -p constraint_runtime --bench solve
```

After Dockerfile dependency changes, rebuild the Dev Container.

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
