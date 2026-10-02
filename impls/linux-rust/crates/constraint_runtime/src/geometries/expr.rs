//! Opaque point/vector expressions (Model B).
//!
//! Per-kind formula catalogs live in the kind modules (e.g. [`super::segment`]).
//! This file wraps those catalogs and owns the geometry identity so constraints
//! and Interact never match on formula kinds. Store→chart resolution happens
//! here; kind formulas stay pure in unpacked chart data.
//!
//! Stack Jacobian carriers live in [`crate::geom`] ([`PointJacobian`] /
//! [`VecJacobian`]); this module only returns them from evaluate helpers.

use super::shell::GeometryId;
use crate::geom::{Point2, PointJacobian, Vec2, VecJacobian};
use crate::state::State;
use crate::store::GeometryStore;

use super::segment::{self, Segment};

/// Opaque point expression: map from State parameters to a [`Point2`].
///
/// Construct via geometry factories (e.g. [`super::Segment::start_expr`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointExpr {
    geometry: GeometryId,
    formula: PointFormula,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum PointFormula {
    Segment(segment::PointFormula),
}

impl From<(GeometryId, segment::PointFormula)> for PointExpr {
    fn from((geometry, formula): (GeometryId, segment::PointFormula)) -> Self {
        Self {
            geometry,
            formula: PointFormula::Segment(formula),
        }
    }
}

impl PointExpr {
    pub fn evaluate(&self, store: &impl GeometryStore, p: &State) -> Point2 {
        match &self.formula {
            PointFormula::Segment(f) => {
                let init = store.unpack_chart::<Segment>(self.geometry, p);
                f.evaluate(init)
            }
        }
    }

    pub fn param_indices(&self, store: &impl GeometryStore) -> Vec<usize> {
        match &self.formula {
            PointFormula::Segment(f) => {
                let block = store.chart_param_block(self.geometry);
                f.param_indices(block)
            }
        }
    }

    pub fn eval_point_jacobian(&self, _store: &impl GeometryStore, _p: &State) -> PointJacobian {
        match &self.formula {
            PointFormula::Segment(f) => f.eval_point_jacobian(),
        }
    }

    pub fn referenced_geometry(&self) -> GeometryId {
        self.geometry
    }
}

/// Opaque free-vector expression: map from State parameters to a [`Vec2`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VecExpr {
    geometry: GeometryId,
    formula: VecFormula,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum VecFormula {
    Segment(segment::VecFormula),
}

impl From<(GeometryId, segment::VecFormula)> for VecExpr {
    fn from((geometry, formula): (GeometryId, segment::VecFormula)) -> Self {
        Self {
            geometry,
            formula: VecFormula::Segment(formula),
        }
    }
}

impl VecExpr {
    pub fn evaluate(&self, store: &impl GeometryStore, p: &State) -> Vec2 {
        match &self.formula {
            VecFormula::Segment(f) => {
                let init = store.unpack_chart::<Segment>(self.geometry, p);
                f.evaluate(init)
            }
        }
    }

    pub fn param_indices(&self, store: &impl GeometryStore) -> Vec<usize> {
        match &self.formula {
            VecFormula::Segment(f) => {
                let block = store.chart_param_block(self.geometry);
                f.param_indices(block)
            }
        }
    }

    pub fn eval_vec_jacobian(&self, _store: &impl GeometryStore, _p: &State) -> VecJacobian {
        match &self.formula {
            VecFormula::Segment(f) => f.eval_vec_jacobian(),
        }
    }

    pub fn referenced_geometry(&self) -> GeometryId {
        self.geometry
    }
}
