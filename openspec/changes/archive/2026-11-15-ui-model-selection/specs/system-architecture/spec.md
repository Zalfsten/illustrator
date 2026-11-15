## MODIFIED Requirements

### Requirement: App presentation mode controls construction visibility and picking

The App SHALL provide a presentation mode that shows **result geometries only** or **includes construction geometries**. The mode SHALL set the visibility filter for the Mesh used in Canvas paint and Mesh-backed hit-testing. Construction geometries hidden by the mode SHALL NOT be pickable **on the Canvas**, neither via Mesh hits nor via logical handles under that filter; other views MAY still select them. Presentation mode is UI session state; geometry role remains in Document.

#### Scenario: Toggle hides and unpicks construction

- **WHEN** the user switches from include-construction mode to result-only mode
- **AND** the Canvas repaints
- **THEN** construction geometries are not stroked
- **AND** pointer hit-testing does not select those construction geometries

#### Scenario: Mode does not alter Document roles

- **WHEN** the user toggles presentation mode
- **THEN** each geometry's result vs construction role in Document is unchanged

#### Scenario: Hidden objects stay selectable by name

- **WHEN** construction geometries are hidden
- **AND** another view selects a construction geometry by name
- **THEN** the geometry is selected although it cannot be picked on the Canvas

## ADDED Requirements

### Requirement: Hiding objects deselects them

When a change of the presentation mode hides selected objects, the App SHALL remove them from the selection, so that actions on the selection do not reach objects the user can no longer see on the Canvas. A logical point SHALL leave the selection when none of its class's endpoints remains visible.

#### Scenario: Switching to result only deselects construction

- **WHEN** a construction segment and a result segment are selected
- **AND** the user switches to result-only mode
- **THEN** only the result segment stays selected
