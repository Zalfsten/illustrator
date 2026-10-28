//! Kind formula contract and tag → kind dispatch.
//!
//! [`Geometry`] is what a kind implements: one value / Jacobian / connectivity
//! triple per formula family. [`Formula`] connects each tag family in
//! [`super::capabilities`] to its triple, so [`crate::expr::Expr`] needs a
//! single generic evaluate / Jacobian path and one kind match
//! (`with_kind!` in [`crate::geometry_kinds`]).
//!
//! **Adding a kind:** implement [`Chart`] (via `chart_atoms!`), declare its
//! capabilities with `geometry_caps!`, and implement [`Geometry`] (including
//! `NAME_PREFIX`; connectivity defaults to the whole chart; override only when
//! sparse, e.g. `point_params` for endpoints), then add it to
//! [`GeometryKind`] / `with_kind!`.

use std::ops::Range;

use super::capabilities::{
    GeometryCaps, HeadingFormula, LengthFormula, PointFormula, TurnFormula, VecFormula,
};
use super::chart::Chart;
use crate::geom::{HeadingAngle, Length, Point2, TurnAngle, Vec2};
use crate::geometry_kinds::GeometryKind;
use crate::matrix::DenseMatrix;

/// Formula-local columns covering the whole chart block.
pub fn all_params<K: Chart>() -> Range<usize> {
    0..K::PARAMETER_COUNT
}

#[cold]
#[track_caller]
fn unsupported(kind: GeometryKind, family: &str) -> ! {
    unreachable!("{kind:?} has no {family} formula")
}

/// Geometry kind: chart plus formula catalog.
///
/// Jacobians write formula-local columns `0..params(f).len()`, which map to
/// chart-local columns `params(f)`. Families a kind does not expose keep the
/// panicking defaults; their factories are kind-specific, so callers cannot
/// build such expressions. The capability set ([`GeometryCaps`]) comes from
/// `geometry_caps!`.
pub trait Geometry: Chart + GeometryCaps + Sized + 'static {
    const KIND: GeometryKind;

    /// Prefix of automatic object names (`seg` → `seg1`, `seg2`, …).
    const NAME_PREFIX: &'static str;

    /// Canonical chart values describing the same curve (identity by default).
    ///
    /// Applied to committed State and to initial values on add; solver iterates
    /// may hold non-canonical values (e.g. a negative arc radius).
    fn canonical(init: Self::Init) -> Self::Init {
        init
    }

    fn point(f: PointFormula, c: Self::Init) -> Point2;
    fn point_jacobian(f: PointFormula, c: Self::Init, j: &mut DenseMatrix<'_>, scale: f64);
    fn point_params(f: PointFormula) -> Range<usize> {
        let _ = f;
        all_params::<Self>()
    }

    fn vec(f: VecFormula, c: Self::Init) -> Vec2;
    fn vec_jacobian(f: VecFormula, c: Self::Init, j: &mut DenseMatrix<'_>, scale: f64);
    fn vec_params(f: VecFormula) -> Range<usize> {
        let _ = f;
        all_params::<Self>()
    }

    fn length(f: LengthFormula, c: Self::Init) -> Length;
    fn length_jacobian(f: LengthFormula, c: Self::Init, j: &mut DenseMatrix<'_>, scale: f64);
    fn length_params(f: LengthFormula) -> Range<usize> {
        let _ = f;
        all_params::<Self>()
    }

    fn heading(f: HeadingFormula, c: Self::Init) -> HeadingAngle {
        let _ = (f, c);
        unsupported(Self::KIND, "heading")
    }
    fn heading_jacobian(f: HeadingFormula, c: Self::Init, j: &mut DenseMatrix<'_>, scale: f64) {
        let _ = (f, c, j, scale);
        unsupported(Self::KIND, "heading")
    }
    fn heading_params(f: HeadingFormula) -> Range<usize> {
        let _ = f;
        unsupported(Self::KIND, "heading")
    }

    fn turn(f: TurnFormula, c: Self::Init) -> TurnAngle {
        let _ = (f, c);
        unsupported(Self::KIND, "turn")
    }
    fn turn_jacobian(f: TurnFormula, c: Self::Init, j: &mut DenseMatrix<'_>, scale: f64) {
        let _ = (f, c, j, scale);
        unsupported(Self::KIND, "turn")
    }
    fn turn_params(f: TurnFormula) -> Range<usize> {
        let _ = f;
        unsupported(Self::KIND, "turn")
    }
}

/// A formula tag family: value type plus the [`Geometry`] triple it dispatches to.
pub trait Formula: Copy + std::fmt::Debug + PartialEq {
    type Value;
    fn eval<K: Geometry>(self, c: K::Init) -> Self::Value;
    fn jacobian<K: Geometry>(self, c: K::Init, j: &mut DenseMatrix<'_>, scale: f64);
    fn params<K: Geometry>(self) -> Range<usize>;
}

macro_rules! formula_family {
    ($($Tag:ty => $Value:ty: $eval:ident, $jac:ident, $params:ident;)+) => {$(
        impl Formula for $Tag {
            type Value = $Value;

            #[inline]
            fn eval<K: Geometry>(self, c: K::Init) -> $Value {
                K::$eval(self, c)
            }

            #[inline]
            fn jacobian<K: Geometry>(self, c: K::Init, j: &mut DenseMatrix<'_>, scale: f64) {
                K::$jac(self, c, j, scale)
            }

            #[inline]
            fn params<K: Geometry>(self) -> Range<usize> {
                K::$params(self)
            }
        }
    )+};
}

formula_family! {
    PointFormula => Point2: point, point_jacobian, point_params;
    VecFormula => Vec2: vec, vec_jacobian, vec_params;
    LengthFormula => Length: length, length_jacobian, length_params;
    HeadingFormula => HeadingAngle: heading, heading_jacobian, heading_params;
    TurnFormula => TurnAngle: turn, turn_jacobian, turn_params;
}
