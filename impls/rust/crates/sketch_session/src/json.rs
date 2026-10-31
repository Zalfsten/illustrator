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

use constraint_runtime::{GeometryRole, StylePatch, StyleProps, StyleRef, DEFAULT_STYLE};

use crate::catalog::{AttrSpec, ConstraintSpec, GeometrySpec};
use crate::command::{Command, DeclCommand};
use crate::wire::{self, Role, WireArrow, WireColor, WireDash};

/// Operations of the interchange form.
pub const OPS: &[&str] = &[
    "create",
    "constrain",
    "delete",
    "rename",
    "set_role",
    "set",
    "set_document",
    "define_style",
    "delete_style",
    "set_style",
];

/// Field names no kind may use for its parameters.
pub const RESERVED: &[&str] = &["op", "kind", "name", "role", "style", "width", "color", "dash", "arrow"];

/// Style fields of `create`, taken out of the object like `role`.
pub(crate) const LOOK_FIELDS: &[&str] = &["style", "width", "color", "dash", "arrow"];

/// Style fields of `create`: the named style (`default` when absent) and
/// the overrides, all optional; written after the kind's fields.
#[derive(Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct LookFields {
    /// Name of the style the geometry refers to; `default` when absent.
    #[serde(default, skip_serializing_if = "Option::is_none", with = "wire::present")]
    #[schemars(with = "String")]
    pub style: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "wire::finite_option"
    )]
    #[schemars(schema_with = "wire::width_schema")]
    pub width: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none", with = "wire::present")]
    #[schemars(with = "WireColor")]
    pub color: Option<WireColor>,
    #[serde(default, skip_serializing_if = "Option::is_none", with = "wire::present")]
    #[schemars(with = "WireDash")]
    pub dash: Option<WireDash>,
    #[serde(default, skip_serializing_if = "Option::is_none", with = "wire::present")]
    #[schemars(with = "WireArrow")]
    pub arrow: Option<WireArrow>,
}

impl From<&StyleRef> for LookFields {
    fn from(r: &StyleRef) -> Self {
        let o = &r.overrides;
        Self {
            style: (r.style != DEFAULT_STYLE).then(|| r.style.clone()),
            width: o.width,
            color: o.color.map(WireColor),
            dash: o.dash.map(Into::into),
            arrow: o.arrow.map(Into::into),
        }
    }
}

impl From<LookFields> for StyleRef {
    fn from(f: LookFields) -> Self {
        Self {
            style: f.style.unwrap_or_else(|| DEFAULT_STYLE.to_owned()),
            overrides: StyleProps {
                width: f.width,
                color: f.color.map(|c| c.0),
                dash: f.dash.map(Into::into),
                arrow: f.arrow.map(Into::into),
            },
        }
    }
}

/// Properties of one named style in a sketch file's `styles` table.
#[derive(Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PropFields {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "wire::finite_option"
    )]
    pub width: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none", with = "wire::present")]
    pub color: Option<WireColor>,
    #[serde(default, skip_serializing_if = "Option::is_none", with = "wire::present")]
    pub dash: Option<WireDash>,
    #[serde(default, skip_serializing_if = "Option::is_none", with = "wire::present")]
    pub arrow: Option<WireArrow>,
}

impl From<&StyleProps> for PropFields {
    fn from(p: &StyleProps) -> Self {
        Self {
            width: p.width,
            color: p.color.map(WireColor),
            dash: p.dash.map(Into::into),
            arrow: p.arrow.map(Into::into),
        }
    }
}

impl From<PropFields> for StyleProps {
    fn from(f: PropFields) -> Self {
        Self {
            width: f.width,
            color: f.color.map(|c| c.0),
            dash: f.dash.map(Into::into),
            arrow: f.arrow.map(Into::into),
        }
    }
}

/// Style properties of `define_style` / `set_style`: absent leaves a
/// property, `null` removes it.
#[derive(Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct PatchFields {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "wire::finite_or_null"
    )]
    #[schemars(schema_with = "wire::width_or_null_schema")]
    pub width: Option<Option<f64>>,
    #[serde(default, skip_serializing_if = "Option::is_none", with = "wire::or_null")]
    #[schemars(with = "Option<WireColor>")]
    pub color: Option<Option<WireColor>>,
    #[serde(default, skip_serializing_if = "Option::is_none", with = "wire::or_null")]
    #[schemars(with = "Option<WireDash>")]
    pub dash: Option<Option<WireDash>>,
    #[serde(default, skip_serializing_if = "Option::is_none", with = "wire::or_null")]
    #[schemars(with = "Option<WireArrow>")]
    pub arrow: Option<Option<WireArrow>>,
}

impl From<&StylePatch> for PatchFields {
    fn from(p: &StylePatch) -> Self {
        Self {
            width: p.width,
            color: p.color.map(|c| c.map(WireColor)),
            dash: p.dash.map(|d| d.map(Into::into)),
            arrow: p.arrow.map(|a| a.map(Into::into)),
        }
    }
}

impl From<PatchFields> for StylePatch {
    fn from(f: PatchFields) -> Self {
        Self {
            width: f.width,
            color: f.color.map(|c| c.map(|c| c.0)),
            dash: f.dash.map(|d| d.map(Into::into)),
            arrow: f.arrow.map(|a| a.map(Into::into)),
        }
    }
}

