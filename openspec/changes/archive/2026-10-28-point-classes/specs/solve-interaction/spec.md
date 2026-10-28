# Spec Delta

## MODIFIED Requirements

### Requirement: Logical UI points are congruence classes

A logical UI point that the user can grab SHALL correspond to one congruence class (see constraint-catalog) that contains at least one visible curve endpoint expression. Endpoints SHALL NOT be merged into one logical point because of their locations, only through Congruence; the set of logical points SHALL depend only on structure and visibility filter, not on State. Dragging a logical point SHALL apply the soft mouse target to every expression of its class; stay-near-previous soft terms SHALL NOT be applied to the State columns of those expressions for that evaluation (so stay does not fight the pointer).

#### Scenario: Dragging a shared corner moves all congruent endpoints together

- **WHEN** two segment ends are joined as one logical UI point via Congruence
- **AND** the user drags that UI point through a successful solve
- **THEN** both endpoint expressions remain congruent
- **AND** both evaluate to the updated shared location

#### Scenario: Direct Congruence partners share the soft mouse target

- **WHEN** Congruence relates point expressions \(A\) and \(B\)
- **AND** the user drags the logical UI point corresponding to \(A\)
- **THEN** the soft mouse-follow terms for that evaluation include the State columns of both \(A\) and \(B\)
- **AND** stay-near-previous soft terms omit those columns for that evaluation

#### Scenario: The whole class shares the soft mouse target

- **WHEN** Congruences relate point expressions \(A\) to \(B\) and \(B\) to \(C\)
- **AND** the user drags the logical UI point corresponding to \(A\)
- **THEN** the soft mouse-follow terms for that evaluation include the State columns of \(A\), \(B\) and \(C\)
- **AND** stay-near-previous soft terms omit those columns for that evaluation

#### Scenario: A Join chain is one logical point

- **WHEN** three segment endpoints are related by two Congruences in a chain
- **THEN** the drawing presents exactly one logical UI point for them

#### Scenario: Coincident unjoined endpoints are separate logical points

- **WHEN** the end of one segment and the start of another evaluate to the same location
- **AND** no Congruence chain relates them
- **THEN** the drawing presents two logical UI points
- **AND** dragging one of them moves only the endpoints of its own class

#### Scenario: Logical points do not change during a drag

- **WHEN** the user drags an unjoined endpoint onto or through the location of another unjoined endpoint
- **THEN** the set of logical UI points stays the same throughout the drag
