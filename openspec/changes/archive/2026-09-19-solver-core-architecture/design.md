# Design

## Context

See `proposal.md`. Change `constraint-runtime-v1` delivered a working spike where segments own `(x1,y1,x2,y2)`, the document packs/unpacks a parameter vector each solve, and `LmSolver` uses finite-difference Jacobians over a fat `Document` API. This change’s specs define the intended architecture; this design describes how the Linux Rust impl will conform.

## Goals / Non-Goals

**Goals:**

- Refactor `constraint_runtime` so \(p\) is authoritative; geometries hold connectivity ranges into \(p\).
- Implement local residual + analytical local Jacobian for the v1 catalog; scatter into global \(r\) and \(J\).
- Replace `SolverBackend::solve(&mut Document)` with a port over evaluation callbacks + `&mut [f64]` state, keeping an owned LM backend first and leaving an FFI-shaped eval contract ready for adapters.
- Keep egui UI and drag semantics working against the new core.

**Non-Goals:**

- Shipping an external solver (Ceres/GSL/…) in this change—only the port shape and an owned backend.
- Changing constraint catalog semantics or adding new constraint types.
- Sparse CSR storage optimization beyond what the small dense LM needs (assembly API should still be scatter-shaped so sparse backends can follow later).

## Decisions

### D1: State layout

**Choice:** `StateVector { p: Vec<f64> }` owned by the document/session; `Segment { id, param_offset }` with fixed stride 4 (endpoint chart). Constraints store geometry ids; assembly looks up connectivities.

**Alternatives:** Keep scalars on structs and sync (rejected—violates specs). Variable charts per segment (deferred).

### D2: Connectivity type

**Choice:** Explicit `Connectivity { global_indices: SmallVec<[usize; 8]> }` (or fixed arrays) per contribution; geometries expose param connectivity; constraints expose residual-row ownership plus column connectivity for involved params.

**Rationale:** Matches FEM scatter; same indices used for gather of \(p_N\) and scatter of \(J_N\).

### D3: Evaluation traits

**Choice:**

- `ResidualContribution`: `residual_arity`, `connectivity`, `eval_residual(p, out)`, `eval_jacobian(p, out_local_or_scatter)`.
- `AssembledSystem`: builds global \(r\), \(J\) (dense `n_r × n_p` for now) at \(p\).
- Soft terms implement the same contribution trait, attached only during interaction.

### D4: Solver port

**Choice:**

```text
trait SolverPort {
  fn solve(&mut self, eval: &dyn SystemEval, p: &mut [f64], opts: SolverOptions) -> SolveResult;
}

trait SystemEval {
  fn n_params(&self) -> usize;
  fn n_residuals(&self) -> usize;
  fn residual(&self, p: &[f64], r: &mut [f64]);
  fn jacobian(&self, p: &[f64], j: &mut [f64]); // row-major dense for owned LM
}

struct SolverOptions { tol, max_iters, lm_lambda0, ... }
```

Owned `LmSolver` uses analytical \(J\) from `SystemEval`. FFI adapter later: C trampolines calling `SystemEval`.

### D5: Relationship to prior spike

**Choice:** Refactor in place under `impls/linux-rust`; keep acceptance tests; replace FD primary path with analytical assembly (FD may remain as a test oracle).

## Risks / Trade-offs

- **[Risk] Dense Jacobian cost** → Accept for v1 sizes; API stays scatter-friendly for sparse later.
- **[Risk] Drift from `constraint-runtime-v1` delta wording** → Keep catalog/drag specs; architecture specs override storage/solver shape.
- **[Trade-off] Explicit connectivity verbosity vs. correctness** → Prefer verbosity; matches user FEM mental model.

## Migration Plan

Refactor library modules; update UI to mutate \(p\) / rebuild connectivities on add-segment; re-run `cargo test -p constraint_runtime`.

## Open Questions

- Exact `SolverOptions` field set beyond tol / max_iters / LM λ — expand when second backend appears.
- Whether to archive `constraint-runtime-v1` before or after this refactor lands (process only).
