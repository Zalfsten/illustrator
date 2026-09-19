# system-assembly

## Purpose

Defines FEM-style assembly: local residual and Jacobian contributions are scattered into the global residual and global Jacobian using local→global connectivity.

## Requirements

### Requirement: Global residual is assembled from local contributions

The system SHALL form the global residual vector \(r(p)\) by evaluating each active contribution’s local residual and scattering (accumulating) its entries into global residual indices according to connectivity.

#### Scenario: Multiple constraints fill one residual vector

- **WHEN** a sketch has several structural constraints
- **AND** the residual is evaluated at state \(p\)
- **THEN** each constraint’s local residual entries appear in the global residual at the positions dictated by connectivity
- **AND** the global residual length equals the sum of the arities of active residual contributions (for the structural system without soft terms)

### Requirement: Global Jacobian is assembled from local contributions

The system SHALL form the global Jacobian \(J(p) = \partial r / \partial p\) by evaluating each contribution’s local Jacobian \(J_N\) and scattering its entries into global row/column indices via connectivity:

\(J[M_N(i), M_N(j)] \mathrel{+}= J_N[i,j]\) (up to residual-row indexing conventions).

#### Scenario: Local derivative lands at mapped global indices

- **WHEN** a local Jacobian entry \(J_N[i,j]\) is assembled
- **THEN** it is added into the global Jacobian at the row and column corresponding to the mapped residual component and the mapped global parameter indices
- **AND** parameters not involved in that contribution receive no fill from that contribution

### Requirement: Assembly is re-evaluated at the current state

Because the system is nonlinear, residual and Jacobian assembly SHALL be performed at the current \(p\) whenever the solver requests an evaluation. The system SHALL NOT treat a previously assembled Jacobian as valid for a different \(p\) unless an explicit caching contract says otherwise.

#### Scenario: Solver iteration triggers fresh assembly

- **WHEN** the solver requests residual or Jacobian evaluation at a new \(p\)
- **THEN** assembly computes contributions using that \(p\)
- **AND** the returned residual/Jacobian correspond to that state

### Requirement: Soft interaction terms assemble the same way

Temporary soft terms used during interaction (for example mouse-follow or stay-near-previous) SHALL, when active, contribute residual and Jacobian blocks that are assembled with the same connectivity/scatter rules, without becoming persistent document constraints.

#### Scenario: Soft term adds rows without persisting constraints

- **WHEN** soft interaction terms are active during a solve evaluation
- **THEN** their residual/Jacobian contributions are included in the assembled system for that evaluation
- **AND** after the interaction ends they are not retained as structural document constraints
