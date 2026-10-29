//! `constrain` kind `join` (Congruence).

use constraint_runtime::{Congruence, Document};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{ConstraintCommand, Resolver};
use crate::command::CommandError;
use crate::wire::PointAddr;

/// Two point addresses.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PointPair {
    pub a: PointAddr,
    pub b: PointAddr,
}

impl ConstraintCommand for Congruence {
    const TAG: &'static str = "join";
    type Params = PointPair;

    fn resolve(p: &PointPair, r: &Resolver<'_>) -> Result<Self, CommandError> {
        let c = Congruence {
            a: r.point(&p.a)?,
            b: r.point(&p.b)?,
        };
        if r.document().point_class(&c.a).contains(&c.b) {
            return Err(CommandError::AlreadyJoined {
                a: p.a.to_string(),
                b: p.b.to_string(),
            });
        }
        Ok(c)
    }

    fn params(&self, doc: &Document) -> Option<PointPair> {
        Some(PointPair {
            a: doc.point_address(&self.a)?.into(),
            b: doc.point_address(&self.b)?.into(),
        })
    }
}
