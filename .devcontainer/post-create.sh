#!/usr/bin/env bash
set -euo pipefail

echo "OpenSpec version: $(openspec --version)"
echo "Rust: $(rustc --version)"
echo "Cargo: $(cargo --version)"
echo "Node: $(node --version)"
echo "gh: $(gh --version | head -n1)"
echo "Claude Code: $(claude --version)"
echo "aider: $(aider --version)"

if [[ ! -d openspec ]]; then
  echo "Initializing OpenSpec for Cursor and Claude Code..."
  openspec init --tools cursor,claude --no-animation
else
  echo "Refreshing OpenSpec tool integrations..."
  openspec update --no-animation || true
fi

# Browser build of sketch_ui (WebAssembly + Trunk).
echo "Installing wasm32 target and trunk..."
rustup target add wasm32-unknown-unknown
if ! command -v trunk >/dev/null 2>&1; then
  if command -v cargo-binstall >/dev/null 2>&1; then
    cargo binstall -y trunk || cargo install --locked trunk
  else
    cargo install --locked trunk
  fi
fi
echo "trunk: $(trunk --version)"

echo "Dev container ready. Use /opsx-propose (Cursor) or /opsx:propose (Claude Code)."
