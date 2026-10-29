//! `create` kind `arc`.

use constraint_runtime::{Arc, HeadingAngle, Length, Point2, TurnAngle};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{finite, finite_point, GeometryCommand};
use crate::command::CommandError;
use crate::wire;

/// Angles in radians (start heading from +x, counterclockwise sweep).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ArcParams {
    #[serde(with = "wire::point")]
    #[schemars(schema_with = "wire::point::schema")]
    pub center: Point2,
    #[serde(with = "wire::finite")]
    /// Radius, zero or positive.
    #[schemars(with = "f64", range(min = 0))]
    pub radius: f64,
    #[serde(with = "wire::finite")]
    /// Heading of the start point from the center, radians from +x.
    #[schemars(with = "f64")]
    pub start_heading: f64,
    #[serde(with = "wire::finite")]
    /// Turn from start to end, radians, counterclockwise positive.
    #[schemars(with = "f64")]
    pub sweep: f64,
}

impl GeometryCommand for Arc {
    const TAG: &'static str = "arc";
    type Params = ArcParams;

    /// A negative radius is rejected, not canonicalized: it is a solver
    /// artifact, never an input.
    fn init(p: &ArcParams) -> Result<Self::Init, CommandError> {
        if finite("radius", p.radius)? < 0.0 {
            return Err(CommandError::InvalidValue {
                field: "radius",
                reason: "negative",
            });
        }
        Ok((
            finite_point("center", p.center)?,
            Length::new(p.radius),
            HeadingAngle::new(finite("start_heading", p.start_heading)?),
            TurnAngle::new(finite("sweep", p.sweep)?),
        ))
    }

    fn params((center, radius, start, sweep): Self::Init) -> ArcParams {
        ArcParams {
            center,
            radius: radius.value(),
            start_heading: start.radians(),
            sweep: sweep.radians(),
        }
    }
}
