#!/usr/bin/env bash
# Platform-neutrality check: the core crates must build for the browser target.
# Usage: impls/rust/scripts/check-wasm.sh  (from anywhere)
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build --target wasm32-unknown-unknown -p constraint_runtime -p sketch_session -p label_layout -p label_web -p label_worker -p sketch_ui
