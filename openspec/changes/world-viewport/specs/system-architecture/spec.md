# Spec Delta

## ADDED Requirements

### Requirement: App owns a Viewport for world–screen mapping

The App SHALL own a Viewport as UI session state that maps between **world coordinates** (document/Mesh space, y-up) and **screen coordinates** (Canvas pixels). Canvas paint SHALL transform Mesh edge positions from world to screen via the Viewport. Pointer (or equivalent) input SHALL be mapped from screen to world via the Viewport before updating Document State, Interact soft targets, or issuing Mesh hit queries against world-space Mesh positions. The Viewport SHALL NOT be part of DocumentStructure or committed State.

#### Scenario: Paint uses screen positions derived from world Mesh

- **WHEN** the App paints the sketch from a Mesh built at the committed State
- **THEN** stroked edge endpoints on the Canvas are the Viewport's screen images of the Mesh edge world positions
- **AND** Document State values remain unchanged by painting

#### Scenario: Pointer updates use world coordinates

- **WHEN** the user places a point or drags with the pointer on the Canvas
- **THEN** the location passed into Document / Interact is the Viewport's world image of the screen pointer position
- **AND** that world location is not required to equal the raw screen pixel coordinates

#### Scenario: Pixel hit radius maps to world length

- **WHEN** a tool issues a Mesh hit query with a radius specified in screen pixels
- **THEN** the App converts that radius to world length using the Viewport scale before calling the Mesh hit API
- **AND** a successful hit still returns stable geometry identity from the world-space Mesh

### Requirement: World coordinates are y-up

Document State parameter blocks that encode planar point coordinates, and Mesh edge positions derived from them, SHALL use a right-handed sense in the plane with **+y upward** (mathematical/scientific convention). Screen toolkits that use +y downward SHALL obtain the flip only through the Viewport mapping, not by storing inverted y in committed State.

#### Scenario: Increasing world y moves toward the top of the Canvas after mapping

- **WHEN** two world points share the same x and the second has larger y
- **AND** both are mapped to screen through the Viewport
- **THEN** the second point's screen y is less than or equal to the first's when the Canvas uses +y downward
