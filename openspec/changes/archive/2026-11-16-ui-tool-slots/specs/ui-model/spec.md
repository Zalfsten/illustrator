## ADDED Requirements

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

## MODIFIED Requirements

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
