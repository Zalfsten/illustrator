# Proposal

## Why

The circular-arc chart stores a **signed** radius: `(C, R, θ₀, Δθ)` and `(C, −R, θ₀ + π, Δθ)` describe the same arc. A negative radius has no geometric meaning, and the ambiguity leaks into everything built on the chart:

- the "radius" value needs `|R|` everywhere,
- the start-heading expression returns `θ₀`, while the start point's real polar angle is `θ₀ + π` when `R < 0`,
- an equal-radius relation `R_a − R_b` is wrong when `R_a = −R_b`.

The upcoming curve invariants and curve relations (change `curve-invariants-and-relations`: concentric, equal radius, …) need a clean, non-negative radius. For figures in publications and teaching, the radius a user sees and constrains must be the geometric one.

The sign exists for a reason: the owned LM solver has no bounds, so an iterate can always cross `R = 0`. The signed chart keeps the curve formula smooth there. This change keeps that solver freedom and removes the ambiguity from everything outside the solve.

## What Changes

- **Canonical committed radius:** the committed document State never holds a negative arc radius. Whenever a solved working State is committed (and when an arc is added), an arc with `R < 0` is rewritten to `(C, −R, θ₀ + π, Δθ)`. The curve is identical, so points, tangents, normals and length are unchanged at every `t`.
- **Solver freedom unchanged:** during a solve the radius parameter stays an unconstrained real. Negative values may occur in iterates and are harmless.
- **Radius expression yields the magnitude** `|R|` (consistent with the arc length `|R|·|Δθ|`), so radius comparisons are geometric even in iterates.
- Spec `geometry-ir`: the arc chart's radius becomes "radius length DOF, non-negative in committed State" instead of "signed radius length DOF", with the canonicalization rule.

## Capabilities

### New Capabilities

- (none)

### Modified Capabilities

- `geometry-ir`: "Circular arc is an admitted curve kind with endpoints". The radius is non-negative in committed State, a commit canonicalizes negative-radius iterates without changing the curve, and the radius expression yields the magnitude.

## Non-goals

- Curve invariant capabilities, renaming Parallel/Orthogonal, new curve relations (follow-up change `curve-invariants-and-relations`).
- Bounds or barrier terms in the solver. A log-radius parameterization (`R = e^ρ`) was considered and rejected (see design).
- Wrapping angles: headings and turns stay unwrapped reals, as before.

Specs stay shared across implementations. Where the canonicalization is triggered and how solver iterates are represented are design decisions of each implementation.

## Impact

- `impls/linux-rust/crates/constraint_runtime`: `geometry/formula.rs` (canonical form hook on `Geometry`), `geometry_kinds/arc.rs` (canonical form, radius formula, docs), `document.rs` (canonicalize a State, canonical `add_geometry`), `interact.rs` (canonicalize the working State before the structural gate / commit), tests.
- Docs: `docs/constraint-mathematics.md` (arc chart), `openspec/config.yaml` context ("signed Length radius").
- `openspec/specs/geometry-ir/spec.md` via the delta.
