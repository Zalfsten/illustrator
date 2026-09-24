# Design

## Context

See proposal.md — Why. Today `sketch_ui` stores sample endpoints and pointer positions as egui pixel coordinates in Document State. Mesh and hits share that space. Target: Linux Rust `constraint_runtime` (unchanged Mesh world API) + `sketch_ui` Viewport.

## Goals / Non-Goals

**Goals:**

- World y-up in Document/Mesh; Viewport in App with uniform scale, origin, y-flip.
- Paint via `to_screen`; input and hits via `to_world` + `px_to_world`.
- Fit Viewport to canvas rect and fixed world bounds (margin); no zoom gestures.

**Non-Goals:** Pan/zoom UI; nonuniform scale; putting Viewport inside `constraint_runtime`.

## Decisions

### 1. Viewport lives only in the App

**Choice:** `Viewport` module in `sketch_ui`.  
**Why:** Specs keep Document toolkit-free; Mesh stays pure world geometry.  
**Alternative:** Viewport in `constraint_runtime` — rejected (GUI coupling).

### 2. Mesh stays in world; scale pixel radii at the call site

**Choice:** `r_world = r_px / scale` before `hit_endpoint` / `hit_geometry`.  
**Why:** Existing Mesh API and tests stay valid; pick size stable under zoom when scale changes later.  
**Alternative:** Screen-space hit after transforming all edges — more code, same UX.

### 3. Fit-to-bounds, not interactive camera

**Choice:** Each frame (or on resize), rebuild Viewport from canvas rect + fixed world AABB with margin.  
**Why:** Enough to prove the split; pan/zoom is a later change.

### 4. Sample in world units

**Choice:** Right-angle sample near the origin (e.g. `(0,0)-(2,0)`, `(2,0)-(2,2)`).  
**Why:** Makes the pixel→world bug impossible to miss.

## Risks / Trade-offs

- [Logical handles in world, paint in screen] → Always go through Viewport; never mix Pos2 into State.
- [Non-square pixels / DPI] → Deferred; scale is logical egui points for now.

## Migration Plan

1. Land OpenSpec artifacts.
2. Add Viewport + tests; rewire App; update glossary.
3. Archive → sync main specs.

## Open Questions

- None for this slice (y-up and pixel radius confirmed).
