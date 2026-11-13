# Spec Delta: display-mesh

## MODIFIED Requirements

### Requirement: Curve tessellation follows curvature

Curve edges SHALL be placed according to the curve's curvature along its whole length: shorter edges where the curvature is larger, longer edges where it is smaller, and a single edge where the curvature is zero. Edge placement SHALL NOT depend only on the curve's tangents at the ends of a parameter interval, nor only on curvature sampled at isolated points. Refining the edges of one part of a curve SHALL NOT add edges to parts that already stay within the chord tolerance.

#### Scenario: Zero curvature is one edge

- **WHEN** a line segment is meshed
- **THEN** it contributes one edge

#### Scenario: Equal tangents at the ends do not hide the curve

- **WHEN** an arc with sweep 360° minus a small angle is meshed, so that its tangents at both ends almost agree
- **THEN** it contributes as many edges as the chord tolerance requires for its curvature and length, not a single chord

#### Scenario: Straight stretch beside a curved part

- **WHEN** a curve that is straight on its first part and circular on its second part is meshed with a chord tolerance small against the circle's radius
- **THEN** its edge count stays far below the per-curve edge cap
- **AND** the straight part is covered by fewer edges than the circular part

#### Scenario: A narrow curvature spike is not missed

- **WHEN** a curve whose curvature is small except on a piece much narrower than its edges would otherwise be is meshed
- **THEN** every point of the curve lies within the chord tolerance of its edge, unless the edge cap is reached
