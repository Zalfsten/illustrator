//! Opaque point/vector/scalar expressions (Model B).
//!
//! Shared formula *tags* live in [`super::capabilities`] (curve vs endpoint catalogs).
//! This module keys them by geometry kind for evaluate dispatch and owns the
//! geometry identity so constraints and Interact never match on formula kinds.
//! Evaluate resolves the id once, then unpacks via [`GeometryStore`] helpers.
//!
//! Stack Jacobian carriers live in [`crate::geom`] ([`PointJacobian`] /
//! [`VecJacobian`] / [`ScalarJacobian`]); this module only returns them from
//! evaluate helpers.

use super::capabilities::{
    CurvePointFormula, CurveScalarFormula, EndpointPointFormula, VecFormula,
};
use super::shell::GeometryId;
use crate::geom::{Point2, PointJacobian, ScalarJacobian, Vec2, VecJacobian};
use crate::geometry_kinds::Segment;
use crate::state::State;
use crate::store::GeometryStore;

/// Opaque point expression: map from State parameters to a [`Point2`].
///
/// Construct via capability factories (e.g. [`super::EndpointExprs::start_expr`] on a kind).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointExpr {
    geometry: GeometryId,
    formula: PointFormulaKeyed,
}

/// Kind-keyed wrapper around curve / endpoint formula catalogs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum PointFormulaKeyed {
    SegmentCurve(CurvePointFormula),
    SegmentEndpoint(EndpointPointFormula),
}

impl PointExpr {
    pub(crate) fn new(geometry: GeometryId, formula: PointFormulaKeyed) -> Self {
        Self { geometry, formula }
    }

    pub fn evaluate(&self, store: &impl GeometryStore, p: &State) -> Point2 {
        let g = store.expect_geometry(self.geometry);
        let chart = store.unpack_chart::<Segment>(g, p);
        match &self.formula {
            PointFormulaKeyed::SegmentCurve(f) => Segment::eval_curve_point(*f, chart),
            PointFormulaKeyed::SegmentEndpoint(f) => Segment::eval_endpoint_point(*f, chart),
        }
    }

    pub fn param_indices(&self, store: &impl GeometryStore) -> Vec<usize> {
        let g = store.expect_geometry(self.geometry);
        let block = store.chart_param_block(g);
        match &self.formula {
            PointFormulaKeyed::SegmentCurve(f) => Segment::curve_point_param_indices(*f, block),
            PointFormulaKeyed::SegmentEndpoint(f) => {
                Segment::endpoint_point_param_indices(*f, block)
            }
        }
    }

    pub fn eval_point_jacobian(&self, _store: &impl GeometryStore, _p: &State) -> PointJacobian {
        match &self.formula {
            PointFormulaKeyed::SegmentCurve(f) => Segment::eval_curve_point_jacobian(*f),
            PointFormulaKeyed::SegmentEndpoint(f) => Segment::eval_endpoint_point_jacobian(*f),
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
    formula: VecFormulaKeyed,
}

/// Kind-keyed wrapper around the shared curve [`VecFormula`] catalog.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum VecFormulaKeyed {
    Segment(VecFormula),
}

impl VecExpr {
    pub(crate) fn new(geometry: GeometryId, formula: VecFormulaKeyed) -> Self {
        Self { geometry, formula }
    }

    pub fn evaluate(&self, store: &impl GeometryStore, p: &State) -> Vec2 {
        let g = store.expect_geometry(self.geometry);
        match &self.formula {
            VecFormulaKeyed::Segment(f) => {
                Segment::eval_vec(*f, store.unpack_chart::<Segment>(g, p))
            }
        }
    }

    pub fn param_indices(&self, store: &impl GeometryStore) -> Vec<usize> {
        let g = store.expect_geometry(self.geometry);
        match &self.formula {
            VecFormulaKeyed::Segment(f) => {
                Segment::vec_param_indices(*f, store.chart_param_block(g))
            }
        }
    }

    pub fn eval_vec_jacobian(&self, _store: &impl GeometryStore, _p: &State) -> VecJacobian {
        match &self.formula {
            VecFormulaKeyed::Segment(f) => Segment::eval_vec_jacobian(*f),
        }
    }

    pub fn referenced_geometry(&self) -> GeometryId {
        self.geometry
    }
}

/// Opaque scalar expression: map from State parameters to an `f64`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScalarExpr {
    geometry: GeometryId,
    formula: ScalarFormulaKeyed,
}

/// Kind-keyed wrapper around the shared curve [`CurveScalarFormula`] catalog.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum ScalarFormulaKeyed {
    Segment(CurveScalarFormula),
}

impl ScalarExpr {
    pub(crate) fn new(geometry: GeometryId, formula: ScalarFormulaKeyed) -> Self {
        Self { geometry, formula }
    }

    pub fn evaluate(&self, store: &impl GeometryStore, p: &State) -> f64 {
        let g = store.expect_geometry(self.geometry);
        match &self.formula {
            ScalarFormulaKeyed::Segment(f) => {
                Segment::eval_scalar(*f, store.unpack_chart::<Segment>(g, p))
            }
        }
    }

    pub fn param_indices(&self, store: &impl GeometryStore) -> Vec<usize> {
        let g = store.expect_geometry(self.geometry);
        match &self.formula {
            ScalarFormulaKeyed::Segment(f) => {
                Segment::scalar_param_indices(*f, store.chart_param_block(g))
            }
        }
    }

    pub fn eval_scalar_jacobian(&self, store: &impl GeometryStore, p: &State) -> ScalarJacobian {
        let g = store.expect_geometry(self.geometry);
        match &self.formula {
            ScalarFormulaKeyed::Segment(f) => {
                Segment::eval_scalar_jacobian(*f, store.unpack_chart::<Segment>(g, p))
            }
        }
    }

    pub fn referenced_geometry(&self) -> GeometryId {
        self.geometry
    }
}
