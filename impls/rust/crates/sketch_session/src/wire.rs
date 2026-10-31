//! Interchange adapters: points and vectors as pairs, finite numbers, typed
//! addresses, label and style enums, colours.
//!
//! Writing a non-finite number is an error (serde_json would otherwise write
//! `null`, which does not read back); reading rejects it as well.

use std::fmt;

use constraint_runtime::{Arrow, Dash, GeometryRole, HAlign, Markup, Rgb, VAlign};
use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{de, ser, Deserialize, Deserializer, Serialize, Serializer};

fn check<E: fmt::Display>(v: f64, err: impl FnOnce(&'static str) -> E) -> Result<f64, E> {
    if v.is_finite() {
        Ok(v)
    } else {
        Err(err("non-finite number"))
    }
}

/// `f64` that must be finite (`#[serde(with = "wire::finite")]`).
pub mod finite {
    use super::*;

    pub fn serialize<S: Serializer>(v: &f64, s: S) -> Result<S::Ok, S::Error> {
        check(*v, ser::Error::custom)?.serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<f64, D::Error> {
        check(f64::deserialize(d)?, de::Error::custom)
    }
}

/// [`Point2`](constraint_runtime::Point2) as `[x, y]` (`#[serde(with = "wire::point")]`).
pub mod point {
    use constraint_runtime::Point2;

    use super::*;

    pub fn serialize<S: Serializer>(p: &Point2, s: S) -> Result<S::Ok, S::Error> {
        [
            check(p.x, ser::Error::custom)?,
            check(p.y, ser::Error::custom)?,
        ]
        .serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Point2, D::Error> {
        let [x, y] = <[f64; 2]>::deserialize(d)?;
        Ok(Point2::new(
            check(x, de::Error::custom)?,
            check(y, de::Error::custom)?,
        ))
    }

    /// Schema of a point (`#[schemars(schema_with = "wire::point::schema")]`).
    pub fn schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "description": "Point [x, y].",
            "type": "array",
            "items": { "type": "number" },
            "minItems": 2,
            "maxItems": 2
        })
    }
}

/// Free vector [`Vec2`](constraint_runtime::Vec2) as `[dx, dy]`
/// (`#[serde(with = "wire::vector")]`).
pub mod vector {
    use constraint_runtime::Vec2;

    use super::*;

    pub fn serialize<S: Serializer>(v: &Vec2, s: S) -> Result<S::Ok, S::Error> {
        [
            check(v.x, ser::Error::custom)?,
            check(v.y, ser::Error::custom)?,
        ]
        .serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec2, D::Error> {
        let [x, y] = <[f64; 2]>::deserialize(d)?;
        Ok(Vec2::new(
            check(x, de::Error::custom)?,
            check(y, de::Error::custom)?,
        ))
    }

    /// Schema of a free vector (`#[schemars(schema_with = "wire::vector::schema")]`).
    pub fn schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "description": "Free vector [dx, dy].",
            "type": "array",
            "items": { "type": "number" },
            "minItems": 2,
            "maxItems": 2
        })
    }
}

/// Optional finite `f64`, absent when `None`
/// (`#[serde(default, skip_serializing_if = "Option::is_none", with = "wire::finite_option")]`).
pub mod finite_option {
    use super::*;

    pub fn serialize<S: Serializer>(v: &Option<f64>, s: S) -> Result<S::Ok, S::Error> {
        match v {
            Some(v) => check(*v, ser::Error::custom)?.serialize(s),
            None => s.serialize_none(),
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<f64>, D::Error> {
        Ok(Some(check(f64::deserialize(d)?, de::Error::custom)?))
    }
}

/// Optional field that, when present, must hold a value (`null` is rejected,
/// not read as absent)
/// (`#[serde(default, skip_serializing_if = "Option::is_none", with = "wire::present")]`
/// plus `#[schemars(with = "T")]`).
pub mod present {
    use super::*;

    pub fn serialize<T: Serialize, S: Serializer>(v: &Option<T>, s: S) -> Result<S::Ok, S::Error> {
        v.serialize(s)
    }

    pub fn deserialize<'de, T: Deserialize<'de>, D: Deserializer<'de>>(
        d: D,
    ) -> Result<Option<T>, D::Error> {
        T::deserialize(d).map(Some)
    }
}

/// Optional free vector, absent when `None`
/// (`#[serde(default, skip_serializing_if = "Option::is_none", with = "wire::vector_option")]`).
pub mod vector_option {
    use constraint_runtime::Vec2;

