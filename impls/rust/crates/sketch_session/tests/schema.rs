//! Generated command schemas against the checked-in files, and spec examples
//! validated against them.
//!
//! `UPDATE_SCHEMA=1 cargo test -p sketch_session --test schema` rewrites the
//! files after an intended change of the interchange form.

use std::path::PathBuf;

use serde_json::Value;
use sketch_session::{command_schema, declarative_schema, AttrSpec, ConstraintSpec, GeometrySpec};

fn check_file(file: &str, schema: &Value) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("schema")
        .join(file);
    let text = serde_json::to_string_pretty(schema).unwrap() + "\n";
    if std::env::var_os("UPDATE_SCHEMA").is_some() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, &text).unwrap();
    }
    let stored = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{}: {e} (run with UPDATE_SCHEMA=1)", path.display()));
    assert!(
        stored == text,
        "{} differs from the generated schema; review the change and run with UPDATE_SCHEMA=1",
        path.display()
    );
}

fn branches(schema: &Value) -> Vec<(String, Option<String>, &Value)> {
    schema["oneOf"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| {
            let p = &b["properties"];
            let op = p["op"]["const"].as_str().unwrap().to_owned();
            let kind = p
                .get("kind")
                .map(|k| k["const"].as_str().unwrap().to_owned());
            (op, kind, b)
        })
        .collect()
}

#[test]
fn command_schema_is_checked_in() {
    check_file("commands.schema.json", command_schema().as_value());
}

#[test]
fn declarative_schema_is_checked_in() {
    check_file("declarative.schema.json", declarative_schema().as_value());
}

#[test]
fn command_schema_lists_every_operation_and_kind() {
    let schema = command_schema().to_value();
    let b = branches(&schema);
    for tag in GeometrySpec::TAGS {
        assert!(
            b.iter()
                .any(|(op, k, _)| op == "create" && k.as_deref() == Some(tag)),
            "{tag}"
        );
    }
    for tag in ConstraintSpec::TAGS {
        assert!(
            b.iter()
                .any(|(op, k, _)| op == "constrain" && k.as_deref() == Some(tag)),
            "{tag}"
        );
    }
    for tag in AttrSpec::TAGS {
        assert!(
            b.iter()
                .any(|(op, k, _)| op == "set" && k.as_deref() == Some(tag)),
            "{tag}"
        );
    }
    assert_eq!(AttrSpec::TAGS, ["label"]);
    for op in ["delete", "rename", "set_role", "set_document"] {
        assert!(b.iter().any(|(o, k, _)| o == op && k.is_none()), "{op}");
    }
    assert_eq!(
        b.len(),
        GeometrySpec::TAGS.len() + ConstraintSpec::TAGS.len() + AttrSpec::TAGS.len() + 4
    );
}

#[test]
fn declarative_schema_requires_names() {
    let schema = declarative_schema().to_value();
    let b = branches(&schema);
    assert_eq!(
        b.len(),
        GeometrySpec::TAGS.len() + ConstraintSpec::TAGS.len()
    );
    for (op, _, branch) in b {
        assert!(op == "create" || op == "constrain", "{op}");
        let required = branch["required"].as_array().unwrap();
        assert!(required.contains(&Value::from("name")), "{branch}");
    }
}

fn validator(schema: &Value) -> jsonschema::Validator {
    jsonschema::validator_for(schema).unwrap()
}

const VALID: &[&str] = &[
    r#"{"op": "rename", "name": "seg1", "new_name": "x_axis"}"#,
    r#"{"op": "constrain", "kind": "parallel", "a": "seg1.direction", "b": "seg2.direction"}"#,
    r#"{"op": "create", "kind": "segment", "start": [0, 0], "end": [3, 0]}"#,
    r#"{"op": "create", "kind": "arc", "name": "fillet", "role": "construction", "center": [3, 1], "radius": 1, "start_heading": -1.5707963267948966, "sweep": 1.5707963267948966}"#,
    r#"{"op": "constrain", "kind": "fix", "point": "fillet.start", "target": [3, 0]}"#,
    r#"{"op": "create", "kind": "arc", "center": [0, 0], "radius": 0, "start_heading": 0, "sweep": 1}"#,
    r#"{"op": "delete", "name": "seg1"}"#,
    r#"{"op": "set_role", "name": "seg1", "role": "result"}"#,
    r#"{"op": "create", "kind": "label", "origin": [1, 0], "heading": 0, "source": "$x$", "offset": [2, 3]}"#,
    r#"{"op": "create", "kind": "label", "name": "t", "origin": [0, 0], "heading": 1, "source": "a", "markup": "text", "halign": "center", "valign": "last", "size": 8}"#,
    r#"{"op": "set", "kind": "label", "name": "lbl1", "valign": "last"}"#,
    r#"{"op": "set", "kind": "label", "name": "lbl1", "size": null}"#,
    r#"{"op": "set_document", "scale": 5}"#,
    r#"{"op": "set_document", "preamble": "\\newcommand{\\R}{\\mathbb{R}}"}"#,
];

