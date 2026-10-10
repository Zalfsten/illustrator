//! Opaque point / vector / length expressions (Model B).
//!
//! Constraints and Interact use these handles without matching on formula kinds.
//! Geometry-kind formula keying and evaluate / Jacobian dispatch live in
//! [`crate::geometry::expr`].

use crate::geom::Vec2;
use crate::geometry::expr::{
    HeadingAngleFormulaKeyed, LengthFormulaKeyed, PointFormulaKeyed, TurnAngleFormulaKeyed,
    VecFormulaKeyed,
};
use crate::geometry::GeometryId;
use crate::matrix::{DenseMatrix, MatrixAccess};
use crate::store::GeometryStore;
use crate::vector::Vector;

/// Upper bound on formula-local parameter count for expression Jacobians.
///
/// Used for stack scratch in [`VecExpr::accumulate_contracted`]. Raise when a
/// kind's vector formula depends on more DOFs than this.
pub const MAX_FORMULA_LOCAL_DOF: usize = 8;

/// Opaque point expression: map from Vector parameters to a [`crate::geom::Point2`].
///
/// Construct via capability factories (e.g. [`crate::geometry::EndpointExprs::start_expr`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointExpr {
    pub(crate) geometry: GeometryId,
    pub(crate) formula: PointFormulaKeyed,
}

impl PointExpr {
    pub(crate) fn new(geometry: GeometryId, formula: PointFormulaKeyed) -> Self {
        Self { geometry, formula }
    }

    pub fn referenced_geometry(&self) -> GeometryId {
        self.geometry
    }
}

/// Opaque free-vector expression: map from Vector parameters to a [`crate::geom::Vec2`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VecExpr {
    pub(crate) geometry: GeometryId,
    pub(crate) formula: VecFormulaKeyed,
}

impl VecExpr {
    pub(crate) fn new(geometry: GeometryId, formula: VecFormulaKeyed) -> Self {
        Self { geometry, formula }
    }

    pub fn referenced_geometry(&self) -> GeometryId {
        self.geometry
    }

    /// Contract the free-vector Jacobian with `g` into residual row 0 of `j`:
    /// `j[0,i] += scale * g · col_i(J_v)`.
    ///
    /// Derived from [`Self::accumulate_jacobian`] — kinds only implement the
    /// \(2 \times n\) vector Jacobian (dispatched in [`crate::geometry::expr`]).
    pub fn accumulate_contracted<S: GeometryStore + ?Sized, J: MatrixAccess>(
        &self,
        store: &S,
        p: &Vector,
        j: &mut J,
        g: Vec2,
        scale: f64,
    ) {
        let n = self.param_count();
        debug_assert!(j.ncols() >= n);
        debug_assert!(
            n <= MAX_FORMULA_LOCAL_DOF,
            "vector formula local DOF count {n} exceeds MAX_FORMULA_LOCAL_DOF"
        );
        let mut buf = [0.0; 2 * MAX_FORMULA_LOCAL_DOF];
        let mut jv = DenseMatrix::packed(&mut buf[..2 * n], 2, n);
        self.accumulate_jacobian(store, p, &mut jv, 1.0);
        for col in 0..n {
            j[(0, col)] += scale * (g.x * jv[(0, col)] + g.y * jv[(1, col)]);
        }
    }
}

/// Opaque length expression: map from Vector parameters to a [`crate::geom::Length`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LengthExpr {
    pub(crate) geometry: GeometryId,
    pub(crate) formula: LengthFormulaKeyed,
}

impl LengthExpr {
    pub(crate) fn new(geometry: GeometryId, formula: LengthFormulaKeyed) -> Self {
        Self { geometry, formula }
    }

    pub fn referenced_geometry(&self) -> GeometryId {
        self.geometry
    }
}

/// Opaque heading-angle expression: map from Vector parameters to a [`crate::geom::HeadingAngle`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HeadingAngleExpr {
    pub(crate) geometry: GeometryId,
    pub(crate) formula: HeadingAngleFormulaKeyed,
}

impl HeadingAngleExpr {
    #[allow(dead_code)] // constructed when a kind registers heading formulas
    pub(crate) fn new(geometry: GeometryId, formula: HeadingAngleFormulaKeyed) -> Self {
        Self { geometry, formula }
    }

    pub fn referenced_geometry(&self) -> GeometryId {
        self.geometry
    }
}

/// Opaque turn-angle expression: map from Vector parameters to a [`crate::geom::TurnAngle`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TurnAngleExpr {
    pub(crate) geometry: GeometryId,
    pub(crate) formula: TurnAngleFormulaKeyed,
}

impl TurnAngleExpr {
    #[allow(dead_code)] // constructed when a kind registers turn formulas
    pub(crate) fn new(geometry: GeometryId, formula: TurnAngleFormulaKeyed) -> Self {
        Self { geometry, formula }
    }

    pub fn referenced_geometry(&self) -> GeometryId {
        self.geometry
    }
}
