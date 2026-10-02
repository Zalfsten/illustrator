# Design

## Context

See proposal.md. linux-rust already has opaque `PointExpr` / `VecExpr` (Model B), capability traits `CurveExprs` / `EndpointExprs`, and catalog constraints with arity-1 or arity-2 residuals. No scalar expression type or length factory exists yet.

## Goals / Non-Goals

**Goals:**

- Opaque `ScalarExpr` with evaluate / param_indices / 1×n analytical Jacobian.
- `CurveExprs::length_expr` → `ScalarExpr`; segment implements Euclidean length.
- `Constraint::EqualLength { a, b }` with residual \(a - b\).
- Stack-friendly scalar Jacobian carrier alongside point/vector Jacobians.

**Non-Goals:** Public compose (`norm`, point−point); `FixLength`; circle; general arc-length integration; UI tool for EqualLength.

## Decisions

### 1. ScalarExpr mirrors Point/Vec (Model B)

Kind-keyed outer catalog + shared/capability formula tags; constraints never match formulas. Add `ScalarJacobian` as \(1 \times n\) (same `MAX_*` width policy as point Jacobians, or shared max).

### 2. `length_expr` on `CurveExprs`, not `EndpointExprs`

Length applies to closed curves. Segment override/impl: \(L = \|B - A\|\) with analytical \(1 \times 4\) (or sparse if desired) Jacobian. No trait default that invents general arc length.

Macro: extend `impl_curve_exprs!` to also emit `length_expr` via a kind wrap (e.g. `segment_scalar`), or add `impl_curve_length!` — prefer extending the curve macro with a scalar wrap ident.

### 3. EqualLength residual

Single residual \(r = s_a - s_b\). Local Jacobian: concatenate \(\partial s_a\) and \(-\partial s_b\) like Parallel concatenates vector blocks. Structural orphan-drop and assembly treat it like other structural constraints.

### 4. No compose in this change

Segment length is a kind formula, not `norm(end - start)` as a public API. Compose can land later without blocking EqualLength.

## Risks / Trade-offs

- **[Risk]** Near-zero length → singular Jacobian columns for \(L\) → Mitigation: same degeneracy stance as zero tangent (assembly must not crash; meaningful solve not required).
- **[Trade-off]** Extending `CurveExprs` is a **BREAKING** trait change for any external impl (in-repo only Segment today).

## Migration Plan

Implement ScalarExpr + segment length + EqualLength; update docs glossary; tests (unit Jacobian FD + acceptance two segments equal length under drag).

## Open Questions

None.
