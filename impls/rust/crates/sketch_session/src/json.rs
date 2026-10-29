//! Interchange form of commands: one flat JSON object per command.
//!
//! `op` selects the operation; `create` / `constrain` also carry `kind`, and
//! the kind's parameters are fields of the same object. Reading is strict:
//! unknown fields, operations and kinds are errors. The object is buffered as
//! a JSON map, the reserved fields are taken out and the rest goes to the
//! kind's parameter type (`deny_unknown_fields`), so typos never fall back to
//! defaults.

use std::fmt;

use schemars::JsonSchema;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Map, Value};

use constraint_runtime::GeometryRole;

use crate::catalog::{ConstraintSpec, GeometrySpec};
use crate::command::{Command, DeclCommand};
use crate::wire::Role;

/// Operations of the interchange form.
pub const OPS: &[&str] = &["create", "constrain", "delete", "rename", "set_role"];

/// Field names no kind may use for its parameters.
pub const RESERVED: &[&str] = &["op", "kind", "name", "role"];

/// Fields of `delete`.
#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct DeleteFields {
    pub name: String,
}

/// Fields of `rename`.
#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct RenameFields {
    pub name: String,
    pub new_name: String,
}

/// Fields of `set_role`.
#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct SetRoleFields {
    pub name: String,
    pub role: Role,
}

#[derive(Serialize)]
struct WithOp<F> {
    op: &'static str,
    #[serde(flatten)]
    fields: F,
}

/// Interchange text could not be read as a command.
#[derive(Debug)]
pub struct ReadError(serde_json::Error);

/// A command could not be written (e.g. a non-finite number).
#[derive(Debug)]
pub struct WriteError(serde_json::Error);

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "cannot read command: {}", self.0)
    }
}

impl fmt::Display for WriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "cannot write command: {}", self.0)
    }
}

impl std::error::Error for ReadError {}
impl std::error::Error for WriteError {}

impl Command {
    pub fn from_json(text: &str) -> Result<Self, ReadError> {
        serde_json::from_str(text).map_err(ReadError)
    }

    pub fn to_json(&self) -> Result<String, WriteError> {
        serde_json::to_string(self).map_err(WriteError)
    }
}

impl DeclCommand {
    pub fn from_json(text: &str) -> Result<Self, ReadError> {
        serde_json::from_str(text).map_err(ReadError)
    }

    pub fn to_json(&self) -> Result<String, WriteError> {
        serde_json::to_string(self).map_err(WriteError)
    }

    /// A whole list; every entry is checked before any is returned.
    pub fn list_from_json(text: &str) -> Result<Vec<Self>, ReadError> {
        serde_json::from_str(text).map_err(ReadError)
    }

    pub fn list_to_json(list: &[Self]) -> Result<String, WriteError> {
        serde_json::to_string(list).map_err(WriteError)
    }
}

/// Kind parameters must not shadow the reserved fields (checked once).
fn debug_check_reserved() {
    #[cfg(debug_assertions)]
    {
        static CLASHES: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
        let n = *CLASHES.get_or_init(|| crate::schema::reserved_clashes().len());
        debug_assert_eq!(n, 0, "a kind parameter uses a reserved field name");
    }
}

impl Serialize for Command {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        debug_check_reserved();
        match self {
            Command::Create {
                name,
                role,
                geometry,
            } => geometry.serialize_tagged("create", name.as_deref(), Some((*role).into()), s),
            Command::Constrain { name, constraint } => {
                constraint.serialize_tagged("constrain", name.as_deref(), s)
            }
            Command::Delete { name } => WithOp {
                op: "delete",
                fields: DeleteFields { name: name.clone() },
            }
            .serialize(s),
            Command::Rename { name, new_name } => WithOp {
                op: "rename",
                fields: RenameFields {
                    name: name.clone(),
                    new_name: new_name.clone(),
                },
            }
            .serialize(s),
            Command::SetRole { name, role } => WithOp {
                op: "set_role",
                fields: SetRoleFields {
                    name: name.clone(),
                    role: (*role).into(),
                },
            }
            .serialize(s),
        }
    }
}

