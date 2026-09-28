//! Parameterized geometries: local params are references into the global state vector.
//!
//! Affine distinction: [`Vec2`] is a free vector (displacement / direction);
//! [`Point2`] is a location. Prefer `b - a` over constructing components by hand.
//!
//! Curve addressing: [`CurveParam::Normalized`] is the canonical chart \(t \in [0,1]\)
//! (native formula domain affinely mapped). [`CurveParam::Uniform`] is constant-speed
//! / arc-length fraction on \([0,1]\). For line segments the two coincide.

use crate::connectivity::Connectivity;
use std::ops::{Add, Mul, Neg, Sub};

pub const SEGMENT_STRIDE: usize = 4;

/// Free vector in the plane (displacement, direction, difference of points).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Vec2 {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    pub fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y
    }

    /// Signed 2D cross product magnitude \(u_x v_y - u_y v_x\).
    pub fn cross(self, other: Self) -> f64 {
        self.x * other.y - self.y * other.x
    }

    pub fn length2(self) -> f64 {
        self.dot(self)
    }

    pub fn length(self) -> f64 {
        self.length2().sqrt()
    }

    /// Leftward planar rotate \((-y, x)\) (CCW 90°).
    pub fn left_normal(self) -> Self {
        Self {
            x: -self.y,
            y: self.x,
        }
    }
}

impl Neg for Vec2 {
    type Output = Self;

    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl Add for Vec2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for Vec2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Mul<f64> for Vec2 {
    type Output = Self;

    fn mul(self, s: f64) -> Self {
        Self {
            x: self.x * s,
            y: self.y * s,
        }
    }
}

/// Affine point (location). Representable as a position vector relative to an origin,
/// but not interchangeable with [`Vec2`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point2 {
    pub x: f64,
    pub y: f64,
}

impl Point2 {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn dist2(self, other: Self) -> f64 {
        (self - other).length2()
    }
}

impl Sub for Point2 {
    type Output = Vec2;

