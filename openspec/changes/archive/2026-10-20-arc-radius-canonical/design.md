# Design

Target implementation: `impls/linux-rust` (crate `constraint_runtime`).

## Context

See proposal.md (Why). Current state:

- `geometry_kinds/arc.rs`: chart `(center: Point2, radius: Length, start: HeadingAngle, sweep: TurnAngle)`, curve `γ(t) = C + R·(cos θ, sin θ)` with `R` as stored. Length uses `|R|·|Δθ|` with a `signum` Jacobian. `LengthFormula::Radius` returns the signed `R` (Jacobian `[0,0,1,0,0]`). `HeadingFormula::StartHeading` returns `θ₀` as stored.
- `Length` is a signed State DOF by policy (`geom.rs`: "geometric non-negativity is separate"), because the solver has no bounds.
- `Document::add_geometry` packs the `Init` tuple directly. `Document::commit_vector` copies a Vector into the committed State. The only production caller of `commit_vector` is `InteractiveSolve::solve_commit_soft` (two call sites: after the soft phase and after the structural polish), each after `structural_ok(doc, &working)`. `restore_vector` is used for fallback / undo / cancel and restores previously committed (hence already canonical) States.
- No constraint in the catalog consumes heading or radius expressions today.

## Goals / Non-Goals

**Goals:**

- Committed State never contains `R < 0`. Solver iterates keep full freedom.
- Generic hook so other kinds can define a canonical chart form later (e.g. charts from the curve-family idea), without arc-specific code in Document / Interact.

**Non-Goals:**

- Angle wrapping, solver bounds, log-radius parameterization.

## Decisions

### 1. Canonical-form hook on `Geometry`

Add a provided method to the `Geometry` trait (`geometry/formula.rs`):

```rust
/// Canonical chart values describing the same curve (identity by default).
fn canonical(init: Self::Init) -> Self::Init { init }
```

`Arc` overrides it: if `R < 0` return `(C, −R, θ₀ + π, Δθ)`, else unchanged. `R = 0` stays (degenerate anyway). The result is the same curve at every `t`: `C + (−R)(cos(θ+π), sin(θ+π)) = C + R(cos θ, sin θ)`, and tangent / normal (`∂γ/∂t`) and length are unchanged.

*Alternative:* an arc-specific helper called from Document. Rejected, because the hook keeps Document kind-agnostic (dispatch via `with_kind!`, like all other kind operations).

### 2. Where canonicalization runs

- `Document::canonicalize(&self, p: &mut Vector)`: for every geometry, `with_kind!` → `unpack` its block → `K::canonical` → `pack` back into the same block. The block layout and size are unchanged, so no reindexing is needed.
- `Document::add_geometry`: packs `K::canonical(init)`.
- `InteractiveSolve::solve_commit_soft`: canonicalizes `working` **before** the structural gate at both commit points, then commits. The gate therefore always evaluates exactly the State that gets committed. Residuals of today's catalog are invariant under the rewrite anyway (point / vector / length expressions are geometric, and the radius expression becomes `|R|`), but checking the committed State is the robust order.
- `commit_vector` / `restore_vector` stay plain copies. All committing paths go through Interact, and restored States were committed before.

*Alternative:* canonicalize inside `Document::commit_vector`. Rejected, because the gate would then check a different State than the one committed. That is fine for today's constraints, but fragile for future heading-based constraints.

### 3. Radius expression returns `|R|`

`LengthFormula::Radius` evaluates to `|R|` with Jacobian `signum(R)` on the radius column (0 at `R = 0`), the same convention as `LengthFormula::Length`. Radius comparisons (future EqualRadius) are then geometric even in iterates. The kink at `R = 0` coincides with the degenerate arc.

The start-heading expression stays the raw `θ₀`. After canonicalization it equals the start point's polar angle. In iterates with `R < 0` it is off by `π`, which is acceptable because no catalog constraint uses it today. Documented in the arc docs.

### 4. Rejected: `R = e^ρ`

A log-radius parameter makes the radius structurally positive and smooth, and keeps "equal radius" linear in `ρ`. It was rejected for now: every radius derivative gets a factor `R`, initial values need `ln`, small radii become numerically sluggish, and `R = 0` is unreachable (no continuous collapse through degenerate arcs). Canonicalizing on commit achieves the user-visible goal with a local change.

## Risks / Trade-offs

- [Heading jumps by `π` on commit when `R` was negative] → `θ₀` is unwrapped; the jump only affects consumers of the raw heading parameter (none in the catalog). Stay-near soft terms are rebuilt from the committed snapshot at the next drag, so they see the canonical values.
- [Drag sessions] → undo snapshots are committed States, hence canonical.
- [Future heading-based constraints] → they must define their meaning on canonical arcs. The gate-after-canonicalize order (decision 2) makes violations visible instead of committing them.

## Migration Plan

Additive behaviour inside linux-rust; there is no persisted file format yet. Rollback: revert the change.
