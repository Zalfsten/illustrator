//! `constrain` kind `equal_length`.

use constraint_runtime::{Document, EqualLength};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{ConstraintCommand, Resolver};
use crate::command::CommandError;
use crate::wire::LenAddr;

/// Two length addresses.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LenPair {
    pub a: LenAddr,
    pub b: LenAddr,
}

impl ConstraintCommand for EqualLength {
    const TAG: &'static str = "equal_length";
    type Params = LenPair;

    fn resolve(p: &LenPair, r: &Resolver<'_>) -> Result<Self, CommandError> {
        Ok(EqualLength {
            a: r.length(&p.a)?,
            b: r.length(&p.b)?,
        })
    }

    fn params(&self, doc: &Document) -> Option<LenPair> {
        Some(LenPair {
            a: doc.length_address(&self.a)?.into(),
            b: doc.length_address(&self.b)?.into(),
        })
    }
}
