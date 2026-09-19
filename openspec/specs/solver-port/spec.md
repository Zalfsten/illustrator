# solver-port

## Purpose

Defines a swappable nonlinear solver port that consumes residual/Jacobian evaluations and exposes common solver controls, whether backed by an owned implementation or an external library.

## Requirements

### Requirement: Abstract solver port

The system SHALL expose an abstract solver port that drives updates to the global state vector \(p\) by requesting residual evaluations and Jacobian evaluations (or an agreed equivalent such as Jacobian-vector products) through callbacks or an equivalent evaluation interface—not by receiving a single static matrix as the sole problem definition for a nonlinear solve.

#### Scenario: Solver requests eval callbacks each iteration

- **WHEN** a nonlinear solve runs
- **THEN** the solver port repeatedly obtains residual information at the current \(p\)
- **AND** obtains Jacobian information (analytical assembly or an explicitly configured equivalent) at the current \(p\) as required by the chosen algorithm
- **AND** updates \(p\) toward satisfying the assembled structural residuals

### Requirement: Owned or external backend

Implementations SHALL be able to satisfy the solver port either with an owned numerical backend or by adapting an external solver library behind the same port. Spec-level behavior SHALL NOT require a particular library or algorithm name.

#### Scenario: Backend can be substituted

- **WHEN** one solver backend conforming to the port is replaced by another
- **THEN** structural solve semantics (invariants, tolerances as configured) remain expressible through the same port
- **AND** no requirement mandates a specific vendor library

### Requirement: Common solver parameters

The solver port SHALL support configuration of common nonlinear-solve controls, including at least: convergence tolerance on residual norms (or an equivalent stopping criterion), maximum iterations, and algorithm/damping-related options applicable to the backend (for example step damping or trust-region / Levenberg–Marquardt style parameters when the backend uses them). Backends SHALL document which parameters they honor; unsupported parameters SHALL fail closed (reject or clearly ignore with observable signaling)—implementations MUST NOT silently claim success while ignoring a required tolerance setting.

#### Scenario: Tolerance and iteration limit are configurable

- **WHEN** a caller sets a residual tolerance and a maximum iteration count on the solver port
- **THEN** the active backend uses those settings in its stopping criteria (or rejects the configuration if it cannot)
- **AND** a successful solve reports satisfaction within the configured residual tolerance for structural residuals

### Requirement: FFI-friendly evaluation contract

The evaluation interface used by the solver port SHALL be expressible in a form suitable for binding to foreign-language solvers (for example opaque user data plus residual/Jacobian fill callbacks), so that an external library can call back into assembly without requiring Rust-specific closure types at the ABI boundary.

#### Scenario: External adapter can call assembly

- **WHEN** an external solver backend is attached
- **THEN** it can obtain residual and Jacobian fills at the current \(p\) through the port’s evaluation contract
- **AND** assembly remains responsible for connectivity-based scatter into the buffers the backend expects
