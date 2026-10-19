# Proposal

## Why

The linux-rust default solver is Ceres, a C++ library behind an FFI bridge. That native dependency is the main obstacle to a later WebAssembly build of the constraint runtime, which would let the illustration tool run in a browser for teaching use. At the same time, the evaluation contract fixes a dense \(m \times n\) Jacobian, so each solve step costs \(O(m n^2)\) and allocates \(O(m n)\) memory. Interactive drag will not scale to the multi-figure sketches the product targets (axes, curves, tangents, labelled constructions).

Both problems sit in the same place, the Jacobian representation and the solver that consumes it, so this change addresses them together.

## What Changes

- **BREAKING (evaluation contract):** the Jacobian handed from assembly to a solver backend is no longer required to be a dense \(m \times n\) buffer. Assembly fills a **contribution-blocked sparse Jacobian**: each contribution's derivative block is stored once, with its residual rows and connectivity columns, and the global \(J\) is defined as the sum of those blocks.
- Assembly makes explicit that duplicate connectivity columns within one contribution (e.g. a Congruence between two endpoints of the same geometry) **sum** into the global Jacobian. Before this change, a later block could overwrite an earlier one.
- The default interactive backend in linux-rust becomes an **owned, pure-Rust sparse damped least-squares solver**. Ceres becomes an optional, non-default reference backend, behind the same solver port.
- A hot-path benchmark is added before the refactor, so performance claims can be measured.

## Capabilities

### New Capabilities

- (none)

### Modified Capabilities

- `solver-port`: the evaluation contract allows (and the owned backend uses) a sparse, contribution-blocked Jacobian instead of mandating a dense buffer. External adapters may densify.
- `system-assembly`: the global Jacobian is the **sum** of contribution blocks, with duplicate connectivity columns accumulated. Contribution column indices are an assembly-owned mapping, not required to be the Document's State indices.

## Non-goals

- WebAssembly build or browser front-end. This change only removes the native default dependency that would block it.
- Solver-side elimination of trivial point identifications (Congruence between parameter points), arc reparametrisation, and splitting into connected components. The column-mapping decision keeps elimination possible later.
- Changes to soft/hard interaction semantics, soft weights, or commit gating.
- Illustration features (styles, labels, export). They are unaffected.

Specs stay shared across implementations. Library and algorithm choices (sparse factorisation, damping strategy) are linux-rust design decisions in `design.md`.

## Impact

- `impls/linux-rust/crates/constraint_runtime`: `matrix.rs` (new block-sparse Jacobian; scatter view removed), `assemble.rs`, `solver.rs` (`SystemEval` signature, new sparse LM), `solver/ffi.rs`, `ceres_solver.rs`, `build.rs`, `interact.rs` (default backend), `lib.rs` re-exports, tests and a new benchmark.
- New Rust dependency: `faer` (pure Rust sparse linear algebra). Ceres, glog, gflags and Eigen become optional (feature `ceres`). The Dev Container may keep them installed for reference tests.
- `sketch_ui`: no API change expected beyond the default backend type.
