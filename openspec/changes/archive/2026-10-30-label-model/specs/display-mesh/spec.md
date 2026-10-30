# Spec Delta

## MODIFIED Requirements

### Requirement: Mesh exposes drawable edges tagged with geometry identity

The Mesh SHALL expose a collection of drawable edges. Each edge SHALL provide two endpoint locations (points) in **document/world coordinates** (the same space as State-evaluated geometry positions), suitable for stroking after a view transform, and SHALL carry the stable geometry identity of the geometry that produced it and that geometry's result/construction role (for presentation). For geometries tessellated into multiple edges in later slices, every edge of that geometry SHALL carry the same geometry identity. Mesh positions SHALL NOT be defined as window-pixel coordinates. Edge vertex locations SHALL be obtained by evaluating point expressions of the curve capability (on-curve samples, and endpoint expressions where the geometry has the endpoints capability) at the State used to build the Mesh, not by bypassing the expression evaluate path with geometry-only State sampling APIs. Tessellating a curve SHALL NOT require the endpoints capability. Geometries without the curve capability (labels) SHALL contribute no edges and no endpoint hit targets.

#### Scenario: Segment becomes one tagged edge

- **WHEN** a Document contains a single line segment geometry and the Mesh is built at the committed State
- **THEN** the Mesh contains one edge whose endpoints match the segment's evaluated endpoints in world coordinates
- **AND** that edge's geometry identity equals the segment's geometry identity

#### Scenario: Curve without endpoints is tessellated

- **WHEN** a Document contains a curve geometry whose kind has the curve capability but not the endpoints capability
- **AND** the Mesh is built at the committed State
- **THEN** the Mesh contains one or more edges tagged with that geometry's identity
- **AND** their vertices are on-curve samples of that geometry

#### Scenario: Label contributes no edges

- **WHEN** a Document contains one segment and one label and the Mesh is built
- **THEN** the Mesh contains exactly the segment's edge
- **AND** no edge or endpoint hit target carries the label's identity