impl Serialize for DeclCommand {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        debug_check_reserved();
        match self {
            DeclCommand::Create {
                name,
                role,
                geometry,
            } => geometry.serialize_tagged("create", Some(name), Some((*role).into()), s),
            DeclCommand::Constrain { name, constraint } => {
                constraint.serialize_tagged("constrain", Some(name), s)
            }
        }
    }
}

impl<'de> Deserialize<'de> for Command {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let mut m = Map::deserialize(d)?;
        command(&mut m).map_err(D::Error::custom)
    }
}

impl<'de> Deserialize<'de> for DeclCommand {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let mut m = Map::deserialize(d)?;
        decl_command(&mut m).map_err(D::Error::custom)
    }
}

type JsonResult<T> = Result<T, serde_json::Error>;

fn command(m: &mut Map<String, Value>) -> JsonResult<Command> {
    let op = required_str(m, "op")?;
    Ok(match op.as_str() {
        "create" => {
            let (name, role, geometry) = create(m)?;
            Command::Create {
                name,
                role,
                geometry,
            }
        }
        "constrain" => {
            let (name, constraint) = constrain(m)?;
            Command::Constrain { name, constraint }
        }
        "delete" => {
            let f: DeleteFields = rest(m)?;
            Command::Delete { name: f.name }
        }
        "rename" => {
            let f: RenameFields = rest(m)?;
            Command::Rename {
                name: f.name,
                new_name: f.new_name,
            }
        }
        "set_role" => {
            let f: SetRoleFields = rest(m)?;
            Command::SetRole {
                name: f.name,
                role: f.role.into(),
            }
        }
        other => return Err(serde_json::Error::unknown_variant(other, OPS)),
    })
}

fn decl_command(m: &mut Map<String, Value>) -> JsonResult<DeclCommand> {
    let op = required_str(m, "op")?;
    let missing_name =
        || serde_json::Error::custom("missing field `name` (required in the declarative subset)");
    Ok(match op.as_str() {
        "create" => {
            let (name, role, geometry) = create(m)?;
            DeclCommand::Create {
                name: name.ok_or_else(missing_name)?,
                role,
                geometry,
            }
        }
        "constrain" => {
            let (name, constraint) = constrain(m)?;
            DeclCommand::Constrain {
                name: name.ok_or_else(missing_name)?,
                constraint,
            }
        }
        other if OPS.contains(&other) => {
            return Err(serde_json::Error::custom(format!(
                "operation `{other}` is not in the declarative subset"
            )))
        }
        other => return Err(serde_json::Error::unknown_variant(other, &OPS[..2])),
    })
}

fn create(m: &mut Map<String, Value>) -> JsonResult<(Option<String>, GeometryRole, GeometrySpec)> {
    let kind = required_str(m, "kind")?;
    let name = optional_str(m, "name")?;
    let role = match m.remove("role") {
        Some(v) => Role::deserialize(v)?.into(),
        None => Default::default(),
    };
    let geometry = GeometrySpec::from_fields(&kind, Value::Object(std::mem::take(m)))?;
    Ok((name, role, geometry))
}

fn constrain(m: &mut Map<String, Value>) -> JsonResult<(Option<String>, ConstraintSpec)> {
    let kind = required_str(m, "kind")?;
    let name = optional_str(m, "name")?;
    let constraint = ConstraintSpec::from_fields(&kind, Value::Object(std::mem::take(m)))?;
    Ok((name, constraint))
}

fn rest<T: serde::de::DeserializeOwned>(m: &mut Map<String, Value>) -> JsonResult<T> {
    serde_json::from_value(Value::Object(std::mem::take(m)))
}

fn required_str(m: &mut Map<String, Value>, key: &'static str) -> JsonResult<String> {
    optional_str(m, key)?.ok_or_else(|| serde_json::Error::missing_field(key))
}

fn optional_str(m: &mut Map<String, Value>, key: &'static str) -> JsonResult<Option<String>> {
    match m.remove(key) {
        None => Ok(None),
        Some(Value::String(s)) => Ok(Some(s)),
        Some(_) => Err(serde_json::Error::custom(format!(
            "`{key}` must be a string"
        ))),
    }
}

#[cfg(test)]
#[path = "tests_json.rs"]
mod tests_json;
