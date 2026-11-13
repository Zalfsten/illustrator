//! Kind formula contract and tag → kind dispatch.
//!
//! [`Geometry`] is what a kind implements: one value / Jacobian / connectivity
//! triple per formula family. Values and Jacobians read the geometry's chart
//! values and its typed kind attributes ([`Geometry::Attrs`]); connectivity
//! depends on the formula alone. [`Formula`] connects each tag family in
//! [`super::capabilities`] to its triple, so [`crate::expr::Expr`] needs a
//! single generic evaluate / Jacobian path and one kind match
//! (`with_kind!` in [`crate::geometry_kinds`]).
//!
//! **Adding a kind:** implement [`Chart`] (via `chart_atoms!`), declare its
//! capabilities with `geometry_caps!`, and implement [`Geometry`] (including
//! `NAME_PREFIX`, `Attrs` and `attrs_of`, `()` for kinds without attributes;
//! connectivity defaults to the whole chart; override only when
//! sparse, e.g. `point_params` for endpoints), then add it to
//! [`GeometryKind`] / `with_kind!`.

use std::ops::Range;

use super::capabilities::{
    GeometryCaps, HeadingFormula, LengthFormula, PointFormula, TurnFormula, VecFormula,
};
use super::chart::Chart;
use super::shape::ShapePiece;
use crate::geom::{HeadingAngle, Length, Point2, TurnAngle, Vec2};
use crate::geometry_kinds::{GeometryKind, KindAttrs};
use crate::matrix::DenseMatrix;

