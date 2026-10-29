//! `constrain` kind `parallel`.

use constraint_runtime::{Document, VecExpr, VectorsParallel};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{ConstraintCommand, Resolver};
use crate::command::CommandError;
use crate::wire::VecAddr;

/// Two free-vector addresses.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VecPair {
    pub a: VecAddr,
    pub b: VecAddr,
}

impl VecPair {
    pub(super) fn resolve(&self, r: &Resolver<'_>) -> Result<(VecExpr, VecExpr), CommandError> {
        Ok((r.vector(&self.a)?, r.vector(&self.b)?))
    }

    pub(super) fn of(doc: &Document, a: &VecExpr, b: &VecExpr) -> Option<Self> {
        Some(Self {
            a: doc.vec_address(a)?.into(),
            b: doc.vec_address(b)?.into(),
        })
    }
}

impl ConstraintCommand for VectorsParallel {
    const TAG: &'static str = "parallel";
    type Params = VecPair;

    fn resolve(p: &VecPair, r: &Resolver<'_>) -> Result<Self, CommandError> {
        let (a, b) = p.resolve(r)?;
        Ok(VectorsParallel { a, b })
    }

    fn params(&self, doc: &Document) -> Option<VecPair> {
        VecPair::of(doc, &self.a, &self.b)
    }
}
