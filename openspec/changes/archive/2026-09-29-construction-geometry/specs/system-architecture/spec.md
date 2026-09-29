# Spec Delta

## ADDED Requirements

### Requirement: App presentation mode controls construction visibility and picking

The App SHALL provide a presentation mode that either shows **result geometries only** or **includes construction geometries**. That mode SHALL determine the visibility filter used when building the Mesh for Canvas paint and for Mesh-backed hit-testing. When construction geometries are not shown in the current mode, they SHALL NOT be pickable via Mesh hits or via logical handles derived under the same filter. Presentation mode is UI session state; geometry role remains in DocumentStructure.

#### Scenario: Toggle hides and unpicks construction

- **WHEN** the user switches from include-construction mode to result-only mode
- **AND** the Canvas repaints
- **THEN** construction geometries are not stroked
- **AND** pointer hit-testing does not select those construction geometries

#### Scenario: Mode does not alter Document roles

- **WHEN** the user toggles presentation mode
- **THEN** each geometry's result vs construction role in DocumentStructure is unchanged
