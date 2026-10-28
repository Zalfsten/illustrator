# Spec Delta

## RENAMED Requirements

- FROM: `### Requirement: App owns presentation and hosts Interact`
- TO: `### Requirement: App owns presentation and edits through the command session`

## MODIFIED Requirements

### Requirement: App owns presentation and edits through the command session

The system SHALL provide an App component that presents the active sketch of a command session (sketch-commands). The App SHALL host a Canvas that displays the sketch and delivers pointer (or equivalent) input. The App SHALL change the drawing only through the session's commands and drag operations, and SHALL NOT mutate the Document directly. Interact is hosted by the session. The App SHALL NOT embed solver algorithm details; interactive solving SHALL go through Interact and SolverPort.

#### Scenario: Canvas renders from Document state

- **WHEN** the App paints the sketch
- **THEN** geometry appearance is derived from the Document's structure evaluated at a concrete State (normally the committed \(p\))
- **AND** the Canvas does not own a second authoritative copy of structural relationships

#### Scenario: Tools edit through commands

- **WHEN** a GUI tool adds a geometry or a constraint
- **THEN** the change is applied as a session command
- **AND** it appears in the session's undo history
