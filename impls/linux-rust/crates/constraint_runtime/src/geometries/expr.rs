//! Opaque point/vector expressions (Model B).
//!
//! Per-kind formula catalogs live in the kind modules (e.g. [`super::segment`]).
//! This file only wraps those catalogs so constraints and Interact never match
//! on formula kinds. Extend the outer enums when adding a geometry kind.

use crate::geom::{GeometryId, Point2, Vec2};
use crate::store::GeometryStore;

use super::segment;

/// Opaque point expression: map from State parameters to a [`Point2`].
///
/// Construct via geometry factories (e.g. [`super::Segment::start_expr`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointExpr(PointFormula);

#[derive(Clone, Copy, Debug, PartialEq)]
enum PointFormula {
    Segment(segment::PointFormula),
}

impl PointExpr {
    pub(crate) fn from_segment(formula: segment::PointFormula) -> Self {
        Self(PointFormula::Segment(formula))
    }

    pub fn evaluate(&self, store: &impl GeometryStore, p: &[f64]) -> Point2 {
        match &self.0 {
            PointFormula::Segment(f) => f.evaluate(store, p),
        }
    }

    pub fn param_indices(&self, store: &impl GeometryStore) -> Vec<usize> {
        match &self.0 {
            PointFormula::Segment(f) => f.param_indices(store),
        }
    }

    pub fn eval_point_jacobian(
        &self,
        store: &impl GeometryStore,
        p: &[f64],
        dx: &mut [f64],
        dy: &mut [f64],
    ) {
        match &self.0 {
            PointFormula::Segment(f) => f.eval_point_jacobian(store, p, dx, dy),
        }
    }

    pub fn referenced_geometry(&self) -> GeometryId {
        match &self.0 {
            PointFormula::Segment(f) => f.referenced_geometry(),
        }
    }
}

/// Opaque free-vector expression: map from State parameters to a [`Vec2`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VecExpr(VecFormula);

#[derive(Clone, Copy, Debug, PartialEq)]
enum VecFormula {
    Segment(segment::VecFormula),
}

impl VecExpr {
    pub(crate) fn from_segment(formula: segment::VecFormula) -> Self {
        Self(VecFormula::Segment(formula))
    }

    pub fn evaluate(&self, store: &impl GeometryStore, p: &[f64]) -> Vec2 {
        match &self.0 {
            VecFormula::Segment(f) => f.evaluate(store, p),
        }
    }

    pub fn referenced_geometry(&self) -> GeometryId {
        match &self.0 {
            VecFormula::Segment(f) => f.referenced_geometry(),
        }
    }
}
