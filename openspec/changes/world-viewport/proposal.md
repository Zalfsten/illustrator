# Proposal

## Why

Document State and Mesh currently use window-pixel positions as if they were geometry coordinates. That couples the constraint model to the GUI toolkit and breaks scientific y-up conventions. Separating a durable world frame from screen presentation is required before pan/zoom, export, and multi-view work.

## What Changes

- Document / State / Mesh positions SHALL live in a **world coordinate frame** with **y-up** (independent of window pixels).
- The App SHALL own a **Viewport** (session state) that maps world ↔ screen, including the usual **y-axis flip** for GUI toolkits.
- Canvas paint SHALL transform Mesh edges from world to screen; pointer input SHALL map screen → world before Document / Interact updates.
- Hit radii expressed in **screen pixels** SHALL be converted to world length (`r_px / scale`) before Mesh hit queries (Mesh API stays in world space).

## Capabilities

### New Capabilities

- *(none)* — viewport and world frame extend existing App / Mesh contracts.

### Modified Capabilities

- `system-architecture`: App owns Viewport; Canvas paint and pointer input use world ↔ screen mapping; world frame is y-up.
- `display-mesh`: Mesh edge positions and hit query space are document/world coordinates (not screen pixels).

## Non-goals

- Interactive pan/zoom gestures or UI chrome for camera control.
- Nonuniform scale, rotation beyond y-flip, perspective.
- Export DPI / print pipelines; multi-canvas synchronized views.

## Impact

- Shared specs: `system-architecture`, `display-mesh`; glossary in `docs/constraint-mathematics.md`.
- Linux Rust: `sketch_ui` Viewport + rewire sample/paint/input; `constraint_runtime` Mesh/Document remain world-agnostic numerically (no toolkit dependency).
