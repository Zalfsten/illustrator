#!/usr/bin/env bash
# Platform-neutrality check: the core crates must build for the browser target.
# Usage: impls/rust/scripts/check-wasm.sh  (from anywhere)
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build --target wasm32-unknown-unknown -p constraint_runtime -p sketch_ui
