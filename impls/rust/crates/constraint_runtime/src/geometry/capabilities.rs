//! Kind-agnostic formula tags and capability-scoped expression factories.
//!
//! - [`CurveExprs`]: on-curve / tangent / normal / length.
//! - [`EndpointExprs`]: start / end (open/join curves only).
//! - [`LineExprs`]: direction (curves with a straight carrier).
//! - [`CircularExprs`]: center / radius (curves with a circular carrier).
//! - [`AnchorExprs`]: origin (geometries with a placement frame, e.g. labels).
//!
//! Every capability is opt-in, including the curve one: a label is a geometry
//! without a curve. Direction, center, radius and origin are **invariants**:
//! independent of a curve location `t` and of the chart. Universal relations
//! (parallel lines, concentric arcs, equal radii) are pointwise constraints on
//! them.
//!
//! Tags name *what* to evaluate; the geometry kind behind an expression's
//! [`GeometryId`] decides *how* (see [`crate::geometry::Geometry`]). Factory
//! traits are opt-in per kind so e.g. closed curves cannot hand out endpoints.
//!
//! A kind declares its capabilities once with `geometry_caps!`, which emits the
//! factory trait impls and the matching runtime [`Caps`] (for code that holds
//! only a [`GeometryId`]; see [`crate::document::Document::caps`]).

use super::formula::Geometry;
use super::shell::GeometryId;
use crate::expr::{LengthExpr, PointExpr, VecExpr};
use crate::geom::CurveParam;

/// Runtime capability set of a geometry kind (a per-kind constant).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Caps {
    /// [`CurveExprs`]: on-curve / tangent / normal / length.
    pub curve: bool,
    /// [`EndpointExprs`]: start / end.
    pub endpoints: bool,
    /// [`LineExprs`]: direction.
    pub line: bool,
    /// [`CircularExprs`]: center / radius.
    pub circular: bool,
    /// [`AnchorExprs`]: origin.
    pub anchor: bool,
}

impl Caps {
    pub const NONE: Self = Self {
        curve: false,
        endpoints: false,
        line: false,
        circular: false,
        anchor: false,
    };
    pub const CURVE: Self = Self::NONE.with_curve();

    pub const fn with_curve(self) -> Self {
        Self {
            curve: true,
            ..self
        }
    }

    pub const fn with_endpoints(self) -> Self {
        Self {
            endpoints: true,
            ..self
        }
    }

    pub const fn with_line(self) -> Self {
        Self { line: true, ..self }
    }

    pub const fn with_circular(self) -> Self {
        Self {
            circular: true,
            ..self
        }
    }

    pub const fn with_anchor(self) -> Self {
        Self {
            anchor: true,
            ..self
        }
    }
}

/// Capability set of a kind. Implemented only via `geometry_caps!`, so it
/// cannot diverge from the kind's factory trait impls.
pub trait GeometryCaps {
    const CAPS: Caps;
}

/// Declare a kind's capabilities: factory trait impls plus [`GeometryCaps`].
///
/// `curve` → [`CurveExprs`], `endpoints` → [`EndpointExprs`],
/// `line` → [`LineExprs`], `circular` → [`CircularExprs`], `anchor` →
/// [`AnchorExprs`].
///
/// ```ignore
/// geometry_caps!(Segment: curve, endpoints, line);
/// // impl CurveExprs for Segment {}
/// // impl EndpointExprs for Segment {}
/// // impl LineExprs for Segment {}
/// // impl GeometryCaps for Segment { const CAPS = CURVE + endpoints + line }
/// ```
macro_rules! geometry_caps {
    ($ty:ty : $($cap:ident),* $(,)?) => {
        $($crate::geometry::capabilities::geometry_caps!(@impl $ty; $cap);)*

        impl $crate::geometry::GeometryCaps for $ty {
            const CAPS: $crate::geometry::Caps = {
                let caps = $crate::geometry::Caps::NONE;
                $(let caps = $crate::geometry::capabilities::geometry_caps!(@with caps; $cap);)*
                caps
            };
        }
    };

    (@impl $ty:ty; curve) => { impl $crate::geometry::CurveExprs for $ty {} };
    (@impl $ty:ty; endpoints) => { impl $crate::geometry::EndpointExprs for $ty {} };
    (@impl $ty:ty; line) => { impl $crate::geometry::LineExprs for $ty {} };
    (@impl $ty:ty; circular) => { impl $crate::geometry::CircularExprs for $ty {} };
    (@impl $ty:ty; anchor) => { impl $crate::geometry::AnchorExprs for $ty {} };

    (@with $caps:ident; curve) => { $caps.with_curve() };
    (@with $caps:ident; endpoints) => { $caps.with_endpoints() };
    (@with $caps:ident; line) => { $caps.with_line() };
    (@with $caps:ident; circular) => { $caps.with_circular() };
    (@with $caps:ident; anchor) => { $caps.with_anchor() };
}
pub(crate) use geometry_caps;

