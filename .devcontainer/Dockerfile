FROM mcr.microsoft.com/devcontainers/javascript-node:1-22-bookworm

# OpenSpec CLI (spec-driven workflow for AI coding agents)
RUN npm install -g @fission-ai/openspec@latest \
  && openspec --version

# X11 / OpenGL runtime libs for egui/winit (host display via /tmp/.X11-unix)
RUN apt-get update \
  && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
    libxcursor1 \
    libxrandr2 \
    libxi6 \
    libx11-6 \
    libxkbcommon0 \
    libxkbcommon-x11-0 \
    libxcb1 \
    libxcb-render0 \
    libxcb-shape0 \
    libxcb-xfixes0 \
    libgl1 \
    libegl1 \
    libfontconfig1 \
  && rm -rf /var/lib/apt/lists/*

# Rust toolchain for impls/linux-rust
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --default-toolchain stable --profile minimal \
  && chmod -R a+w "$RUSTUP_HOME" "$CARGO_HOME" \
  && rustc --version && cargo --version
