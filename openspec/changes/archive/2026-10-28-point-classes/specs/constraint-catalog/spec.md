# Spec Delta

## ADDED Requirements

### Requirement: Congruence classes of point expressions are enumerable

Given a point expression \(P\), the system SHALL enumerate its congruence class: \(P\) together with every point expression reachable from \(P\) through a chain of structural Congruence relationships (the transitive closure). The class SHALL depend only on the Congruence relationships in Document, not on evaluated locations, and SHALL NOT include point expressions that are merely at the same location.

#### Scenario: A Join chain forms one class

- **WHEN** Congruence relates \(A\) to \(B\) and another Congruence relates \(B\) to \(C\)
- **AND** the class of \(A\) is requested
- **THEN** the class contains \(A\), \(B\) and \(C\)
- **AND** the classes of \(B\) and \(C\) are the same set

#### Scenario: Coincident unrelated points stay in separate classes

- **WHEN** point expressions \(A\) and \(B\) evaluate to the same location
- **AND** no chain of Congruence relates them
- **THEN** the class of \(A\) does not contain \(B\)

#### Scenario: Removing a link splits the class

- **WHEN** Congruences relate \(A\) to \(B\) and \(B\) to \(C\)
- **AND** the Congruence between \(B\) and \(C\) is removed
- **THEN** the class of \(A\) is \(A\) and \(B\)
- **AND** the class of \(C\) is \(C\) alone
