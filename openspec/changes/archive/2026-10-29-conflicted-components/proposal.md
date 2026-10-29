# Proposal

## Why

A drawing must be able to hold a contradiction without becoming unusable. Today, solving and the commit check cover the whole drawing at once. One contradiction anywhere therefore makes every later constraint command and every drag fail, also in unrelated parts of the figure. Contradictions will reach a drawing through loaded files (#8: hand-written, AI-generated, damaged, or checked against a stricter tolerance in a later version) and, later, through complex operations. Direct user input stays strict (#30): a command that creates a new contradiction is still rejected.

This serves the product goal of durable relationships in figures that are edited over time. A figure with one broken relation stays editable everywhere else, and the broken part keeps its last shape until the user resolves it, instead of being moved to some other solution.

## What Changes

- The constraint graph splits into **components**: geometries connected through the constraints that reference them. Components depend on structure only.
- A component is **conflicted** if its structural residual at its committed values is above solve tolerance; otherwise it is **consistent**. Nothing extra is stored, so undo and redo carry the state along with the drawing.
- Solving and the commit check work **per component**: a command or drag solves the components it touches and checks only those; other components are not moved and not checked.
- **Conflicted components are frozen**: their values do not change. Only delete, rename and set-role apply to them. A constraint command that touches a frozen component, or a drag that starts on one of its points, fails with the new error class **`frozen`**.
- **Rejection refers to new contradictions:** a command fails with `unsatisfiable` if it leaves a previously consistent component (or a new geometry) in a conflicted component. Contradictions elsewhere do not block commands.
- **Resolving:** a delete inside a frozen component always succeeds; every remaining piece of it gets one solve attempt and is consistent again if that succeeds.
- **Session start:** a session created from an existing drawing gives each component above tolerance one solve attempt; the rest stay frozen. This is the entry point file loading (#8) will use.
- The session **lists conflicted components** (names of their geometries and constraints, residual).

## Non-goals

- Any display: red colouring, status line, drag hint (#52 builds on the listing and the `frozen` class).
- Rank analysis: naming the contradicting constraints, telling redundant from contradicting (#16). Detection here is by residual only, so a component the solver cannot solve from its current values counts as conflicted (same known limitation as #30).
- File loading and its load mode without eager solve (#8), exact duplicate handling (#53), a user-facing conflict policy, complex operations that may leave contradictions.
- Change commands that edit a constraint in place (for example a new fix target); they will be added later as further resolving edits.

The requirements stay shared across implementations; the per-component solve is an observable contract, not a prescribed algorithm.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `constraint-catalog`: components of the constraint graph become enumerable from structure; the hard-invariant requirement applies per consistent component.
- `sketch-commands`: structural consistency per component, rejection of new contradictions only, the `frozen` error class, resolving by delete, session start from an existing drawing, listing conflicted components.
- `solve-interaction`: a drag solves and checks only the dragged component; no drag on a frozen component.

## Impact

- `impls/rust/crates/constraint_runtime`: component enumeration (structure only, like congruence classes); assembly and the structural check restricted to a set of components; interactive solve and drag per component.
- `impls/rust/crates/sketch_session`: per-component logic in `apply`, `drag_begin` and `Session::new`; `CommandError::Frozen`; `Session::conflicts()`.
- `impls/rust/crates/sketch_ui`: no behaviour change; compiles against the new error variant.
- Unblocks the conflict part of #8 and the red display of #52.