    fn sub(self, rhs: Self) -> Vec2 {
        Vec2::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Add<Vec2> for Point2 {
    type Output = Self;

    fn add(self, rhs: Vec2) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub<Vec2> for Point2 {
    type Output = Self;

    fn sub(self, rhs: Vec2) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

/// Parameter addressing a location on a curve.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CurveParam {
    /// Canonical chart \(t \in [0,1]\) (normalized native domain).
    Normalized(f64),
    /// Constant-speed / arc-length fraction \(u \in [0,1]\).
    Uniform(f64),
}

impl CurveParam {
    /// Chart \(t\) when Normalized ≡ Uniform (line segments today).
    pub fn chart_t(self) -> f64 {
        match self {
            CurveParam::Normalized(t) | CurveParam::Uniform(t) => t,
        }
    }
}

/// Curve evaluation at normalized chart parameter \(t \in [0,1]\).
pub trait Curve {
    fn position_at(&self, t: f64, p: &[f64]) -> Point2;
    fn tangent_at(&self, t: f64, p: &[f64]) -> Vec2;
    fn normal_at(&self, t: f64, p: &[f64]) -> Vec2 {
        self.tangent_at(t, p).left_normal()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GeometryId(pub usize);

/// Alias kept for the segment-only UI / tools layer.
pub type SegmentId = GeometryId;

/// Presentation role: result figure vs construction helper.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum GeometryRole {
    #[default]
    Result,
    Construction,
}

/// Document-level sum of geometry kinds (Model E storage).
#[derive(Clone, Debug, PartialEq)]
pub enum Geometry {
    Segment(Segment),
}

impl Geometry {
    pub fn id(&self) -> GeometryId {
        match self {
            Geometry::Segment(s) => s.id,
        }
    }

    pub fn role(&self) -> GeometryRole {
        match self {
            Geometry::Segment(s) => s.role,
        }
    }

    pub fn parameter_count(&self) -> usize {
        match self {
            Geometry::Segment(_) => SEGMENT_STRIDE,
        }
    }

    pub fn connectivity(&self) -> Connectivity {
        match self {
            Geometry::Segment(s) => s.connectivity(),
        }
    }

    /// After a contiguous block was removed from State, shift this geometry's
    /// local→global indices that lay after the hole.
    pub fn reindex_after_removal(&mut self, removed_start: usize, removed_len: usize) {
        match self {
            Geometry::Segment(s) => {
                if s.param_offset > removed_start {
                    s.param_offset -= removed_len;
                }
            }
        }
    }

    /// Curve capability when this geometry is a curve (segment, later circle, …).
    pub fn curve(&self) -> Option<&dyn Curve> {
        match self {
            Geometry::Segment(s) => Some(s),
        }
    }
}

/// O(1) geometry lookup during evaluate/solve.
/// Implemented by [`crate::document::DocumentStructure`] via an id→slot index.
pub trait GeometryStore {
    fn geometry(&self, id: GeometryId) -> Option<&Geometry>;

    fn expect_geometry(&self, id: GeometryId) -> &Geometry {
        self.geometry(id)
            .unwrap_or_else(|| panic!("geometry id {id:?} not in document"))
    }
}

/// Line segment: endpoint chart, parameters live at `p[param_offset..param_offset+4]`.
#[derive(Clone, Debug, PartialEq)]
pub struct Segment {
    pub id: GeometryId,
    pub param_offset: usize,
    pub role: GeometryRole,
}

impl Segment {
    pub fn connectivity(&self) -> Connectivity {
        let o = self.param_offset;
        Connectivity::new(vec![o, o + 1, o + 2, o + 3])
    }

    pub fn start(&self, p: &[f64]) -> Point2 {
        Point2::new(p[self.param_offset], p[self.param_offset + 1])
    }

    pub fn end(&self, p: &[f64]) -> Point2 {
        Point2::new(p[self.param_offset + 2], p[self.param_offset + 3])
    }

    /// Free direction vector from start to end (`end - start`).
    pub fn direction(&self, p: &[f64]) -> Vec2 {
        self.end(p) - self.start(p)
    }

    /// Point formula for the start endpoint of the segment identified by `id`.
    pub fn start_expr(id: GeometryId) -> PointExpr {
        PointExpr(PointFormula::Endpoint {
            geometry: id,
            end: EndpointKind::Start,
        })
    }

    /// Point formula for the end endpoint of the segment identified by `id`.
    pub fn end_expr(id: GeometryId) -> PointExpr {
        PointExpr(PointFormula::Endpoint {
            geometry: id,
            end: EndpointKind::End,
        })
    }

    /// Point formula for a chart sample on the segment identified by `id`.
    pub fn on_curve_expr(id: GeometryId, param: CurveParam) -> PointExpr {
        PointExpr(PointFormula::OnCurve {
            geometry: id,
            param,
        })
    }

    pub fn start_point(&self) -> PointExpr {
        Self::start_expr(self.id)
    }

    pub fn end_point(&self) -> PointExpr {
        Self::end_expr(self.id)
    }

    pub fn point_at(&self, param: CurveParam) -> PointExpr {
        Self::on_curve_expr(self.id, param)
    }

    pub fn endpoints(&self) -> (PointExpr, PointExpr) {
        (self.start_point(), self.end_point())
    }

    /// Free-vector formula: segment direction (`end - start`).
    pub fn direction_expr(id: GeometryId) -> VecExpr {
        VecExpr(VecFormula::SegmentDirection { geometry: id })
    }

    /// Free-vector formula: curve tangent at `param`.
    pub fn tangent_expr(id: GeometryId, param: CurveParam) -> VecExpr {
        VecExpr(VecFormula::TangentAt {
            geometry: id,
            param,
        })
    }

    /// Free-vector formula: curve normal at `param`.
    pub fn normal_expr(id: GeometryId, param: CurveParam) -> VecExpr {
        VecExpr(VecFormula::NormalAt {
            geometry: id,
            param,
        })
    }

    pub fn direction_vec(&self) -> VecExpr {
        Self::direction_expr(self.id)
    }

    pub fn tangent_vec(&self, param: CurveParam) -> VecExpr {
        Self::tangent_expr(self.id, param)
    }

    pub fn normal_vec(&self, param: CurveParam) -> VecExpr {
        Self::normal_expr(self.id, param)
    }

    pub fn all_param_indices(&self) -> [usize; 4] {
        let o = self.param_offset;
        [o, o + 1, o + 2, o + 3]
    }
}

impl Curve for Segment {
    fn position_at(&self, t: f64, p: &[f64]) -> Point2 {
        let a = self.start(p);
        let b = self.end(p);
        Point2::new(
            (1.0 - t) * a.x + t * b.x,
            (1.0 - t) * a.y + t * b.y,
        )
    }

    fn tangent_at(&self, _t: f64, p: &[f64]) -> Vec2 {
        // γ'(t) = B - A (constant); length = segment length
        self.direction(p)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EndpointKind {
    Start,
    End,
}

/// Private formula catalog for [`PointExpr`] (Model B: opaque handle, closed enum).
#[derive(Clone, Copy, Debug, PartialEq)]
enum PointFormula {
    Endpoint {
        geometry: GeometryId,
        end: EndpointKind,
    },
    OnCurve {
        geometry: GeometryId,
        param: CurveParam,
    },
}

/// Opaque point expression: map from State parameters to a [`Point2`].
///
/// Construct via geometry factories (e.g. [`Segment::start_point`]); constraints
/// and Interact only call [`Self::evaluate`] / Jacobian helpers — they do not
/// match on formula kinds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointExpr(PointFormula);

impl PointExpr {
    pub fn evaluate(&self, store: &impl GeometryStore, p: &[f64]) -> Point2 {
        match self.0 {
            PointFormula::Endpoint { geometry, end } => {
                let Geometry::Segment(s) = store.expect_geometry(geometry);
                match end {
                    EndpointKind::Start => s.start(p),
                    EndpointKind::End => s.end(p),
                }
            }
            PointFormula::OnCurve { geometry, param } => store
                .expect_geometry(geometry)
                .curve()
                .expect("OnCurve requires a curve geometry")
                .position_at(param.chart_t(), p),
        }
    }

    pub fn param_indices(&self, store: &impl GeometryStore) -> Vec<usize> {
        match self.0 {
            PointFormula::Endpoint { geometry, end } => {
                let Geometry::Segment(s) = store.expect_geometry(geometry);
                let o = match end {
                    EndpointKind::Start => s.param_offset,
                    EndpointKind::End => s.param_offset + 2,
                };
                vec![o, o + 1]
            }
            PointFormula::OnCurve { geometry, .. } => {
                store.expect_geometry(geometry).connectivity().global_indices
            }
        }
    }

    pub fn eval_point_jacobian(
        &self,
        store: &impl GeometryStore,
        _p: &[f64],
        dx: &mut [f64],
        dy: &mut [f64],
    ) {
        match self.0 {
            PointFormula::Endpoint { .. } => {
                assert_eq!(dx.len(), 2);
                assert_eq!(dy.len(), 2);
                dx.fill(0.0);
                dy.fill(0.0);
                dx[0] = 1.0;
                dy[1] = 1.0;
            }
            PointFormula::OnCurve { geometry, param } => {
                let n = store.expect_geometry(geometry).connectivity().len();
                assert_eq!(dx.len(), n);
                assert_eq!(dy.len(), n);
                dx.fill(0.0);
                dy.fill(0.0);
                let Geometry::Segment(_) = store.expect_geometry(geometry);
                let t = param.chart_t();
                dx[0] = 1.0 - t;
                dy[1] = 1.0 - t;
                dx[2] = t;
                dy[3] = t;
            }
        }
    }

    pub fn referenced_geometry(&self) -> GeometryId {
        match self.0 {
            PointFormula::Endpoint { geometry, .. }
            | PointFormula::OnCurve { geometry, .. } => geometry,
        }
    }
}

/// Private formula catalog for [`VecExpr`] (same opacity principle as [`PointExpr`]).
#[derive(Clone, Copy, Debug, PartialEq)]
enum VecFormula {
    SegmentDirection { geometry: GeometryId },
    TangentAt {
        geometry: GeometryId,
        param: CurveParam,
    },
    NormalAt {
        geometry: GeometryId,
        param: CurveParam,
    },
}

/// Opaque free-vector expression: map from State parameters to a [`Vec2`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VecExpr(VecFormula);

impl VecExpr {
    pub fn evaluate(&self, store: &impl GeometryStore, p: &[f64]) -> Vec2 {
        match self.0 {
            VecFormula::SegmentDirection { geometry } => {
                let Geometry::Segment(s) = store.expect_geometry(geometry);
                s.direction(p)
            }
            VecFormula::TangentAt { geometry, param } => store
                .expect_geometry(geometry)
                .curve()
                .expect("TangentAt requires a curve geometry")
                .tangent_at(param.chart_t(), p),
            VecFormula::NormalAt { geometry, param } => store
                .expect_geometry(geometry)
                .curve()
                .expect("NormalAt requires a curve geometry")
                .normal_at(param.chart_t(), p),
        }
    }

    pub fn referenced_geometry(&self) -> GeometryId {
        match self.0 {
            VecFormula::SegmentDirection { geometry }
            | VecFormula::TangentAt { geometry, .. }
            | VecFormula::NormalAt { geometry, .. } => geometry,
        }
    }
}

#[cfg(test)]
#[path = "tests_geom.rs"]
mod tests_geom;
