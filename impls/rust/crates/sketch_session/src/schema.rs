//! JSON Schemas of the interchange form, composed from the command catalog.
//!
//! One `oneOf` branch per fixed operation and per `create` / `constrain` kind;
//! each branch is a closed object (`additionalProperties: false`), matching
//! the strict reader in [`crate::json`].

use schemars::generate::SchemaSettings;
use schemars::{json_schema, JsonSchema, Schema};
use serde_json::{json, Map, Value};

use crate::catalog::{ConstraintSpec, GeometrySpec};
use crate::json::{DeleteFields, RenameFields, SetRoleFields, RESERVED};
use crate::wire::Role;

/// Schema of `T` with all subschemas inlined (no `$ref`).
pub(crate) fn inline_schema<T: JsonSchema>() -> Schema {
    SchemaSettings::draft2020_12()
        .with(|s| s.inline_subschemas = true)
        .into_generator()
        .into_root_schema_for::<T>()
}

/// `(kind tag, field)` for every kind parameter that uses a reserved name
/// ([`crate::json::RESERVED`]); empty in a consistent catalog.
pub(crate) fn reserved_clashes() -> Vec<(&'static str, String)> {
    GeometrySpec::kind_schemas()
        .into_iter()
        .chain(ConstraintSpec::kind_schemas())
        .flat_map(|(tag, s)| parts(s).0.into_iter().map(move |(field, _)| (tag, field)))
        .filter(|(_, field)| RESERVED.contains(&field.as_str()))
        .collect()
}

/// Schema of every command (protocol).
pub fn command_schema() -> Schema {
    let mut branches = creates_and_constraints(false);
    branches.push(fixed("delete", inline_schema::<DeleteFields>()));
    branches.push(fixed("rename", inline_schema::<RenameFields>()));
    branches.push(fixed("set_role", inline_schema::<SetRoleFields>()));
    root("Command", "One command of a drawing session.", branches)
}

/// Schema of the declarative subset (files): creates and constraints with names.
pub fn declarative_schema() -> Schema {
    root(
        "DeclCommand",
        "One command of the declarative subset; `name` is required.",
        creates_and_constraints(true),
    )
}

fn creates_and_constraints(name_required: bool) -> Vec<Value> {
    let role = inline_schema::<Role>().to_value();
    let mut out = Vec::new();
    for (tag, params) in GeometrySpec::kind_schemas() {
        out.push(branch("create", tag, Some(&role), params, name_required));
    }
    for (tag, params) in ConstraintSpec::kind_schemas() {
        out.push(branch("constrain", tag, None, params, name_required));
    }
    out
}

fn name_schema() -> Value {
    json!({ "type": "string", "pattern": "^[A-Za-z_][A-Za-z0-9_]*$" })
}

fn branch(
    op: &str,
    kind: &str,
    role: Option<&Value>,
    params: Schema,
    name_required: bool,
) -> Value {
    let (mut props, mut required) = parts(params);
    let mut head = Map::new();
    head.insert("op".into(), json!({ "const": op }));
    head.insert("kind".into(), json!({ "const": kind }));
    head.insert("name".into(), name_schema());
    if let Some(role) = role {
        head.insert("role".into(), strip_meta(role.clone()));
    }
    head.append(&mut props);
    let mut req = vec![json!("op"), json!("kind")];
    if name_required {
        req.push(json!("name"));
    }
    req.append(&mut required);
    json!({
        "title": format!("{op} {kind}"),
        "type": "object",
        "properties": head,
        "required": req,
        "additionalProperties": false
    })
}

fn fixed(op: &str, fields: Schema) -> Value {
    let (mut props, mut required) = parts(fields);
    let mut head = Map::new();
    head.insert("op".into(), json!({ "const": op }));
    head.append(&mut props);
    let mut req = vec![json!("op")];
    req.append(&mut required);
    json!({
        "title": op,
        "type": "object",
        "properties": head,
        "required": req,
        "additionalProperties": false
    })
}

/// Properties and required list of an object schema.
fn parts(s: Schema) -> (Map<String, Value>, Vec<Value>) {
    let mut v = s.to_value();
    let props = match v.get_mut("properties").map(Value::take) {
        Some(Value::Object(p)) => p,
        _ => Map::new(),
    };
    let required = match v.get_mut("required").map(Value::take) {
        Some(Value::Array(r)) => r,
        _ => Vec::new(),
    };
    (props, required)
}

fn strip_meta(mut v: Value) -> Value {
    if let Value::Object(m) = &mut v {
        m.remove("$schema");
        m.remove("title");
    }
    v
}

fn root(title: &str, description: &str, branches: Vec<Value>) -> Schema {
    json_schema!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": title,
        "description": description,
        "oneOf": branches
    })
}
