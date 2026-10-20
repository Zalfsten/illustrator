//! Opaque point / vector / length / angle expressions (Model B).
//!
//! An expression is a geometry id plus a kind-agnostic formula tag. The kind
//! is resolved from the document once per call; constraints and Interact never
//! match on formula kinds. Construct via capability factories (e.g.
//! [`crate::geometry::EndpointExprs::start_expr`]).

use crate::document::Document;
use crate::geom::Vec2;
use crate::geometry::{
    Formula, GeometryId, HeadingFormula, LengthFormula, PointFormula, TurnFormula, VecFormula,
};
use crate::geometry_kinds::with_kind;
use crate::matrix::DenseMatrix;
use crate::vector::Vector;

/// Upper bound on formula-local parameter count for expression Jacobians.
///
/// Used for stack scratch in [`VecExpr::jet`]. Raise when a kind's vector
/// formula depends on more DOFs than this.
pub const MAX_FORMULA_LOCAL_DOF: usize = 8;

/// Map from Vector parameters to a formula value on one geometry.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Expr<F> {
    pub(crate) geometry: GeometryId,
    pub(crate) formula: F,
}

/// Point expression → [`crate::geom::Point2`].
pub type PointExpr = Expr<PointFormula>;
/// Free-vector expression → [`crate::geom::Vec2`].
pub type VecExpr = Expr<VecFormula>;
/// Length expression → [`crate::geom::Length`].
pub type LengthExpr = Expr<LengthFormula>;
/// Heading-angle expression → [`crate::geom::HeadingAngle`].
pub type HeadingAngleExpr = Expr<HeadingFormula>;
/// Turn-angle expression → [`crate::geom::TurnAngle`].
pub type TurnAngleExpr = Expr<TurnFormula>;

impl<F: Formula> Expr<F> {
    pub(crate) fn new(geometry: GeometryId, formula: F) -> Self {
        Self { geometry, formula }
    }

    pub fn referenced_geometry(&self) -> GeometryId {
        self.geometry
    }

    pub fn evaluate(&self, doc: &Document, p: &Vector) -> F::Value {
        let g = doc.expect_geometry(self.geometry);
        with_kind!(g.kind(), K => self.formula.eval::<K>(g.unpack::<K>(p)))
    }

    /// Global Vector columns this expression depends on (Jacobian column order).
    pub fn param_indices(&self, doc: &Document) -> Vec<usize> {
        let g = doc.expect_geometry(self.geometry);
        let local = with_kind!(g.kind(), K => self.formula.params::<K>());
        g.global_indices(local)
    }

    /// Formula-local DOF count (no allocation).
    pub fn param_count(&self, doc: &Document) -> usize {
        let g = doc.expect_geometry(self.geometry);
        with_kind!(g.kind(), K => self.formula.params::<K>().len())
    }

    /// Write the formula Jacobian into `j` (formula-local columns), scaled by
    /// `scale`. Returns the number of columns written ([`Self::param_count`]).
    pub fn accumulate_jacobian(
        &self,
        doc: &Document,
        p: &Vector,
        j: &mut DenseMatrix<'_>,
        scale: f64,
    ) -> usize {
        let g = doc.expect_geometry(self.geometry);
        with_kind!(g.kind(), K => {
            self.formula.jacobian::<K>(g.unpack::<K>(p), j, scale);
            self.formula.params::<K>().len()
        })
    }
}

/// Free-vector value plus its \(2 \times n\) formula Jacobian (one lookup / unpack).
pub struct VecJet {
    pub value: Vec2,
    cols: usize,
    jac: [f64; 2 * MAX_FORMULA_LOCAL_DOF],
}

impl VecJet {
    /// Formula-local DOF count.
    pub fn cols(&self) -> usize {
        self.cols
    }

    /// Contract with `g` into residual row 0 of `j`, starting at column `col0`:
    /// `j[0, col0 + i] += scale * g · col_i(J_v)`.
    pub fn contract_into(&self, j: &mut DenseMatrix<'_>, col0: usize, g: Vec2, scale: f64) {
        let n = self.cols;
        debug_assert!(j.ncols() >= col0 + n);
        let (row_x, row_y) = self.jac[..2 * n].split_at(n);
        for col in 0..n {
            j[(0, col0 + col)] += scale * (g.x * row_x[col] + g.y * row_y[col]);
        }
    }
}

impl VecExpr {
    /// Evaluate value and Jacobian together (bilinear residuals need both sides'
    /// values before contracting either Jacobian).
    pub fn jet(&self, doc: &Document, p: &Vector) -> VecJet {
        let g = doc.expect_geometry(self.geometry);
        with_kind!(g.kind(), K => {
            let chart = g.unpack::<K>(p);
            let n = self.formula.params::<K>().len();
            debug_assert!(
                n <= MAX_FORMULA_LOCAL_DOF,
                "vector formula local DOF count {n} exceeds MAX_FORMULA_LOCAL_DOF"
            );
            let mut jac = [0.0; 2 * MAX_FORMULA_LOCAL_DOF];
            self.formula
                .jacobian::<K>(chart, &mut DenseMatrix::packed(&mut jac[..2 * n], 2, n), 1.0);
            VecJet {
                value: self.formula.eval::<K>(chart),
                cols: n,
                jac,
            }
        })
    }
}