    use super::*;

    pub fn serialize<S: Serializer>(v: &Option<Vec2>, s: S) -> Result<S::Ok, S::Error> {
        match v {
            Some(v) => vector::serialize(v, s),
            None => s.serialize_none(),
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Vec2>, D::Error> {
        vector::deserialize(d).map(Some)
    }
}

/// Field that may be absent (`None`), `null` (`Some(None)`) or a finite number
/// (`Some(Some(v))`), e.g. a label size in `set`
/// (`#[serde(default, skip_serializing_if = "Option::is_none", with = "wire::finite_or_null")]`).
pub mod finite_or_null {
    use super::*;

    pub fn serialize<S: Serializer>(v: &Option<Option<f64>>, s: S) -> Result<S::Ok, S::Error> {
        match v {
            Some(Some(v)) => check(*v, ser::Error::custom)?.serialize(s),
            Some(None) | None => s.serialize_none(),
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Option<f64>>, D::Error> {
        match Option::<f64>::deserialize(d)? {
            Some(v) => Ok(Some(Some(check(v, de::Error::custom)?))),
            None => Ok(Some(None)),
        }
    }

    /// A positive number or `null` (`#[schemars(schema_with = "wire::finite_or_null::schema")]`).
    pub fn schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "description": "Size in pt, greater than zero; null to inherit the document font size.",
            "type": ["number", "null"],
            "exclusiveMinimum": 0
        })
    }
}

/// A number greater than zero (`#[schemars(schema_with = "wire::positive_schema")]`).
pub fn positive_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({ "type": "number", "exclusiveMinimum": 0 })
}

/// A stroke width override (`#[schemars(schema_with = "wire::width_schema")]`).
pub fn width_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "description": "Stroke width in pt, greater than zero.",
        "type": "number",
        "exclusiveMinimum": 0
    })
}

/// A stroke width or `null` (`#[schemars(schema_with = "wire::width_or_null_schema")]`).
pub fn width_or_null_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "description": "Stroke width in pt, greater than zero; null to remove it.",
        "type": ["number", "null"],
        "exclusiveMinimum": 0
    })
}

/// Field that may be absent (`None`), `null` (`Some(None)`) or a value
/// (`Some(Some(v))`)
/// (`#[serde(default, skip_serializing_if = "Option::is_none", with = "wire::or_null")]`
/// plus `#[schemars(with = "Option<T>")]`).
pub mod or_null {
    use super::*;

    pub fn serialize<T: Serialize, S: Serializer>(v: &Option<Option<T>>, s: S) -> Result<S::Ok, S::Error> {
        match v {
            Some(Some(v)) => v.serialize(s),
            Some(None) | None => s.serialize_none(),
        }
    }

    pub fn deserialize<'de, T: Deserialize<'de>, D: Deserializer<'de>>(
        d: D,
    ) -> Result<Option<Option<T>>, D::Error> {
        Option::<T>::deserialize(d).map(Some)
    }
}

/// `#rrggbb` with hex digits in either case.
pub fn parse_color(s: &str) -> Option<Rgb> {
    let hex = s.strip_prefix('#')?;
    if hex.len() != 6 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let byte = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).ok();
    Some(Rgb(byte(0)?, byte(2)?, byte(4)?))
}

/// sRGB colour: reads `#rrggbb` in either case, writes lower case.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WireColor(pub Rgb);

