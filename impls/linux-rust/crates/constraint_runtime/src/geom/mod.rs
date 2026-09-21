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

    pub fn endpoints(&self) -> (PointExpr, PointExpr) {
        (
            PointExpr::Endpoint {
                geometry: self.id,
                end: EndpointKind::Start,
            },
            PointExpr::Endpoint {
                geometry: self.id,
                end: EndpointKind::End,
            },
        )
    }

    pub fn endpoint_param_indices(&self, end: EndpointKind) -> [usize; 2] {
        match end {
            EndpointKind::Start => [self.param_offset, self.param_offset + 1],
            EndpointKind::End => [self.param_offset + 2, self.param_offset + 3],
        }
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PointExpr {
    /// Endpoint of a geometry that has the endpoints capability (line segment today).
    Endpoint {
        geometry: GeometryId,
        end: EndpointKind,
    },
    OnCurve {
        geometry: GeometryId,
        param: CurveParam,
    },
}

impl PointExpr {
    pub fn evaluate(&self, store: &impl GeometryStore, p: &[f64]) -> Point2 {
        match *self {
            PointExpr::Endpoint { geometry, end } => {
                let Geometry::Segment(s) = store.expect_geometry(geometry);
                match end {
                    EndpointKind::Start => s.start(p),
                    EndpointKind::End => s.end(p),
                }
            }
            PointExpr::OnCurve { geometry, param } => store
                .expect_geometry(geometry)
                .curve()
                .expect("OnCurve requires a curve geometry")
                .position_at(param.chart_t(), p),
        }
    }

    /// Global parameter indices this expression depends on.
    pub fn param_indices(&self, store: &impl GeometryStore) -> Vec<usize> {
        match *self {
            PointExpr::Endpoint { geometry, end } => {
                let Geometry::Segment(s) = store.expect_geometry(geometry);
                let ix = s.endpoint_param_indices(end);
                vec![ix[0], ix[1]]
            }
            PointExpr::OnCurve { geometry, .. } => {
                store.expect_geometry(geometry).connectivity().global_indices
            }
        }
    }

    /// Analytical \(\partial(x,y)/\partial p_i\) for each index in [`Self::param_indices`],
    /// written as parallel `dx` / `dy` slices (same length as connectivity).
    pub fn eval_point_jacobian(
        &self,
        store: &impl GeometryStore,
        _p: &[f64],
        dx: &mut [f64],
        dy: &mut [f64],
    ) {
        let n = self.param_indices(store).len();
        assert_eq!(dx.len(), n);
        assert_eq!(dy.len(), n);
        dx.fill(0.0);
        dy.fill(0.0);
        match *self {
            PointExpr::Endpoint { .. } => {
                dx[0] = 1.0;
                dy[1] = 1.0;
            }
            PointExpr::OnCurve { geometry, param } => {
                // Segment chart for now; other curve kinds supply their own J later.
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
        match *self {
            PointExpr::Endpoint { geometry, .. } | PointExpr::OnCurve { geometry, .. } => geometry,
        }
    }
}

/// Free-vector expression over geometry parameters.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VecExpr {
    /// Direction of a line segment (`end - start`).
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

impl VecExpr {
    pub fn evaluate(&self, store: &impl GeometryStore, p: &[f64]) -> Vec2 {
        match *self {
            VecExpr::SegmentDirection { geometry } => {
                let Geometry::Segment(s) = store.expect_geometry(geometry);
                s.direction(p)
            }
            VecExpr::TangentAt { geometry, param } => store
                .expect_geometry(geometry)
                .curve()
                .expect("TangentAt requires a curve geometry")
                .tangent_at(param.chart_t(), p),
            VecExpr::NormalAt { geometry, param } => store
                .expect_geometry(geometry)
                .curve()
                .expect("NormalAt requires a curve geometry")
                .normal_at(param.chart_t(), p),
        }
    }
}

#[cfg(test)]
mod tests;
