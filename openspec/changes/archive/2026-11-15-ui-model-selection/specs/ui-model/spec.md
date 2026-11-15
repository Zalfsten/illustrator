## Purpose

The interaction model of a front end, independent of GUI toolkit and input device: a selection shared by every view, picking, box selection and actions on the selection, all driven by device-neutral intents.

## ADDED Requirements

### Requirement: One selection shared by every view

The UI model SHALL hold one selection that every view of the front end (Canvas, object lists, property panels, tools) reads and changes. Entries SHALL be geometries, constraints or logical points (a congruence class of points). The selection SHALL identify entries by object identity, not by screen objects. Views SHALL show entries by name, and points by an address of their class.

#### Scenario: Canvas selection is visible to other views

- **WHEN** the user selects `seg1` on the Canvas
- **THEN** every other view reading the selection sees exactly `seg1`

#### Scenario: Point shown by address

- **WHEN** the end point of `seg1` is selected
- **THEN** the selection lists it by an address of its class, such as `seg1.end`

### Requirement: A logical point is one selection entry

Selecting a point SHALL select its congruence class. Two addresses of the same class SHALL denote the same entry, so toggling a point through one address SHALL remove an entry added through another address of the same class.

#### Scenario: Joined ends are one entry

- **WHEN** `seg1.end` and `seg2.start` are joined and `seg1.end` is selected
- **AND** the user toggles `seg2.start`
- **THEN** the selection no longer contains that point

### Requirement: Selection follows the drawing

After every command, undo, redo or completed drag, the selection SHALL keep the entries whose objects still exist and drop the others. A renamed object SHALL stay selected and SHALL be shown by its new name. Replacing the drawing (New, Open) SHALL clear the selection.

#### Scenario: Rename keeps the entry

- **WHEN** `seg1` is selected and renamed to `base`
- **THEN** the selection contains `base`

#### Scenario: Deleted objects leave the selection

- **WHEN** `seg1` and its constraint `par1` are selected and `seg1` is deleted
- **THEN** the selection is empty, because deleting `seg1` also deleted `par1`

#### Scenario: Undo keeps existing objects selected

- **WHEN** `seg1` is selected and the user creates `seg2`, then undoes that step
- **THEN** `seg1` is still selected

#### Scenario: Open clears the selection

- **WHEN** objects are selected and the user opens another drawing
- **THEN** the selection is empty

### Requirement: Pick at a world position

A pick intent SHALL carry a world position, a tolerance in world length and a mode (replace or toggle). It SHALL select the nearest logical point within the tolerance; only if there is none, the nearest curve within the tolerance. Replace SHALL make that object the only entry, toggle SHALL add or remove it. A replace pick on empty space SHALL clear the selection; a toggle pick there SHALL change nothing.

#### Scenario: Point wins over curve

- **WHEN** a pick lands within tolerance of both the end point of `seg1` and the curve of `seg2`
- **THEN** the point is selected, not `seg2`

#### Scenario: Toggle adds and removes

- **WHEN** `seg1` is selected and the user toggle-picks `seg2`, then toggle-picks `seg1`
- **THEN** the selection contains only `seg2`

#### Scenario: Replace pick on empty space clears

- **WHEN** objects are selected and a replace pick hits nothing
- **THEN** the selection is empty

#### Scenario: Tolerance comes from the front end

- **WHEN** the same world position is picked once with a small and once with a large tolerance, and a curve lies between the two distances
- **THEN** only the pick with the large tolerance selects the curve

### Requirement: Box selection

A box intent SHALL carry two opposite world corners and a mode (replace or add). It SHALL select every geometry visible on the Canvas whose curve lies completely inside the box. Replace SHALL make these the selection, add SHALL add them to it.

#### Scenario: Only geometries completely inside

- **WHEN** a box contains all of `seg1` and half of `seg2`
- **THEN** the box selects `seg1` and not `seg2`

#### Scenario: Add keeps the earlier selection

- **WHEN** `seg3` is selected and the user adds a box around `seg1`
- **THEN** the selection contains `seg1` and `seg3`

### Requirement: Delete the selection

A delete intent SHALL delete every selected geometry (with the constraints that reference it) and every selected constraint, as one step in the session's history. A selected point SHALL delete every curve whose endpoint lies in its congruence class. With an empty selection, it SHALL change nothing and create no step.

#### Scenario: One undo step

- **WHEN** `seg1` and `seg2` are selected and deleted
- **AND** the user undoes once
- **THEN** both segments are back

#### Scenario: A point deletes the curves at it

- **WHEN** `seg1.end` and `seg2.start` are joined, `seg3` is not, and only the point `seg1.end` is selected and deleted
- **THEN** `seg1`, `seg2` and their join are deleted in one history step
- **AND** `seg3` remains

#### Scenario: Empty selection deletes nothing

- **WHEN** nothing is selected and a delete intent arrives
- **THEN** the drawing is unchanged and no history step is created

### Requirement: Front ends drive the UI model through intents

The UI model SHALL receive device-neutral intents (pick, box, clear, delete) with positions and tolerances in world units, not device events. Each front end SHALL translate its own input devices into intents and SHALL choose the hit tolerance. The UI model SHALL NOT depend on a GUI toolkit or an input device, and its behaviour SHALL be testable without a window.

#### Scenario: Same intents, same selection

- **WHEN** two front ends send the same sequence of intents to the same drawing
- **THEN** their selections agree

#### Scenario: Tested without a window

- **WHEN** the UI model's selection rules are tested
- **THEN** the tests send intents to a session and need no window, toolkit or input device
