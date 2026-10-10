# Spec Delta: display-mesh

## ADDED Requirements

### Requirement: Circular arcs tessellate into polyline Mesh edges

When the Document contains a circular arc geometry that passes the Mesh visibility filter, building the Mesh at State \(p\) SHALL emit one or more drawable edges that approximate the arc as a polyline in world coordinates. Every edge produced for that arc SHALL carry the same geometry identity and role. The polyline SHALL begin at the arc's evaluated start endpoint and end at its evaluated end endpoint. Subdivision density MAY depend on the absolute sweep; a single chord MAY be used only when that remains a faithful display of a near-degenerate sweep. Segment geometries SHALL continue to contribute a single edge between their endpoints.

#### Scenario: Arc becomes multiple tagged edges

- **WHEN** a Document contains a single circular arc with a non-trivial sweep and the Mesh is built at the committed State
- **THEN** the Mesh contains more than one edge tagged with that arc's geometry identity
- **AND** the first polyline vertex matches the arc start endpoint and the last matches the arc end endpoint (within ordinary floating tolerance)

#### Scenario: Segment still one edge beside an arc

- **WHEN** a Document contains one line segment and one circular arc
- **AND** the Mesh is built at the committed State
- **THEN** the segment contributes exactly one edge
- **AND** the arc contributes one or more edges all tagged with the arc's identity
