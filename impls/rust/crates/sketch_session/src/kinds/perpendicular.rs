//! `constrain` kind `perpendicular`.

use constraint_runtime::{Document, VectorsOrthogonal};

use super::{ConstraintCommand, Resolver, VecPair};
use crate::command::CommandError;

impl ConstraintCommand for VectorsOrthogonal {
    const TAG: &'static str = "perpendicular";
    type Params = VecPair;

    fn resolve(p: &VecPair, r: &Resolver<'_>) -> Result<Self, CommandError> {
        let (a, b) = p.resolve(r)?;
        Ok(VectorsOrthogonal { a, b })
    }

    fn params(&self, doc: &Document) -> Option<VecPair> {
        VecPair::of(doc, &self.a, &self.b)
    }
}
