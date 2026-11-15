## ADDED Requirements

### Requirement: Command groups are one history step

The session SHALL apply a list of commands as a group: in order, each command seeing the result of the ones before. If every command succeeds, the group SHALL be one step in the session's history. If one fails, the drawing SHALL be left exactly as before the group, no step SHALL be created, and the error SHALL name the failing command's position. A group SHALL NOT be applied while a drag is in progress.

#### Scenario: Group undoes in one step

- **WHEN** a group deletes `seg1` and then `seg2`
- **AND** the user undoes once
- **THEN** both segments are back

#### Scenario: A failing command rolls back the group

- **WHEN** a group deletes `seg1` and then a name that does not exist
- **THEN** the group fails naming the second command
- **AND** `seg1` still exists and no history step was created

#### Scenario: Later commands see earlier results

- **WHEN** a group creates a segment named `a` and then renames `a` to `b`
- **THEN** the drawing contains `b` and not `a`