impl fmt::Display for WireColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Rgb(r, g, b) = self.0;
        write!(f, "#{r:02x}{g:02x}{b:02x}")
    }
}

impl Serialize for WireColor {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for WireColor {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        parse_color(&s)
            .map(WireColor)
            .ok_or_else(|| de::Error::custom(format!("invalid value for color: {s:?} is not #rrggbb")))
    }
}

impl JsonSchema for WireColor {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "Color".into()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "description": "sRGB colour #rrggbb.",
            "type": "string",
            "pattern": "^#[0-9A-Fa-f]{6}$"
        })
    }
}

/// Interchange mirror of a core enum (lowercase tags; the core stays serde-free).
macro_rules! wire_enum {
    ($(#[$doc:meta])* $name:ident => $core:ident { $($v:ident),+ $(,)? }) => {
        $(#[$doc])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
        #[serde(rename_all = "lowercase")]
        pub enum $name {
            $($v,)+
        }

        impl Default for $name {
            fn default() -> Self {
                $core::default().into()
            }
        }

        impl From<$core> for $name {
            fn from(x: $core) -> Self {
                match x {
                    $($core::$v => $name::$v,)+
                }
            }
        }

        impl From<$name> for $core {
            fn from(x: $name) -> Self {
                match x {
                    $($name::$v => $core::$v,)+
                }
            }
        }
    };
}

wire_enum!(
    /// Label markup: `"latex"` or `"text"`.
    WireMarkup => Markup { Latex, Text }
);
wire_enum!(
    /// Label horizontal alignment: `"left"`, `"center"` or `"right"`.
    WireHAlign => HAlign { Left, Center, Right }
);
wire_enum!(
    /// Label vertical anchor: `"first"` or `"last"`.
    WireVAlign => VAlign { First, Last }
);
wire_enum!(
    /// Dash pattern: `"solid"`, `"dashed"`, `"dotted"` or `"dashdot"`.
    WireDash => Dash { Solid, Dashed, Dotted, DashDot }
);
wire_enum!(
    /// Arrow heads: `"none"`, `"start"`, `"end"` or `"both"`.
    WireArrow => Arrow { None, Start, End, Both }
);

/// Geometry role: `"result"` or `"construction"`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Result,
    Construction,
}

impl From<GeometryRole> for Role {
    fn from(r: GeometryRole) -> Self {
        match r {
            GeometryRole::Result => Role::Result,
            GeometryRole::Construction => Role::Construction,
        }
    }
}

impl From<Role> for GeometryRole {
    fn from(r: Role) -> Self {
        match r {
            Role::Result => GeometryRole::Result,
            Role::Construction => GeometryRole::Construction,
        }
    }
}

macro_rules! address {
    ($(#[$doc:meta])* $name:ident, $example:literal) => {
        $(#[$doc])*
        #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub String);

        impl $name {
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl From<&str> for $name {
            fn from(s: &str) -> Self {
                Self(s.to_owned())
            }
        }

        impl From<String> for $name {
            fn from(s: String) -> Self {
                Self(s)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl JsonSchema for $name {
            fn schema_name() -> std::borrow::Cow<'static, str> {
                stringify!($name).into()
            }

            fn json_schema(_: &mut SchemaGenerator) -> Schema {
                json_schema!({
                    "description": $example,
                    "type": "string"
                })
            }
        }
    };
}

address!(
    /// Address of a point (`seg1.end`, `arc1.center`, `seg1.at(0.5)`).
    PointAddr,
    "Address of a point, e.g. seg1.end or arc1.center."
);
address!(
    /// Address of a free vector (`seg1.direction`, `arc1.tangent(0.5)`).
    VecAddr,
    "Address of a free vector, e.g. seg1.direction or arc1.tangent(0.5)."
);
address!(
    /// Address of a length (`seg1.length`, `arc1.radius`).
    LenAddr,
    "Address of a length, e.g. seg1.length or arc1.radius."
);

#[cfg(test)]
#[path = "tests_wire.rs"]
mod tests_wire;
