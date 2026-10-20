# Spec Delta

## MODIFIED Requirements

### Requirement: Evaluation fills first-class residual and Jacobian buffers

The evaluation interface used by the solver port SHALL accept the current parameter State \(p\) and fill a first-class residual buffer \(r\) and a first-class Jacobian buffer for \(J\). The Jacobian buffer SHALL be a **contribution-blocked sparse** representation: for each assembled contribution it holds that contribution's dense local derivative block together with the residual rows and parameter columns the block maps to, and the represented global \(J\) is the sum of those mapped blocks (see system-assembly). Implementations MAY additionally offer a dense \(m \times n\) materialisation of the same \(J\) (for example for reference backends or tests). The sparsity structure (which rows and columns each block maps to) depends only on the assembled system, not on \(p\). \(r\) and \(p\) SHALL be distinct buffers: residual length is the assembled residual arity, not the parameter count. Adapters for external solver libraries MAY copy between the library's buffers and the first-class buffers.

Backends MAY allocate residual, Jacobian, and algorithm scratch once per solve and reuse those buffers across iterations within that solve. Reuse SHALL NOT change observable residuals/Jacobians at a given \(p\).

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
- **THEN** the backend MAY refill previously allocated evaluation buffers instead of allocating new ones each call
- **AND** each evaluation still reflects the \(p\) supplied for that call

## REMOVED Requirements

### Requirement: FFI-friendly evaluation contract

**Reason**: The product direction is in-process pure-Rust backends and a WebAssembly build. The only foreign-language consumer (Ceres) is removed. Keeping a C-ABI-expressible evaluation contract as a shared obligation adds glue code and unsafe boundaries without a user.

**Migration**: Bind external solvers behind the solver port through an implementation-specific adapter ("Owned or external backend" still allows this). If an implementation later needs a foreign-language solver, it adds the adapter and the conversion to the library's Jacobian layout in its own design, without a shared spec requirement.
