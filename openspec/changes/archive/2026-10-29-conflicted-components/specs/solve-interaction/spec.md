# Spec Delta

## ADDED Requirements

### Requirement: A drag solves only the dragged component

Interactive solves during and at the end of a drag SHALL cover only the component (constraint-catalog) that contains the dragged logical UI point. Only that component's structural residuals SHALL decide whether a solve succeeds, and other components SHALL keep their values. A conflicted component elsewhere SHALL NOT make a drag fail.

#### Scenario: Drag next to a conflicted component

- **WHEN** a drawing holds a conflicted component and a separate consistent component with a free endpoint
- **AND** the user drags that endpoint through successful solves
- **THEN** the endpoint moves toward the pointer under its component's constraints
- **AND** the values of the conflicted component are unchanged

### Requirement: Frozen components cannot be dragged

A drag SHALL NOT start on a logical UI point whose congruence class lies in a conflicted component. The attempt SHALL leave values unchanged, SHALL NOT create a history step, and SHALL report that the component is frozen to the host, which decides how to present it.

#### Scenario: Pointer-down on a frozen point

- **WHEN** a logical UI point belongs to a conflicted component
- **AND** a drag is started on it
- **THEN** no drag is active
- **AND** the drawing and the history are unchanged
- **AND** the host receives the frozen indication