/// `define_style`: the style name, then the properties.
#[derive(Serialize)]
struct DefineStyleOut<'a> {
    style: &'a str,
    #[serde(flatten)]
    patch: PatchFields,
}

/// `set_style`: the geometry, the style (if changed), then the overrides.
#[derive(Serialize)]
struct SetStyleOut<'a> {
    name: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    style: Option<&'a str>,
    #[serde(flatten)]
    patch: PatchFields,
}

/// Fields of `delete_style`.
#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct DeleteStyleFields {
    pub style: String,
}

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

/// Fields of `set_document` (each optional, at least one present).
#[derive(Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct SetDocumentFields {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "wire::finite_option"
    )]
    /// Millimetres per world unit, greater than zero.
    #[schemars(schema_with = "wire::positive_schema")]
    pub scale: Option<f64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "wire::finite_option"
    )]
    /// Font size of labels that inherit theirs, in pt, greater than zero.
    #[schemars(schema_with = "wire::positive_schema")]
    pub font_size: Option<f64>,
    /// LaTeX source with macros for every label in latex markup.
    #[serde(default, skip_serializing_if = "Option::is_none", with = "wire::present")]
    #[schemars(with = "String")]
    pub preamble: Option<String>,
}

#[derive(Serialize)]
struct WithOp<F> {
    op: &'static str,
    #[serde(flatten)]
    fields: F,
}

/// Interchange text could not be read as a command.
#[derive(Debug)]
pub struct ReadError(pub(crate) serde_json::Error);

/// A command could not be written (e.g. a non-finite number).
#[derive(Debug)]
pub struct WriteError(pub(crate) serde_json::Error);

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
                look,
                geometry,
            } => geometry.serialize_tagged("create", name.as_deref(), Some((*role).into()), look, s),
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
            Command::Set { name, attrs } => attrs.serialize_tagged(name, s),
            Command::SetDocument {
                scale,
                font_size,
                preamble,
            } => WithOp {
                op: "set_document",
                fields: SetDocumentFields {
                    scale: *scale,
                    font_size: *font_size,
                    preamble: preamble.clone(),
                },
            }
            .serialize(s),
            Command::DefineStyle { style, patch } => WithOp {
                op: "define_style",
                fields: DefineStyleOut {
                    style,
                    patch: patch.into(),
                },
            }
            .serialize(s),
            Command::DeleteStyle { style } => WithOp {
                op: "delete_style",
                fields: DeleteStyleFields {
                    style: style.clone(),
                },
            }
            .serialize(s),
            Command::SetStyle { name, style, patch } => WithOp {
                op: "set_style",
                fields: SetStyleOut {
                    name,
                    style: style.as_deref(),
                    patch: patch.into(),
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
                look,
                geometry,
            } => geometry.serialize_tagged("create", Some(name), Some((*role).into()), look, s),
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
            let (name, role, look, geometry) = create(m)?;
            Command::Create {
                name,
                role,
                look,
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
        "set" => {
            let kind = required_str(m, "kind")?;
            let name = required_str(m, "name")?;
            let attrs = AttrSpec::from_fields(&kind, Value::Object(std::mem::take(m)))?;
            Command::Set { name, attrs }
        }
        "set_document" => {
            let f: SetDocumentFields = rest(m)?;
            if f == SetDocumentFields::default() {
                return Err(serde_json::Error::custom("no setting given"));
            }
            Command::SetDocument {
                scale: f.scale,
                font_size: f.font_size,
                preamble: f.preamble,
            }
        }
        "define_style" => {
            let style = required_str(m, "style")?;
            let patch: PatchFields = rest(m)?;
            if patch == PatchFields::default() {
                return Err(serde_json::Error::custom("no style property given"));
            }
            Command::DefineStyle {
                style,
                patch: patch.into(),
            }
        }
        "delete_style" => {
            let f: DeleteStyleFields = rest(m)?;
            Command::DeleteStyle { style: f.style }
        }
        "set_style" => {
            let name = required_str(m, "name")?;
            let style = optional_str(m, "style")?;
            let patch: PatchFields = rest(m)?;
            if style.is_none() && patch == PatchFields::default() {
                return Err(serde_json::Error::custom("no style or style property given"));
            }
            Command::SetStyle {
                name,
                style,
                patch: patch.into(),
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
            let (name, role, look, geometry) = create(m)?;
            DeclCommand::Create {
                name: name.ok_or_else(missing_name)?,
                role,
                look,
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

type Create = (Option<String>, GeometryRole, StyleRef, GeometrySpec);

fn create(m: &mut Map<String, Value>) -> JsonResult<Create> {
    let kind = required_str(m, "kind")?;
    let name = optional_str(m, "name")?;
    let role = match m.remove("role") {
        Some(v) => Role::deserialize(v)?.into(),
        None => Default::default(),
    };
    let look: Map<String, Value> = LOOK_FIELDS
        .iter()
        .filter_map(|&k| m.remove(k).map(|v| (k.to_owned(), v)))
        .collect();
    let look: LookFields = serde_json::from_value(Value::Object(look))?;
    let geometry = GeometrySpec::from_fields(&kind, Value::Object(std::mem::take(m)))?;
    Ok((name, role, look.into(), geometry))
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
