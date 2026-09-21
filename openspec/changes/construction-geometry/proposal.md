# Proposal

## Why

Scientific figures use temporary geometry (auxiliary lines, helper circles) to construct durable relationships, then hide those helpers in the published figure. Today every geometry is equally visible and pickable, so construction helpers clutter the result view and remain editable when they should not. Encoding construction role in the Document and tying visibility to pickability keeps helpers in the constraint graph without exposing them in the result presentation.

## What Changes

- Introduce a **geometry role** (result vs construction) stored on each geometry in DocumentStructure — not GUI-only state.
- Construction geometries SHALL remain first-class for constraints and solve; role does not remove them from the IR.
- The App SHALL offer a presentation mode that either shows **result only** or **includes construction**. When construction geometry is not shown, it SHALL NOT be pickable (visibility and hit-testing share one policy).
- Display Mesh (and Mesh-backed hits) SHALL respect that filter. No layer system in this slice.

## Capabilities

### New Capabilities

- *(none)* — construction role extends existing geometry and display contracts.

### Modified Capabilities

- `geometry-ir`: Geometry role (result vs construction); construction remains addressable in expressions/constraints.
- `display-mesh`: Mesh build and hit queries honor a visibility filter derived from presentation mode and geometry role; invisible ⇒ not pickable.
- `system-architecture`: App presentation mode for construction visibility; Canvas/Mesh consume the filtered view.

## Non-goals

- Layer stacks, per-layer lock/export, multi-layer UI.
- Separate Document copies per view; styles beyond a simple construction appearance cue.
- Auto-promoting construction to result; deleting construction on export (filter only).
- Changing which geometries tools may *create* as construction by default beyond a minimal UI affordance if needed for demo.

## Impact

- Shared specs: `geometry-ir`, `display-mesh`, `system-architecture`.
- Linux Rust: flag/role on `Geometry`/`Segment` (or structure metadata), Mesh filter API, `sketch_ui` toggle.
- Export/print later reuses the same filter.