const INVALID: &[&str] = &[
    r#"{"op": "create", "kind": "segment", "start": [0, 0], "end": [1, 0], "rol": "construction"}"#,
    r#"{"op": "create", "kind": "segment", "start": [0, 0, 0], "end": [1, 0]}"#,
    r#"{"op": "parallel", "a": "seg1.direction", "b": "seg2.direction"}"#,
    r#"{"op": "create", "kind": "spline", "start": [0, 0], "end": [1, 0]}"#,
    r#"{"op": "create", "kind": "arc", "center": [0, 0], "radius": -1, "start_heading": 0, "sweep": 1}"#,
    r#"{"op": "create", "kind": "segment", "name": "seg.1", "start": [0, 0], "end": [1, 0]}"#,
    r#"{"op": "constrain", "kind": "parallel", "role": "result", "a": "x", "b": "y"}"#,
    r#"{"op": "create", "kind": "label", "origin": [0, 0], "heading": 0}"#,
    r#"{"op": "create", "kind": "label", "origin": [0, 0], "heading": 0, "source": "", "size": 0}"#,
    r#"{"op": "create", "kind": "label", "origin": [0, 0], "heading": 0, "source": "", "halign": "middle"}"#,
    r#"{"op": "set", "kind": "label", "name": "lbl1"}"#,
    r#"{"op": "set", "kind": "label", "source": "x"}"#,
    r#"{"op": "set", "kind": "segment", "name": "seg1", "start": [0, 0]}"#,
    r#"{"op": "set_document"}"#,
    r#"{"op": "set_document", "scale": 0}"#,
];

#[test]
fn spec_examples_validate_against_the_command_schema() {
    let v = validator(command_schema().as_value());
    for text in VALID {
        let instance: Value = serde_json::from_str(text).unwrap();
        assert!(v.is_valid(&instance), "{text}");
        assert!(sketch_session::Command::from_json(text).is_ok(), "{text}");
    }
    for text in INVALID {
        let instance: Value = serde_json::from_str(text).unwrap();
        assert!(!v.is_valid(&instance), "{text}");
    }
}

#[test]
fn declarative_schema_rejects_what_the_subset_excludes() {
    let v = validator(declarative_schema().as_value());
    let ok: Value = serde_json::from_str(
        r#"{"op": "create", "kind": "segment", "name": "seg1", "role": "result", "start": [0, 0], "end": [3, 0]}"#,
    )
    .unwrap();
    assert!(v.is_valid(&ok));
    for text in [
        r#"{"op": "constrain", "kind": "parallel", "a": "seg1.direction", "b": "seg2.direction"}"#,
        r#"{"op": "rename", "name": "seg1", "new_name": "x_axis"}"#,
        r#"{"op": "set", "kind": "label", "name": "lbl1", "source": "x"}"#,
        r#"{"op": "set_document", "scale": 5}"#,
    ] {
        let instance: Value = serde_json::from_str(text).unwrap();
        assert!(!v.is_valid(&instance), "{text}");
    }
}

/// Everything the writer produces is valid under the schemas.
#[test]
fn written_commands_validate() {
    use constraint_runtime::{Document, GeometryRole, Point2};
    use sketch_session::{ArcParams, Command, DeclCommand, PointPair, SegmentParams, Session};
    let mut s = Session::new(Document::new());
    let cmds = [
        Command::Create {
            name: None,
            role: GeometryRole::Construction,
            geometry: GeometrySpec::Segment(SegmentParams {
                start: Point2::new(0.0, 0.0),
                end: Point2::new(1.0, 0.0),
            }),
        },
        Command::Create {
            name: Some("fillet".into()),
            role: GeometryRole::Result,
            geometry: GeometrySpec::Arc(ArcParams {
                center: Point2::new(1.0, 1.0),
                radius: 1.0,
                start_heading: -std::f64::consts::FRAC_PI_2,
                sweep: 1.0,
            }),
        },
        Command::Constrain {
            name: None,
            constraint: ConstraintSpec::Congruence(PointPair {
                a: "seg1.end".into(),
                b: "fillet.start".into(),
            }),
        },
    ];
    let all = validator(command_schema().as_value());
    for c in cmds {
        let v: Value = serde_json::from_str(&c.to_json().unwrap()).unwrap();
        assert!(all.is_valid(&v), "{v}");
        s.apply(c).unwrap();
    }
    let decl = validator(declarative_schema().as_value());
    for c in s.declarative_commands().unwrap() {
        let v: Value = serde_json::from_str(&DeclCommand::to_json(&c).unwrap()).unwrap();
        assert!(decl.is_valid(&v), "{v}");
    }
}
