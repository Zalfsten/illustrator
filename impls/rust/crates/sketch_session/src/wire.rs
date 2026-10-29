//! Interchange adapters: points as `[x, y]`, finite numbers, typed addresses.
//!
//! Writing a non-finite number is an error (serde_json would otherwise write
//! `null`, which does not read back); reading rejects it as well.

use std::fmt;

use constraint_runtime::GeometryRole;
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