/// Point formulas (curve position and geometric endpoints).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PointFormula {
    OnCurve {
        param: CurveParam,
    },
    /// Kind-efficient start point (may depend on fewer DOFs than `OnCurve(0)`).
    Start,
    /// Kind-efficient end point.
    End,
    /// Center of a circular carrier (invariant; [`CircularExprs`]).
    Center,
    /// Origin of a placement frame (invariant; [`AnchorExprs`]).
    Origin,
}

/// Free-vector formulas for the curve frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VecFormula {
    TangentAt {
        param: CurveParam,
    },
    NormalAt {
        param: CurveParam,
    },
    /// Direction of a straight carrier or baseline, independent of `t`
    /// (invariant; [`LineExprs`]).
    Direction,
}

/// Length formulas.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LengthFormula {
    /// Total curve arc length (`length_expr`).
    Length,
    /// Circle radius magnitude \(|R|\) (circular arc).
    Radius,
}

/// Heading-angle formulas (kinds that expose absolute orientation).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HeadingFormula {
    StartHeading,
}

/// Turn-angle formulas (kinds that expose free angular displacement).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TurnFormula {
    Sweep,
}

/// Curve chart expression factories (position / frame / length along the chart).
pub trait CurveExprs: Geometry {
    fn on_curve_expr(id: GeometryId, param: CurveParam) -> PointExpr {
        PointExpr::new(id, PointFormula::OnCurve { param })
    }

    fn tangent_expr(id: GeometryId, param: CurveParam) -> VecExpr {
        VecExpr::new(id, VecFormula::TangentAt { param })
    }

    fn normal_expr(id: GeometryId, param: CurveParam) -> VecExpr {
        VecExpr::new(id, VecFormula::NormalAt { param })
    }

    fn length_expr(id: GeometryId) -> LengthExpr {
        LengthExpr::new(id, LengthFormula::Length)
    }
}

/// Geometries with a straight carrier or baseline: direction (invariant).
pub trait LineExprs: Geometry {
    fn direction_expr(id: GeometryId) -> VecExpr {
        VecExpr::new(id, VecFormula::Direction)
    }
}

/// Curves with a circular carrier: center and radius (invariants of the circle).
pub trait CircularExprs: Geometry {
    fn center_expr(id: GeometryId) -> PointExpr {
        PointExpr::new(id, PointFormula::Center)
    }

    /// Circle radius \(|R|\) (non-negative).
    fn radius_expr(id: GeometryId) -> LengthExpr {
        LengthExpr::new(id, LengthFormula::Radius)
    }
}

/// Geometric join endpoints (not for intrinsically periodic closed curves).
pub trait EndpointExprs: Geometry {
    fn start_expr(id: GeometryId) -> PointExpr {
        PointExpr::new(id, PointFormula::Start)
    }

    fn end_expr(id: GeometryId) -> PointExpr {
        PointExpr::new(id, PointFormula::End)
    }
}

/// Geometries with a placement frame: its origin (invariant).
pub trait AnchorExprs: Geometry {
    fn origin_expr(id: GeometryId) -> PointExpr {
        PointExpr::new(id, PointFormula::Origin)
    }
}
