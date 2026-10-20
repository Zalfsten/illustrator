# Spec Delta

## ADDED Requirements

### Requirement: Core is independent of host platform services

Document, expression evaluation, assembly, the SolverPort and its backends, Interact, the display Mesh, and App presentation logic SHALL NOT directly use host platform services such as file systems, process control, wall-clock timers, threads, windowing, or browser APIs. They SHALL exchange persistent or external data only as in-memory values (for example bytes or text). Each implementation SHALL be able to run the same core on every host platform it targets without changing core behavior.

#### Scenario: Same sketch behaves identically on different hosts

- **WHEN** the same Document is dragged with the same pointer inputs on two host platforms targeted by one implementation
- **THEN** the committed States after each step agree within the solver's numerical tolerance
- **AND** no core component's result depends on which host it ran on

#### Scenario: Core builds for a host without blocking I/O

- **WHEN** an implementation targets a host that offers no blocking file access and no additional threads
- **THEN** the core components still build and run on that host
- **AND** only the host boundary differs between that host and the others

### Requirement: Host services pass through a single host boundary

The App SHALL obtain every host platform service (for example opening or saving files, clipboard, time, sharing) through one replaceable host boundary supplied when the App starts. A service whose result is not available at once SHALL deliver it by completion (callback, message, or equivalent) and SHALL NOT block the interactive input-solve-paint loop while waiting.

#### Scenario: File open completes later without freezing interaction

- **WHEN** the user asks to open a file and the host shows a file picker
- **THEN** the Canvas keeps repainting and accepting input while the picker is open
- **AND** the file contents reach the App as an in-memory value once the host completes the request

#### Scenario: Host boundary is swappable at startup

- **WHEN** the App starts on a different host platform
- **THEN** only the supplied host boundary differs
- **AND** Document, Interact, and the SolverPort are created the same way as on any other host

### Requirement: Optional host capabilities are discoverable

The host boundary SHALL let the App ask at runtime whether an optional capability is available on the current host. When a capability is unavailable, the App SHALL hide or disable the UI that depends on it. Using the App SHALL NOT fail because an optional capability is missing.

#### Scenario: Missing capability hides its command

- **WHEN** the current host does not provide an optional capability (for example system sharing)
- **THEN** the App does not offer the command that needs it
- **AND** all other commands keep working

#### Scenario: Available capability is offered

- **WHEN** the current host provides that optional capability
- **THEN** the App offers the corresponding command
- **AND** invoking it goes through the host boundary
