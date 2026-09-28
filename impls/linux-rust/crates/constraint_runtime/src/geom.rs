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
/// Curve-shaped document geometries expose this via [`Geometry::position_at`] /
/// [`Geometry::tangent_at`] / [`Geometry::normal_at`].
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

/// Document geometry: shared identity, role, State block offset, plus kind-specific body.
#[derive(Clone, Debug, PartialEq)]
pub struct Geometry {
    pub id: GeometryId,
    pub role: GeometryRole,
    /// Start index of this geometry's contiguous parameter block in State.
    pub param_offset: usize,
    pub body: GeometryBody,
}

/// Kind-specific payload (charts). Shared fields live on [`Geometry`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GeometryBody {
    Segment,
}

impl GeometryBody {
    pub fn parameter_count(self) -> usize {
        match self {
            GeometryBody::Segment => Segment::STRIDE,
        }
    }
}

impl Geometry {
    pub fn id(&self) -> GeometryId {
        self.id
    }

    pub fn role(&self) -> GeometryRole {
        self.role
    }

    pub fn set_role(&mut self, role: GeometryRole) {
        self.role = role;
    }

    pub fn parameter_count(&self) -> usize {
        self.body.parameter_count()
    }

    pub fn connectivity(&self) -> Connectivity {
        let o = self.param_offset;
        let n = self.parameter_count();
        Connectivity::new((o..o + n).collect())
    }

    /// After a contiguous block was removed from State, shift this geometry's
    /// local→global indices that lay after the hole.
    pub fn reindex_after_removal(&mut self, removed_start: usize, removed_len: usize) {
        if self.param_offset > removed_start {
            self.param_offset -= removed_len;
        }
    }

    /// Curve sample at normalized chart \(t\) (segment, later circle, …).
    pub fn position_at(&self, t: f64, p: &[f64]) -> Point2 {
        match self.body {
            GeometryBody::Segment => Segment::position_at(self.param_offset, t, p),
        }
    }

    pub fn tangent_at(&self, t: f64, p: &[f64]) -> Vec2 {
        match self.body {
            GeometryBody::Segment => Segment::tangent_at(self.param_offset, t, p),
        }
    }

