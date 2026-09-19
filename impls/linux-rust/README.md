# Linux Rust implementation

First implementation of the shared OpenSpec capabilities (`geometry-ir`,
`constraint-catalog`, `solve-interaction`) for change `constraint-runtime-v1`.

## Requirements

- Rust stable (`rustc`, `cargo`) — installed in the Dev Container image, or via [rustup](https://rustup.rs/).

## Build & test

```bash
cd impls/linux-rust
cargo test -p constraint_runtime
cargo build -p sketch_ui
```

## Run UI

Needs the host display. The Dev Container mounts `/tmp/.X11-unix` and forwards `DISPLAY`
(see `.devcontainer/devcontainer.json`) and installs X11/EGL libs in the image. On the Linux
host, allow local Docker clients once per session:

```bash
xhost +local:docker
```

Then rebuild/reopen the container (needed after Dockerfile X11-lib changes) and:

```bash
cargo run -p sketch_ui
```

Tools: Select/Drag, Segment (two clicks), Join, Parallel, Orthogonal, FixPoint.
