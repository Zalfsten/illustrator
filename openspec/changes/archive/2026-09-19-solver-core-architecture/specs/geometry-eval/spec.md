# Spec Delta

## Purpose

Defines that geometries (and derived constraint contributions) expose evaluable quantities and analytical derivatives with respect to their local parameters for use in residual and Jacobian assembly.

## ADDED Requirements

### Requirement: Geometries expose evaluation functions

A geometry SHALL provide evaluation functions that map its local parameters to geometric quantities needed by the system (for example endpoint locations, direction vectors, or other chart-defined outputs). These functions express results in terms of the geometry’s local parameterization.

#### Scenario: Endpoint evaluation depends on local parameters

- **WHEN** a line segment geometry is asked for an endpoint location
- **THEN** the result is computed from that segment’s local parameters
- **AND** those local parameters are the ones referenced in the global state vector

### Requirement: Geometries expose derivatives w.r.t. local parameters

A geometry SHALL provide analytical derivatives of its exposed quantities with respect to its local parameters (or equivalent derivative information sufficient to form local Jacobian contributions). Finite-difference approximation MAY be used only as a fallback or verification aid, not as the sole specified primary path for production solves.

#### Scenario: Local Jacobian contribution can be formed analytically

- **WHEN** a residual contribution depends on a geometry quantity
- **AND** assembly requests derivatives of that residual w.r.t. the geometry’s local parameters
- **THEN** the system can obtain those derivatives from analytical geometry/constraint derivative information
- **AND** the resulting local Jacobian block is defined with respect to the geometry’s local parameter ordering

### Requirement: Constraints contribute residuals and local Jacobians

Each structural constraint SHALL define a residual contribution \(r_N\) as a function of the local parameters of the geometries it involves (via their quantities), and SHALL define the corresponding local Jacobian \(J_N = \partial r_N / \partial p_N\) (or blocks thereof) with respect to those local parameters.

#### Scenario: Orthogonal constraint yields residual and local derivatives

- **WHEN** an Orthogonal constraint between two segments is assembled at a given state
- **THEN** it produces a residual contribution of the specified arity
- **AND** it produces local partial derivatives of that residual w.r.t. the involved local parameters
