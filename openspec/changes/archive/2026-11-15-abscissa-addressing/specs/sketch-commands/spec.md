## ADDED Requirements

### Requirement: Expression changes keep abscissa addresses valid

A `set` that changes a function curve's expression SHALL fail with `invalid expression` if an abscissa address on that curve, used by a constraint, would become invalid (geometry-ir), and the error SHALL name every affected constraint. Changes of range or scale SHALL NOT be checked this way, because they never invalidate an abscissa.

#### Scenario: Undefined after the change

- **WHEN** `join1` joins `fn1.at(x=-1)` of sine curve `fn1` to `seg1.end`, and `set` changes the expression of `fn1` to `log(x+1)`
- **THEN** the command fails with the `invalid expression` class
- **AND** the error names `join1`, and `fn1` keeps the expression `sin(x)`

#### Scenario: Still valid after the change

- **WHEN** the same drawing gets the expression `log(x+2)` instead
- **THEN** the command succeeds and `join1` still holds within solve tolerance

#### Scenario: Range change is never checked

- **WHEN** the range of `fn1` changes to \([2, 3]\), so that \(x = -1\) lies outside it
- **THEN** the command is not rejected because of `join1`
