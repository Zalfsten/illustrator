# Spec Delta

## MODIFIED Requirements

### Requirement: Evaluation fills first-class residual and Jacobian buffers

The evaluation interface used by the solver port SHALL accept the current parameter State \(p\) and fill a first-class residual buffer \(r\) and a first-class Jacobian buffer for \(J\). The Jacobian buffer SHALL be a **contribution-blocked sparse** representation: for each assembled contribution it holds that contribution's dense local derivative block together with the residual rows and parameter columns the block maps to, and the represented global \(J\) is the sum of those mapped blocks (see system-assembly). Implementations MAY additionally offer a dense \(m \times n\) materialisation of the same \(J\) (for example for reference backends or tests). The sparsity structure (which rows and columns each block maps to) depends only on the assembled system, not on \(p\). \(r\) and \(p\) SHALL be distinct buffers: residual length is the assembled residual arity, not the parameter count. Implementations MAY expose the same contract at a C ABI as opaque user data plus raw fill pointers; adapters MAY copy between foreign buffers and the first-class buffers.

Owned backends and FFI adapters MAY allocate residual, Jacobian, and algorithm scratch once per solve (or per attached evaluation handle) and reuse those buffers across iterations or callbacks within that solve. Reuse SHALL NOT change observable residuals/Jacobians at a given \(p\).

#### Scenario: Residual and Jacobian buffer kinds

- **WHEN** the solver requests a residual evaluation at \(p\)
- **THEN** assembly writes into a residual buffer of length equal to the number of residual rows
- **AND** that buffer is not the parameter State \(p\)
- **WHEN** the solver requests a Jacobian evaluation at \(p\)
- **THEN** assembly writes each contribution's derivative block into the contribution-blocked Jacobian buffer
- **AND** the represented global \(J\) has \(m\) rows and \(n\) columns

#### Scenario: Sparsity structure is fixed for an assembled system

- **WHEN** the solver requests Jacobian evaluations at two different states \(p_1\) and \(p_2\) of the same assembled system
- **THEN** the row and column mapping of every contribution block is identical for both evaluations
- **AND** only the numeric block entries differ

#### Scenario: Dense materialisation matches the sparse representation

- **WHEN** a dense \(m \times n\) materialisation of the Jacobian is requested at \(p\)
- **THEN** each entry equals the sum of all contribution-block entries mapped to that row and column
- **AND** entries not covered by any block are zero

#### Scenario: Scratch reuse within one solve

- **WHEN** a nonlinear solve performs multiple residual or Jacobian evaluations
- **THEN** the backend or FFI adapter MAY refill previously allocated evaluation buffers instead of allocating new ones each call
- **AND** each evaluation still reflects the \(p\) supplied for that call

### Requirement: FFI-friendly evaluation contract

The evaluation interface used by the solver port SHALL be expressible in a form suitable for binding to foreign-language solvers (for example opaque user data plus residual/Jacobian fill callbacks), so that an external library can call back into assembly without requiring Rust-specific closure types at the ABI boundary. An adapter whose foreign library expects a different Jacobian layout (for example dense) SHALL convert from the contribution-blocked representation so that the foreign library sees the same global \(J\).

#### Scenario: External adapter can call assembly

- **WHEN** an external solver backend is attached
- **THEN** it can obtain residual and Jacobian fills at the current \(p\) through the port's evaluation contract
- **AND** assembly remains responsible for filling contribution blocks with their connectivity
- **AND** the adapter is responsible for converting that representation into the layout the foreign library expects

#### Scenario: Owned and external backends see the same Jacobian

- **WHEN** an owned sparse backend and an external dense backend evaluate the same assembled system at the same \(p\)
- **THEN** the global \(J\) each one consumes is identical up to floating-point rounding
