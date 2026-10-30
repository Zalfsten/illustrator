# Spec Delta

## MODIFIED Requirements

### Requirement: Optional host capabilities are discoverable

The host boundary SHALL let the App ask at runtime whether an optional capability is available on the current host. When a capability is unavailable, the App SHALL hide or disable the UI that depends on it. Using the App SHALL NOT fail because an optional capability is missing. Label typesetting SHALL be such an optional capability, provided through the host boundary.

#### Scenario: Missing capability hides its command

- **WHEN** the current host does not provide an optional capability (for example system sharing)
- **THEN** the App does not offer the command that needs it
- **AND** all other commands keep working

#### Scenario: Available capability is offered

- **WHEN** the current host provides that optional capability
- **THEN** the App offers the corresponding command
- **AND** invoking it goes through the host boundary

#### Scenario: Typesetter comes from the host boundary

- **WHEN** the App starts on a host that provides a typesetter
- **THEN** the App obtains it from the host boundary supplied at start
- **AND** on a host without one, labels are shown in the not-typeset form and everything else works
