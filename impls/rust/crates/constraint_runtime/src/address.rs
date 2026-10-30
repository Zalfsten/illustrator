//! Addresses of points, free vectors and lengths on named geometries.
//!
//! Grammar: `<name>.<suffix>` with suffix `start`, `end`, `center`, `origin`, `at(t)`
//! (points); `direction`, `tangent(t)`, `normal(t)` (free vectors); `length`,
//! `radius` (lengths). `t` is a normalized curve parameter in \([0, 1]\); no
//! whitespace. Resolution goes through the capability-based factories on
//! [`Document`], so a missing capability is an error, not a panic.

use crate::document::Document;
use crate::expr::{LengthExpr, PointExpr, VecExpr};
use crate::geom::CurveParam;
use crate::geometry::{LengthFormula, PointFormula, VecFormula};
use crate::names::{is_valid_name, ObjectId};

/// Resolved address: points and free vectors stay distinct.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Address {
    Point(PointExpr),
    Vector(VecExpr),
    Length(LengthExpr),
}

/// Why an address does not resolve.
#[derive(Clone, Debug, PartialEq)]
pub enum AddressError {
    /// Not `<name>.<suffix>` with a known suffix and argument shape.
    Malformed,
    UnknownName(String),
    /// The name denotes a constraint.
    NotAGeometry(String),
    /// Curve parameter not a finite number in \([0, 1]\).
    ParamOutOfRange(f64),
    /// The geometry lacks the capability the suffix needs.
    MissingCapability {
        name: String,
        capability: &'static str,
    },
}

impl std::fmt::Display for AddressError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AddressError::Malformed => f.write_str("malformed address"),
            AddressError::UnknownName(n) => write!(f, "no object named {n}"),
            AddressError::NotAGeometry(n) => write!(f, "{n} is a constraint, not a geometry"),
            AddressError::ParamOutOfRange(t) => write!(f, "curve parameter {t} not in [0, 1]"),
            AddressError::MissingCapability { name, capability } => {
                write!(f, "{name} has no {capability} capability")
            }
        }
    }
}

impl std::error::Error for AddressError {}

/// `ident` or `ident(arg)`; `None` if malformed.
fn parse_suffix(suffix: &str) -> Option<(&str, Option<f64>)> {
    match suffix.split_once('(') {
        None => Some((suffix, None)),
        Some((ident, rest)) => {
            let arg = rest.strip_suffix(')')?;
            Some((ident, Some(arg.parse().ok()?)))
        }
    }
}

fn normalized(t: f64) -> Result<CurveParam, AddressError> {
    if (0.0..=1.0).contains(&t) {
        Ok(CurveParam::Normalized(t))
    } else {
        Err(AddressError::ParamOutOfRange(t))
    }
}

impl Document {
    /// Resolve `<name>.<suffix>` to an expression (see the module docs).
    pub fn resolve_address(&self, address: &str) -> Result<Address, AddressError> {
        use Address::{Length, Point, Vector};

        let (name, suffix) = address.split_once('.').ok_or(AddressError::Malformed)?;
        let (ident, arg) = parse_suffix(suffix).ok_or(AddressError::Malformed)?;
        if !is_valid_name(name) {
            return Err(AddressError::Malformed);
        }
        let id = match self.lookup(name) {
            Some(ObjectId::Geometry(id)) => id,
            Some(ObjectId::Constraint(_)) => return Err(AddressError::NotAGeometry(name.into())),
            None => return Err(AddressError::UnknownName(name.into())),
        };

        let (expr, capability) = match (ident, arg) {
            ("start", None) => (self.start_expr(id).map(Point), "endpoints"),
            ("end", None) => (self.end_expr(id).map(Point), "endpoints"),
            ("center", None) => (self.center_expr(id).map(Point), "circular"),
            ("origin", None) => (self.origin_expr(id).map(Point), "anchor"),
            ("at", Some(t)) => (self.on_curve_expr(id, normalized(t)?).map(Point), "curve"),
            ("direction", None) => (self.direction_expr(id).map(Vector), "line"),
            ("tangent", Some(t)) => (self.tangent_expr(id, normalized(t)?).map(Vector), "curve"),
            ("normal", Some(t)) => (self.normal_expr(id, normalized(t)?).map(Vector), "curve"),
            ("length", None) => (self.length_expr(id).map(Length), "curve"),
            ("radius", None) => (self.radius_expr(id).map(Length), "circular"),
            _ => return Err(AddressError::Malformed),
        };
        expr.ok_or_else(|| AddressError::MissingCapability {
            name: name.into(),
            capability,
        })
    }

    /// Address of `e` (`seg1.start`, `seg1.at(0.5)`), or `None` if it has none
    /// (e.g. a `Uniform` curve parameter) or its geometry is unnamed.
    pub fn point_address(&self, e: &PointExpr) -> Option<String> {
        let suffix = match e.formula {
            PointFormula::Start => "start".into(),
            PointFormula::End => "end".into(),
            PointFormula::Center => "center".into(),
            PointFormula::Origin => "origin".into(),
            PointFormula::OnCurve {
                param: CurveParam::Normalized(t),
            } => format!("at({t})"),
            PointFormula::OnCurve { .. } => return None,
        };
        self.qualified(e.geometry, &suffix)
    }

    /// Address of `e` (`seg1.direction`, `seg1.tangent(0.5)`), or `None`.
    pub fn vec_address(&self, e: &VecExpr) -> Option<String> {
        let suffix = match e.formula {
            VecFormula::Direction => "direction".into(),
            VecFormula::TangentAt {
                param: CurveParam::Normalized(t),
            } => format!("tangent({t})"),
            VecFormula::NormalAt {
                param: CurveParam::Normalized(t),
            } => format!("normal({t})"),
            VecFormula::TangentAt { .. } | VecFormula::NormalAt { .. } => return None,
        };
        self.qualified(e.geometry, &suffix)
    }

    /// Address of `e` (`seg1.length`, `arc1.radius`), or `None`.
    pub fn length_address(&self, e: &LengthExpr) -> Option<String> {
        let suffix = match e.formula {
            LengthFormula::Length => "length",
            LengthFormula::Radius => "radius",
        };
        self.qualified(e.geometry, suffix)
    }

    fn qualified(&self, id: crate::geometry::GeometryId, suffix: &str) -> Option<String> {
        self.geometry_name(id)
            .map(|name| format!("{name}.{suffix}"))
    }
}

#[cfg(test)]
#[path = "tests_address.rs"]
mod tests_address;