    pub fn normal_at(&self, t: f64, p: &[f64]) -> Vec2 {
        self.tangent_at(t, p).left_normal()
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

/// Line-segment chart helpers. Parameters live at
/// `p[offset..offset+STRIDE]` on the enclosing [`Geometry`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Segment;

impl Segment {
    /// Number of State parameters for one segment (`x1,y1,x2,y2`).
    pub const STRIDE: usize = 4;

    /// Initial State parameters for a segment from endpoints `(a, b)`.
    pub fn params(a: Point2, b: Point2) -> [f64; 4] {
        [a.x, a.y, b.x, b.y]
    }

    /// Build a document [`Geometry`] with a segment body.
    pub fn into_geometry(id: GeometryId, role: GeometryRole, param_offset: usize) -> Geometry {
        Geometry {
            id,
            role,
            param_offset,
            body: GeometryBody::Segment,
        }
    }

    pub fn start(offset: usize, p: &[f64]) -> Point2 {
        Point2::new(p[offset], p[offset + 1])
    }

    pub fn end(offset: usize, p: &[f64]) -> Point2 {
        Point2::new(p[offset + 2], p[offset + 3])
    }

    /// Free direction vector from start to end (`end - start`).
    pub fn direction(offset: usize, p: &[f64]) -> Vec2 {
        Self::end(offset, p) - Self::start(offset, p)
    }

    pub fn position_at(offset: usize, t: f64, p: &[f64]) -> Point2 {
        let a = Self::start(offset, p);
        let b = Self::end(offset, p);
        Point2::new(
            (1.0 - t) * a.x + t * b.x,
            (1.0 - t) * a.y + t * b.y,
        )
    }

    pub fn tangent_at(offset: usize, _t: f64, p: &[f64]) -> Vec2 {
        // γ'(t) = B - A (constant); length = segment length
        Self::direction(offset, p)
    }

    pub fn all_param_indices(offset: usize) -> [usize; 4] {
        [offset, offset + 1, offset + 2, offset + 3]
    }

    /// Point formula for the start endpoint of the segment identified by `id`.
    pub fn start_expr(id: GeometryId) -> PointExpr {
        PointExpr(PointFormula::StartPoint { geometry: id })
    }

    /// Point formula for the end endpoint of the segment identified by `id`.
    pub fn end_expr(id: GeometryId) -> PointExpr {
        PointExpr(PointFormula::EndPoint { geometry: id })
    }

    /// Point formula for a chart sample on the segment identified by `id`.
    pub fn on_curve_expr(id: GeometryId, param: CurveParam) -> PointExpr {
        PointExpr(PointFormula::OnCurve {
            geometry: id,
            param,
        })
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
}

/// Private formula catalog for [`PointExpr`] (Model B: opaque handle, closed enum).
#[derive(Clone, Copy, Debug, PartialEq)]
enum PointFormula {
    StartPoint { geometry: GeometryId },
    EndPoint { geometry: GeometryId },
    OnCurve {
        geometry: GeometryId,
        param: CurveParam,
    },
}

/// Opaque point expression: map from State parameters to a [`Point2`].
///
/// Construct via geometry factories (e.g. [`Segment::start_expr`]); constraints
/// and Interact only call [`Self::evaluate`] / Jacobian helpers — they do not
/// match on formula kinds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointExpr(PointFormula);

impl PointExpr {
    pub fn evaluate(&self, store: &impl GeometryStore, p: &[f64]) -> Point2 {
        match self.0 {
            PointFormula::StartPoint { geometry } => {
                let g = store.expect_geometry(geometry);
                debug_assert!(matches!(g.body, GeometryBody::Segment));
                Segment::start(g.param_offset, p)
            }
            PointFormula::EndPoint { geometry } => {
                let g = store.expect_geometry(geometry);
                debug_assert!(matches!(g.body, GeometryBody::Segment));
                Segment::end(g.param_offset, p)
            }
            PointFormula::OnCurve { geometry, param } => {
                store
                    .expect_geometry(geometry)
                    .position_at(param.chart_t(), p)
            }
        }
    }

    pub fn param_indices(&self, store: &impl GeometryStore) -> Vec<usize> {
        match self.0 {
            PointFormula::StartPoint { geometry } => {
                let g = store.expect_geometry(geometry);
                debug_assert!(matches!(g.body, GeometryBody::Segment));
                let o = g.param_offset;
                vec![o, o + 1]
            }
            PointFormula::EndPoint { geometry } => {
                let g = store.expect_geometry(geometry);
                debug_assert!(matches!(g.body, GeometryBody::Segment));
                let o = g.param_offset + 2;
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
            PointFormula::StartPoint { .. } | PointFormula::EndPoint { .. } => {
                assert_eq!(dx.len(), 2);
                assert_eq!(dy.len(), 2);
                dx.fill(0.0);
                dy.fill(0.0);
                dx[0] = 1.0;
                dy[1] = 1.0;
            }
            PointFormula::OnCurve { geometry, param } => {
                let g = store.expect_geometry(geometry);
                let n = g.connectivity().len();
                assert_eq!(dx.len(), n);
                assert_eq!(dy.len(), n);
                dx.fill(0.0);
                dy.fill(0.0);
                debug_assert!(matches!(g.body, GeometryBody::Segment));
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
            PointFormula::StartPoint { geometry }
            | PointFormula::EndPoint { geometry }
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
                let g = store.expect_geometry(geometry);
                debug_assert!(matches!(g.body, GeometryBody::Segment));
                Segment::direction(g.param_offset, p)
            }
            VecFormula::TangentAt { geometry, param } => store
                .expect_geometry(geometry)
                .tangent_at(param.chart_t(), p),
            VecFormula::NormalAt { geometry, param } => store
                .expect_geometry(geometry)
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
