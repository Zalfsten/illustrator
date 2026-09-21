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
    /// Chart \(t\) for a line segment (`Normalized` ≡ `Uniform`).
    pub fn segment_chart_t(self) -> f64 {
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
pub struct SegmentId(pub usize);

/// Line segment: endpoint chart, parameters live at `p[param_offset..param_offset+4]`.
#[derive(Clone, Debug, PartialEq)]
pub struct Segment {
    pub id: SegmentId,
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
            PointExpr::SegmentEnd {
                segment: self.id,
                end: SegmentEndKind::Start,
            },
            PointExpr::SegmentEnd {
                segment: self.id,
                end: SegmentEndKind::End,
            },
        )
    }

    pub fn endpoint_param_indices(&self, end: SegmentEndKind) -> [usize; 2] {
        match end {
            SegmentEndKind::Start => [self.param_offset, self.param_offset + 1],
            SegmentEndKind::End => [self.param_offset + 2, self.param_offset + 3],
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
pub enum SegmentEndKind {
    Start,
    End,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PointExpr {
    SegmentEnd {
        segment: SegmentId,
        end: SegmentEndKind,
    },
    OnCurve {
        segment: SegmentId,
        param: CurveParam,
    },
}

impl PointExpr {
    pub fn evaluate(&self, segments: &[Segment], p: &[f64]) -> Point2 {
        match *self {
            PointExpr::SegmentEnd { segment, end } => {
                let s = find_segment(segments, segment);
                match end {
                    SegmentEndKind::Start => s.start(p),
                    SegmentEndKind::End => s.end(p),
                }
            }
            PointExpr::OnCurve { segment, param } => {
                let s = find_segment(segments, segment);
                s.position_at(param.segment_chart_t(), p)
            }
        }
    }

    /// Global parameter indices this expression depends on.
    pub fn param_indices(&self, segments: &[Segment]) -> Vec<usize> {
        match *self {
            PointExpr::SegmentEnd { segment, end } => {
                let s = find_segment(segments, segment);
                let ix = s.endpoint_param_indices(end);
                vec![ix[0], ix[1]]
            }
            PointExpr::OnCurve { segment, .. } => {
                let s = find_segment(segments, segment);
                s.all_param_indices().to_vec()
            }
        }
    }

    /// Analytical \(\partial(x,y)/\partial p_i\) for each index in [`Self::param_indices`],
    /// written as parallel `dx` / `dy` slices (same length as connectivity).
    pub fn eval_point_jacobian(
        &self,
        segments: &[Segment],
        _p: &[f64],
        dx: &mut [f64],
        dy: &mut [f64],
    ) {
        let n = self.param_indices(segments).len();
        assert_eq!(dx.len(), n);
        assert_eq!(dy.len(), n);
        dx.fill(0.0);
        dy.fill(0.0);
        match *self {
            PointExpr::SegmentEnd { .. } => {
                dx[0] = 1.0;
                dy[1] = 1.0;
            }
            PointExpr::OnCurve { param, .. } => {
                let t = param.segment_chart_t();
                // pos = (1-t)A + t B; local order [Ax,Ay,Bx,By]
                dx[0] = 1.0 - t;
                dy[1] = 1.0 - t;
                dx[2] = t;
                dy[3] = t;
            }
        }
    }

    pub fn referenced_segment(&self) -> SegmentId {
        match *self {
            PointExpr::SegmentEnd { segment, .. } | PointExpr::OnCurve { segment, .. } => segment,
        }
    }
}

/// Free-vector expression over geometry parameters.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VecExpr {
    SegmentDirection { segment: SegmentId },
    TangentAt {
        segment: SegmentId,
        param: CurveParam,
    },
    NormalAt {
        segment: SegmentId,
        param: CurveParam,
    },
}

impl VecExpr {
    pub fn evaluate(&self, segments: &[Segment], p: &[f64]) -> Vec2 {
        match *self {
            VecExpr::SegmentDirection { segment } => find_segment(segments, segment).direction(p),
            VecExpr::TangentAt { segment, param } => {
                let s = find_segment(segments, segment);
                s.tangent_at(param.segment_chart_t(), p)
            }
            VecExpr::NormalAt { segment, param } => {
                let s = find_segment(segments, segment);
                s.normal_at(param.segment_chart_t(), p)
            }
        }
    }
}

fn find_segment(segments: &[Segment], id: SegmentId) -> &Segment {
    segments
        .iter()
        .find(|s| s.id == id)
        .expect("segment for expression")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editing_state_vector_changes_endpoints() {
        let seg = Segment {
            id: SegmentId(0),
            param_offset: 0,
        };
        let mut p = vec![0.0, 0.0, 1.0, 0.0];
        assert_eq!(seg.end(&p), Point2::new(1.0, 0.0));
        p[2] = 3.0;
        p[3] = 4.0;
        assert_eq!(seg.end(&p), Point2::new(3.0, 4.0));
        assert_eq!(seg.connectivity().global_indices, vec![0, 1, 2, 3]);
    }

    #[test]
    fn point_minus_point_is_vec_direction_is_vec() {
        let a = Point2::new(1.0, 2.0);
        let b = Point2::new(4.0, 6.0);
        let v: Vec2 = b - a;
        assert_eq!(v, Vec2::new(3.0, 4.0));
        assert_eq!(a + v, b);

        let seg = Segment {
            id: SegmentId(0),
            param_offset: 0,
        };
        let p = vec![0.0, 0.0, 1.0, 0.0];
        let d: Vec2 = seg.direction(&p);
        assert_eq!(d, Vec2::new(1.0, 0.0));
        assert_eq!(d.cross(Vec2::new(0.0, 1.0)), 1.0);
        assert_eq!(d.dot(Vec2::new(0.0, 1.0)), 0.0);
    }

    #[test]
    fn segment_curve_position_tangent_normal() {
        let seg = Segment {
            id: SegmentId(0),
            param_offset: 0,
        };
        let p = vec![0.0, 0.0, 4.0, 0.0];
        assert_eq!(seg.position_at(0.0, &p), Point2::new(0.0, 0.0));
        assert_eq!(seg.position_at(0.5, &p), Point2::new(2.0, 0.0));
        assert_eq!(seg.position_at(1.0, &p), Point2::new(4.0, 0.0));
        let tan = seg.tangent_at(0.5, &p);
        assert_eq!(tan, Vec2::new(4.0, 0.0));
        assert!((tan.length() - 4.0).abs() < 1e-12);
        let n = seg.normal_at(0.5, &p);
        assert_eq!(n, Vec2::new(0.0, 4.0));
        assert!((n.dot(tan)).abs() < 1e-12);
        assert!((n.length() - tan.length()).abs() < 1e-12);
        let (start_e, end_e) = seg.endpoints();
        assert_eq!(
            start_e.evaluate(&[seg.clone()], &p),
            seg.position_at(0.0, &p)
        );
        assert_eq!(
            end_e.evaluate(&[seg.clone()], &p),
            seg.position_at(1.0, &p)
        );
    }

    #[test]
    fn oncurve_uniform_matches_normalized_on_segment() {
        let seg = Segment {
            id: SegmentId(0),
            param_offset: 0,
        };
        let segments = vec![seg.clone()];
        let p = vec![1.0, 2.0, 5.0, 8.0];
        let u = 0.25;
        let a = PointExpr::OnCurve {
            segment: SegmentId(0),
            param: CurveParam::Normalized(u),
        }
        .evaluate(&segments, &p);
        let b = PointExpr::OnCurve {
            segment: SegmentId(0),
            param: CurveParam::Uniform(u),
        }
        .evaluate(&segments, &p);
        assert_eq!(a, b);
        assert_eq!(a, seg.position_at(u, &p));

        let mut dx_n = [0.0; 4];
        let mut dy_n = [0.0; 4];
        let mut dx_u = [0.0; 4];
        let mut dy_u = [0.0; 4];
        PointExpr::OnCurve {
            segment: SegmentId(0),
            param: CurveParam::Normalized(u),
        }
        .eval_point_jacobian(&segments, &p, &mut dx_n, &mut dy_n);
        PointExpr::OnCurve {
            segment: SegmentId(0),
            param: CurveParam::Uniform(u),
        }
        .eval_point_jacobian(&segments, &p, &mut dx_u, &mut dy_u);
        assert_eq!(dx_n, dx_u);
        assert_eq!(dy_n, dy_u);
    }

    #[test]
    fn vec_expr_tangent_and_normal() {
        let seg = Segment {
            id: SegmentId(0),
            param_offset: 0,
        };
        let segments = [seg];
        let p = vec![0.0, 0.0, 0.0, 3.0];
        let t = VecExpr::TangentAt {
            segment: SegmentId(0),
            param: CurveParam::Normalized(0.0),
        }
        .evaluate(&segments, &p);
        let n = VecExpr::NormalAt {
            segment: SegmentId(0),
            param: CurveParam::Uniform(0.5),
        }
        .evaluate(&segments, &p);
        assert_eq!(t, Vec2::new(0.0, 3.0));
        assert_eq!(n, Vec2::new(-3.0, 0.0));
    }
}
