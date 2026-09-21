# Spec Delta

## ADDED Requirements

### Requirement: Display Mesh is the Canvas geometry source

The system SHALL provide a display Mesh (or equivalent view-layer edge list) derived from DocumentStructure plus a concrete State \(p\). The Mesh SHALL NOT be part of Document or Interact as authoritative structure or parameters. The Canvas SHALL obtain stroked sketch geometry from that Mesh. Until richer tessellation is needed, a Mesh whose edges correspond one-to-one with line segment geometries SHALL satisfy this requirement. Hit-testing against Mesh primitives MAY be used by tools; when used, hit results SHALL carry stable geometry identities consistent with the Mesh edge tags.

#### Scenario: Canvas paints via Mesh

- **WHEN** the App paints the sketch at the committed State
- **THEN** drawable edges come from a Mesh built from DocumentStructure and that State
- **AND** the Canvas does not own a second authoritative copy of structural relationships

#### Scenario: Mesh stays outside Document authority

- **WHEN** Interact commits a new State after a successful solve
- **THEN** DocumentStructure and committed State remain the sole authorities for parameters and constraints
- **AND** any Mesh used for display is rebuilt or invalidated from that Document rather than written back into DocumentStructure

## REMOVED Requirements

### Requirement: Display tessellation is a deferred view concern

**Reason**: The Canvas must stop iterating Document geometries by kind for paint (and for Mesh-backed picks). A display Mesh is required now for that decoupling, even while only line segments exist; one-edge-per-segment tessellation is sufficient.

**Migration**: Build a Mesh from DocumentStructure + State; paint strokes from Mesh edges; optionally hit-test Mesh edges/endpoints with geometry identity tags. Do not store Mesh inside DocumentStructure.
