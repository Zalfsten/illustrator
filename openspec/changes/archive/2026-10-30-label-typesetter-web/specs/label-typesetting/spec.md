# Spec Delta

## ADDED Requirements

### Requirement: Pending labels stay visible

While any line of a label waits for typesetting, the label SHALL remain visible at its placement. A label that was typeset completely before SHALL keep showing its last complete rendering, placed by its current frame. Otherwise it SHALL be shown in the not-typeset form, marked as pending so that it can be told apart from a failure. The App SHALL indicate that typesetting is in progress.

#### Scenario: Typesetter still loading

- **WHEN** a drawing with a label is shown while the typesetter is loading
- **THEN** the label is shown at its placement in the not-typeset form, marked as pending
- **AND** the App indicates that the typesetter is loading

#### Scenario: Changed line keeps the last rendering

- **WHEN** a typeset label's source changes and its new line is not typeset yet
- **THEN** the label keeps showing its previous complete rendering at its current placement
- **AND** shows the new rendering once every line is typeset

#### Scenario: Label moves while pending

- **WHEN** the geometry a pending label is joined to is dragged
- **THEN** the label's shown form follows the geometry like a typeset label

### Requirement: Typesetting never blocks interaction

Requests for typesetting SHALL NOT block the input–solve–paint loop. The App SHALL stay fully usable while the typesetter loads or works. A completed result SHALL cause a repaint without further input.

#### Scenario: Drag while loading

- **WHEN** the typesetter is loading and the user drags a point
- **THEN** the drawing follows the pointer as without labels

#### Scenario: Result without input

- **WHEN** a requested line completes while the pointer does not move
- **THEN** the label is repainted with its typeset form

## MODIFIED Requirements

### Requirement: Typesetting is a replaceable backend

Typesetting SHALL be reached through one port that takes source, markup, size and preamble and delivers lines or a failure by completion. A request SHALL return at once; its result MAY arrive in the same frame or later. The port SHALL report the typesetter's state: loading, ready, or failed. The drawing model, the command session and label placement SHALL NOT depend on a particular typesetting engine. An implementation MAY provide different typesetters on different hosts, or none.

#### Scenario: Placement without an engine

- **WHEN** label placement is computed from lines supplied by a test typesetter with fixed metrics
- **THEN** the placed positions follow the labels spec for those metrics, without any typesetting engine present

#### Scenario: Immediate typesetter

- **WHEN** a typesetter that answers at once is asked for a line
- **THEN** the line's result is available in the same frame
- **AND** the label is shown typeset without a pending form

#### Scenario: Typesetter state is queryable

- **WHEN** the typesetter has not finished loading
- **THEN** its state is loading
- **AND** it is ready once it can deliver results, or failed if it cannot be loaded

### Requirement: Labels that cannot be typeset stay visible

A label whose source fails to typeset, that is shown where no typesetter is available, or whose typesetter failed to load, SHALL remain visible at its placement and SHALL be marked as not typeset, so that it can be told apart from a typeset label. Its source text SHALL be readable in that form.

#### Scenario: Failure is marked in place

- **WHEN** a label's source is `$\foo$`
- **THEN** the label is shown at its placement, marked as not typeset, with its source readable

#### Scenario: No typesetter available

- **WHEN** the App runs without a typesetter
- **THEN** every label is shown marked as not typeset, at its placement

#### Scenario: Typesetter fails to load

- **WHEN** the typesetter cannot be loaded (for example without a network connection)
- **THEN** every label is shown marked as not typeset, at its placement
- **AND** the App says that the typesetter could not be loaded
