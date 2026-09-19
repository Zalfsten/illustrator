# Proposal

## Why

The first runtime spike stores parameters on geometry objects and feeds the solver a whole document with finite-difference Jacobians. Discussion clarified the intended core architecture: a single global state vector, local→global connectivity for FEM-style residual/Jacobian assembly, geometries that expose values and derivatives, and a swappable solver port. Those contracts belong in shared specs so every implementation—and any future third-party solver adapter—conforms to the same shape.

## What Changes

- Specify a **global state vector** as the sole continuous unknown; local geometry parameters are references (connectivity) into that vector, not independent owned copies.
- Require geometries (and constraints built on them) to expose **evaluation functions and analytical derivatives** with respect to their local parameters.
- Specify **connectivity mapping** and **assembly** of the global residual and global Jacobian from local contributions (scatter), analogous to FEM matrix assembly.
- Specify an **abstract solver port**: implementations MAY use an owned solver or adapt an external library; the port MUST accept residual/Jacobian evaluation callbacks (or equivalent) and expose common solver controls (tolerance, iteration limits, damping/algorithm options as applicable).
- Align the Linux Rust impl with these specs (refactor away from document-owned parameter copies and FD-only Jacobians as the primary path).
- Non-goals: changing the v1 constraint *catalog* semantics (Congruence, Parallel, Orthogonal, FixPoint) or illustration features.

## Capabilities

### New Capabilities

- `state-vector`: Global continuous state vector; local parameters as references into it.
- `geometry-eval`: Geometry- and constraint-level evaluation of quantities and derivatives w.r.t. local parameters.
- `system-assembly`: Local→global connectivity mapping; assembly of global residual and Jacobian.
- `solver-port`: Abstract nonlinear solver interface, swappable backends, common solver parameters.

### Modified Capabilities

- (none in main specs yet — `geometry-ir` / `constraint-catalog` / `solve-interaction` still live only under change `constraint-runtime-v1`; this change adds the solver-core architecture. On archive, these capabilities complement that slice.)

## Impact

- Shared specs become the source of truth for solver-core architecture across implementations.
- `impls/linux-rust` must be refactored to match; behavioral drag/catalog specs from `constraint-runtime-v1` remain in force.
- External solver libraries attach behind `solver-port` without changing IR or assembly contracts.