/// Arc length and curvature upper bound of a curve piece
/// ([`Geometry::piece_bound`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PieceBound {
    /// Exact arc length of the piece.
    pub length: f64,
    /// Upper bound of the unsigned curvature on the whole piece (`+∞` allowed).
    pub curvature: f64,
}

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

    /// Typed non-parameter attributes of this kind (`()` without any).
    ///
    /// Every chart-dependent formula receives them next to the chart values.
    /// They are not State: they add no Jacobian columns, and connectivity
    /// (`*_params`) does not depend on them.
    type Attrs: std::fmt::Debug + PartialEq + 'static;

    /// This kind's attributes inside the stored [`KindAttrs`]; panics on
    /// another kind's variant (an instance's attributes always match its kind).
    fn attrs_of(a: &KindAttrs) -> &Self::Attrs;

    /// Non-parameter attributes a new geometry of this kind starts with.
    fn default_attrs() -> KindAttrs {
        KindAttrs::None
    }

    /// Canonical chart values describing the same curve (identity by default).
    ///
    /// Applied to committed State and to initial values on add; solver iterates
    /// may hold non-canonical values (e.g. a negative arc radius).
    fn canonical(init: Self::Init, a: &Self::Attrs) -> Self::Init {
        let _ = a;
        init
    }

    /// Exact shape (curve capability only), at canonical chart values.
    fn shape(c: Self::Init, a: &Self::Attrs) -> Vec<ShapePiece> {
        let _ = (c, a);
        unsupported(Self::KIND, "shape")
    }

    /// Whether speed \(|\gamma'(t)|\) and [`curvature`](Self::curvature) are the
    /// same at every normalized `t` (lines, circular arcs; curve capability
    /// only). The display Mesh then places equal edges in closed form instead
    /// of sampling speed and curvature along the curve.
    const UNIFORM_CURVE: bool = false;

    /// Unsigned curvature \(|\gamma'\times\gamma''|/|\gamma'|^3\) at normalized
    /// `t ∈ [0, 1]` (curve capability only); 0 where the curve is straight.
    ///
    /// Evaluation-only (display tessellation): no Jacobian, not a constraint
    /// operand. Accepts non-canonical solver iterates.
    fn curvature(t: f64, c: Self::Init, a: &Self::Attrs) -> f64 {
        let _ = (t, c, a);
        unsupported(Self::KIND, "curvature")
    }

    /// Exact arc length of the normalized piece `[t0, t1]` and an upper bound
    /// of its unsigned curvature on the whole piece (curve capability only).
    ///
    /// The length is exact because every kind's normalized chart has constant
    /// speed; a kind without one must still return the exact piece length.
    /// The bound may exceed the curvature but never fall below it, so
    /// consumers (display tessellation) stay sound. Evaluation-only, like
    /// [`curvature`](Self::curvature).
    fn piece_bound(t0: f64, t1: f64, c: Self::Init, a: &Self::Attrs) -> PieceBound {
        let _ = (t0, t1, c, a);
        unsupported(Self::KIND, "piece bound")
    }

    fn point(f: PointFormula, c: Self::Init, a: &Self::Attrs) -> Point2 {
        let _ = (f, c, a);
        unsupported(Self::KIND, "point")
    }
    fn point_jacobian(
        f: PointFormula,
        c: Self::Init,
        a: &Self::Attrs,
        j: &mut DenseMatrix<'_>,
        scale: f64,
    ) {
        let _ = (f, c, a, j, scale);
        unsupported(Self::KIND, "point")
    }
    fn point_params(f: PointFormula) -> Range<usize> {
        let _ = f;
        all_params::<Self>()
    }

    fn vec(f: VecFormula, c: Self::Init, a: &Self::Attrs) -> Vec2 {
        let _ = (f, c, a);
        unsupported(Self::KIND, "vector")
    }
    fn vec_jacobian(
        f: VecFormula,
        c: Self::Init,
        a: &Self::Attrs,
        j: &mut DenseMatrix<'_>,
        scale: f64,
    ) {
        let _ = (f, c, a, j, scale);
        unsupported(Self::KIND, "vector")
    }
    fn vec_params(f: VecFormula) -> Range<usize> {
        let _ = f;
        all_params::<Self>()
    }

    fn length(f: LengthFormula, c: Self::Init, a: &Self::Attrs) -> Length {
        let _ = (f, c, a);
        unsupported(Self::KIND, "length")
    }
    fn length_jacobian(
        f: LengthFormula,
        c: Self::Init,
        a: &Self::Attrs,
        j: &mut DenseMatrix<'_>,
        scale: f64,
    ) {
        let _ = (f, c, a, j, scale);
        unsupported(Self::KIND, "length")
    }
    fn length_params(f: LengthFormula) -> Range<usize> {
        let _ = f;
        all_params::<Self>()
    }

    fn heading(f: HeadingFormula, c: Self::Init, a: &Self::Attrs) -> HeadingAngle {
        let _ = (f, c, a);
        unsupported(Self::KIND, "heading")
    }
    fn heading_jacobian(
        f: HeadingFormula,
        c: Self::Init,
        a: &Self::Attrs,
        j: &mut DenseMatrix<'_>,
        scale: f64,
    ) {
        let _ = (f, c, a, j, scale);
        unsupported(Self::KIND, "heading")
    }
    fn heading_params(f: HeadingFormula) -> Range<usize> {
        let _ = f;
        unsupported(Self::KIND, "heading")
    }

    fn turn(f: TurnFormula, c: Self::Init, a: &Self::Attrs) -> TurnAngle {
        let _ = (f, c, a);
        unsupported(Self::KIND, "turn")
    }
    fn turn_jacobian(
        f: TurnFormula,
        c: Self::Init,
        a: &Self::Attrs,
        j: &mut DenseMatrix<'_>,
        scale: f64,
    ) {
        let _ = (f, c, a, j, scale);
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
    fn eval<K: Geometry>(self, c: K::Init, a: &K::Attrs) -> Self::Value;
    fn jacobian<K: Geometry>(self, c: K::Init, a: &K::Attrs, j: &mut DenseMatrix<'_>, scale: f64);
    fn params<K: Geometry>(self) -> Range<usize>;
}

macro_rules! formula_family {
    ($($Tag:ty => $Value:ty: $eval:ident, $jac:ident, $params:ident;)+) => {$(
        impl Formula for $Tag {
            type Value = $Value;

            #[inline]
            fn eval<K: Geometry>(self, c: K::Init, a: &K::Attrs) -> $Value {
                K::$eval(self, c, a)
            }

            #[inline]
            fn jacobian<K: Geometry>(
                self,
                c: K::Init,
                a: &K::Attrs,
                j: &mut DenseMatrix<'_>,
                scale: f64,
            ) {
                K::$jac(self, c, a, j, scale)
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
