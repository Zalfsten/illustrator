//! Opaque point/vector expressions (Model B).
//!
//! Per-kind formula catalogs live in the kind modules (e.g. [`super::segment`]).
//! This file only wraps those catalogs so constraints and Interact never match
//! on formula kinds. Extend the outer enums when adding a geometry kind.

use crate::geom::{GeometryId, Point2, Vec2};
use crate::state::State;
use crate::store::GeometryStore;

use super::segment;

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
pub struct PointExpr(PointFormula);

#[derive(Clone, Copy, Debug, PartialEq)]
enum PointFormula {
    Segment(segment::PointFormula),
}

impl From<segment::PointFormula> for PointExpr {
    fn from(formula: segment::PointFormula) -> Self {
        Self(PointFormula::Segment(formula))
    }
}

impl PointExpr {
    pub fn evaluate(&self, store: &impl GeometryStore, p: &State) -> Point2 {
        match &self.0 {
            PointFormula::Segment(f) => f.evaluate(store, p),
        }
    }

    pub fn param_indices(&self, store: &impl GeometryStore) -> Vec<usize> {
        match &self.0 {
            PointFormula::Segment(f) => f.param_indices(store),
        }
    }

    pub fn eval_point_jacobian(&self, store: &impl GeometryStore, p: &State) -> PointJacobian {
        match &self.0 {
            PointFormula::Segment(f) => f.eval_point_jacobian(store, p),
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

impl From<segment::VecFormula> for VecExpr {
    fn from(formula: segment::VecFormula) -> Self {
        Self(VecFormula::Segment(formula))
    }
}

impl VecExpr {
    pub fn evaluate(&self, store: &impl GeometryStore, p: &State) -> Vec2 {
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
