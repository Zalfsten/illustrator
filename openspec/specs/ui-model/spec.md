# ui-model Specification

## Purpose

The interaction model of a front end, independent of GUI toolkit and input device: a selection shared by every view, picking, box selection and actions on the selection, all driven by device-neutral intents.

## Requirements

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

### Requirement: Tools declare slots by capability

A tool SHALL declare an ordered list of slots, each asking for one input by capability, not by geometry kind: a free position, a point (a logical point), a direction (a geometry with a direction, such as a line), or a curve. A tool SHALL issue its command when every slot is filled. Selecting (no slots) SHALL be the default tool.

#### Scenario: Parallel asks for two directions

- **WHEN** the Parallel tool is chosen with an empty selection
- **THEN** it has two empty direction slots and issues nothing yet

#### Scenario: An arc does not fill a direction slot

- **WHEN** the Parallel tool waits for a direction and the user picks an arc
- **THEN** the slot stays empty and the user is told that a direction is needed

### Requirement: Choosing a tool pre-fills its slots from the selection

Choosing a tool SHALL move selection entries, in selection order, into the first empty slot whose capability they have; entries that fit no slot SHALL be ignored. The selection SHALL be empty afterwards. If every slot is filled, the tool SHALL issue its command at once.

#### Scenario: Select first, then Parallel

- **WHEN** `seg1` and `seg2` are selected and the user chooses Parallel
- **THEN** a parallel constraint between `seg1` and `seg2` is added at once, without further picks
- **AND** the selection is empty

#### Scenario: Partly filled from the selection

- **WHEN** only `seg1` is selected and the user chooses Parallel
- **THEN** the first slot holds `seg1` and the tool waits for the second direction

#### Scenario: Unfitting entries are ignored

- **WHEN** an arc and `seg1` are selected and the user chooses Parallel
- **THEN** the first slot holds `seg1` and the arc is not used

### Requirement: Picks fill the next empty slot

While a tool other than selecting is active, a pick SHALL offer the nearest object of the next empty slot's capability within the tolerance, or for a position slot the picked world position. An object already held by a slot of the same tool SHALL be refused. A pick that finds nothing fitting SHALL leave the slots unchanged.

#### Scenario: Tool first

- **WHEN** the user chooses Join and picks `seg1.end`, then `seg2.start`
- **THEN** a join of the two points is added

#### Scenario: The same object twice is refused

- **WHEN** the Parallel tool holds `seg1` and the user picks `seg1` again
- **THEN** the second slot stays empty

### Requirement: A finished tool stays active

After issuing its command, a tool SHALL stay active with empty slots, ready for the next set of inputs. If the session rejects the command, the drawing SHALL be unchanged, the error SHALL be reported, and the tool SHALL stay active with empty slots.

#### Scenario: Two constraints in a row

- **WHEN** the Fix tool is active and the user picks `seg1.start`, then `seg2.end`
- **THEN** both points get a fix constraint and Fix is still active

#### Scenario: Rejected command keeps the tool

- **WHEN** `seg1.start` and `seg2.end` are fixed at different locations, the Join tool is active and the user picks both points
- **THEN** the drawing is unchanged, the rejection is reported and Join is still active with empty slots

### Requirement: Cancel ends a tool or clears the selection

A cancel intent SHALL end an active tool other than selecting, drop its filled slots without issuing anything, and make selecting the active tool. With selecting active, cancel SHALL clear the selection.

#### Scenario: Cancel a half-filled tool

- **WHEN** the Parallel tool holds `seg1` and the user cancels
- **THEN** no constraint is added and selecting is the active tool

### Requirement: The UI model prompts for the next input

The UI model SHALL provide a prompt for the active tool that names the tool, the inputs already filled and the capability of the next empty slot, so that every front end can show what is expected next.

#### Scenario: Prompt names the next slot

- **WHEN** the Parallel tool holds `seg1`
- **THEN** the prompt names Parallel, `seg1` and that a direction is expected next

### Requirement: Front ends drive the UI model through intents

The UI model SHALL receive device-neutral intents (pick, box, clear, delete, choose a tool, cancel) with positions and tolerances in world units, not device events. Each front end SHALL translate its own input devices into intents and SHALL choose the hit tolerance. The UI model SHALL NOT depend on a GUI toolkit or an input device, and its behaviour SHALL be testable without a window.

#### Scenario: Same intents, same selection

- **WHEN** two front ends send the same sequence of intents to the same drawing
- **THEN** their selections agree

#### Scenario: Tested without a window

- **WHEN** the UI model's selection rules are tested
- **THEN** the tests send intents to a session and need no window, toolkit or input device

#### Scenario: Same intents, same commands

- **WHEN** two front ends choose the same tool and send the same picks to the same drawing
- **THEN** the same commands are issued and the drawings agree
