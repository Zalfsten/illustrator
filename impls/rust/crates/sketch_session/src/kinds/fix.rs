//! `constrain` kind `fix`.

use constraint_runtime::{Document, FixPoint, Point2};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{finite_point, ConstraintCommand, Resolver};
use crate::command::CommandError;
use crate::wire::{self, PointAddr};

/// A point address and its target location.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FixParams {
    pub point: PointAddr,
    #[serde(with = "wire::point")]
    #[schemars(schema_with = "wire::point::schema")]
    pub target: Point2,
}

impl ConstraintCommand for FixPoint {
    const TAG: &'static str = "fix";
    type Params = FixParams;

    fn resolve(p: &FixParams, r: &Resolver<'_>) -> Result<Self, CommandError> {
        Ok(FixPoint {
            point: r.point(&p.point)?,
            target: finite_point("target", p.target)?,
        })
    }

    fn params(&self, doc: &Document) -> Option<FixParams> {
        Some(FixParams {
            point: doc.point_address(&self.point)?.into(),
            target: self.target,
        })
    }
}
