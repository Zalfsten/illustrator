//! Opaque point/vector expressions (Model B).
//!
//! Per-kind formula catalogs live in the kind modules (e.g. [`super::segment`]).
//! This file wraps those catalogs and owns the geometry identity so constraints
//! and Interact never match on formula kinds. Store→chart resolution happens
//! here; kind formulas stay pure in unpacked chart data.

use super::shell::GeometryId;
use crate::geom::{Point2, Vec2};
use crate::state::State;
use crate::store::GeometryStore;

use super::segment::{self, Segment};

/// Max width (local parameters) of a single point-formula Jacobian.
/// Chart formulas stay small (segment endpoint: 2, on-curve: 4); raise when a
/// kind needs more formula-local DOFs.
pub const MAX_POINT_JACOBIAN_PARAMS: usize = 8;

/// Analytical Jacobian of a point expression: \(2 \times n\) stacked as two
/// rows `dx`, `dy` with \(\texttt{dx}[i]=\partial x/\partial p_i\),
/// \(\texttt{dy}[i]=\partial y/\partial p_i\). Stored on the stack; `n` is the
/// formula-local parameter count (same order as [`PointExpr::param_indices`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointJacobian {
    dx: [f64; MAX_POINT_JACOBIAN_PARAMS],
    dy: [f64; MAX_POINT_JACOBIAN_PARAMS],
    n: usize,
}

impl PointJacobian {
    /// Zero \(2 \times n\) Jacobian (`n` must be ≤ [`MAX_POINT_JACOBIAN_PARAMS`]).
    pub fn zeros(n: usize) -> Self {
        assert!(
            n <= MAX_POINT_JACOBIAN_PARAMS,
            "point Jacobian width {n} exceeds MAX_POINT_JACOBIAN_PARAMS ({MAX_POINT_JACOBIAN_PARAMS})"
        );
        Self {
            dx: [0.0; MAX_POINT_JACOBIAN_PARAMS],
            dy: [0.0; MAX_POINT_JACOBIAN_PARAMS],
            n,
        }
    }

    /// Identity map \((p_0,p_1)\mapsto(x,y)\): \(J = I_2\).
    pub fn endpoint_identity() -> Self {
        let mut j = Self::zeros(2);
        j.dx[0] = 1.0;
        j.dy[1] = 1.0;
        j
    }

    pub fn n(&self) -> usize {
        self.n
    }

    pub fn dx(&self) -> &[f64] {
        &self.dx[..self.n]
    }

    pub fn dy(&self) -> &[f64] {
        &self.dy[..self.n]
    }

    /// Set column `i`: \(\partial(x,y)/\partial p_i = (\partial x, \partial y)\).
    pub(crate) fn set_dx_dy(&mut self, i: usize, dx_i: f64, dy_i: f64) {
        assert!(i < self.n);
        self.dx[i] = dx_i;
        self.dy[i] = dy_i;
    }
}

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

/// Analytical Jacobian of a free-vector expression: same \(2 \times n\) layout as
/// [`PointJacobian`] (rows \(\partial v_x\), \(\partial v_y\)).
pub type VecJacobian = PointJacobian;

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
