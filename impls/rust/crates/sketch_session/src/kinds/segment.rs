//! `create` kind `segment`.

use constraint_runtime::{KindAttrs, Point2, Segment};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{finite_point, GeometryCommand};
use crate::command::CommandError;
use crate::wire;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SegmentParams {
    #[serde(with = "wire::point")]
    #[schemars(schema_with = "wire::point::schema")]
    pub start: Point2,
    #[serde(with = "wire::point")]
    #[schemars(schema_with = "wire::point::schema")]
    pub end: Point2,
}

impl GeometryCommand for Segment {
    const TAG: &'static str = "segment";
    type Params = SegmentParams;

    fn init(p: &SegmentParams) -> Result<(Point2, Point2), CommandError> {
        Ok((finite_point("start", p.start)?, finite_point("end", p.end)?))
    }

    fn params((start, end): (Point2, Point2), _attrs: &KindAttrs) -> SegmentParams {
        SegmentParams { start, end }
    }
}
