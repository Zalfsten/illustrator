use constraint_runtime::{GeometryRole, Point2};

use super::*;
use crate::kinds::{ArcParams, FixParams, LenPair, PointPair, SegmentParams, VecPair};

fn read(text: &str) -> Command {
    Command::from_json(text).unwrap_or_else(|e| panic!("{text}: {e}"))
}

fn read_err(text: &str) -> String {
    match Command::from_json(text) {
        Ok(c) => panic!("{text} was read as {c:?}"),
        Err(e) => e.to_string(),
    }
}

fn segment(start: Point2, end: Point2) -> GeometrySpec {
    GeometrySpec::Segment(SegmentParams { start, end })
}

// --- Commands have one interchange form ---

#[test]
fn rename_in_interchange_form() {
    assert_eq!(
        read(r#"{"op": "rename", "name": "seg1", "new_name": "x_axis"}"#),
        Command::Rename {
            name: "seg1".into(),
            new_name: "x_axis".into()
        }
    );
}

#[test]
fn unknown_operation_is_rejected() {
    let e = read_err(r#"{"op": "parallel", "a": "seg1.direction", "b": "seg2.direction"}"#);
    assert!(e.contains("parallel"), "{e}");
}

#[test]
fn missing_op_is_rejected() {
    read_err(r#"{"name": "seg1"}"#);
    read_err(r#"{"op": 3, "name": "seg1"}"#);
}

// --- Kinds are selected by a kind tag ---

#[test]
fn parallel_in_interchange_form() {
    assert_eq!(
        read(
            r#"{"op": "constrain", "kind": "parallel", "a": "seg1.direction", "b": "seg2.direction"}"#
        ),
        Command::Constrain {
            name: None,
            constraint: ConstraintSpec::VectorsParallel(VecPair {
                a: "seg1.direction".into(),
                b: "seg2.direction".into()
            })
        }
    );
}

#[test]
fn unknown_kind_is_rejected() {
    let e = read_err(r#"{"op": "create", "kind": "spline", "start": [0, 0], "end": [1, 0]}"#);
    assert!(e.contains("spline"), "{e}");
    read_err(r#"{"op": "constrain", "kind": "segment", "a": "x", "b": "y"}"#);
    read_err(r#"{"op": "create", "start": [0, 0], "end": [1, 0]}"#);
}

#[test]
fn kind_tags_are_independent_of_name_prefixes() {
    assert_eq!(GeometrySpec::TAGS, ["segment", "arc"]);
    assert_eq!(
        ConstraintSpec::TAGS,
        ["join", "parallel", "perpendicular", "fix", "equal_length"]
    );
}

// --- Interchange field shapes ---

#[test]
fn segment_in_interchange_form() {
    assert_eq!(
        read(r#"{"op": "create", "kind": "segment", "start": [0, 0], "end": [3, 0]}"#),
        Command::Create {
            name: None,
            role: GeometryRole::Result,
            geometry: segment(Point2::new(0.0, 0.0), Point2::new(3.0, 0.0)),
        }
    );
}

#[test]
fn arc_and_fix_in_interchange_form() {
    let h = -std::f64::consts::FRAC_PI_2;
    assert_eq!(
        read(
            r#"{"op": "create", "kind": "arc", "name": "fillet", "role": "construction",
                "center": [3, 1], "radius": 1, "start_heading": -1.5707963267948966,
                "sweep": 1.5707963267948966}"#
        ),
        Command::Create {
            name: Some("fillet".into()),
            role: GeometryRole::Construction,
            geometry: GeometrySpec::Arc(ArcParams {
                center: Point2::new(3.0, 1.0),
                radius: 1.0,
                start_heading: h,
                sweep: -h,
            }),
        }
    );
    assert_eq!(
        read(r#"{"op": "constrain", "kind": "fix", "point": "fillet.start", "target": [3, 0]}"#),
        Command::Constrain {
            name: None,
            constraint: ConstraintSpec::FixPoint(FixParams {
                point: "fillet.start".into(),
                target: Point2::new(3.0, 0.0),
            }),
        }
    );
}

#[test]
fn other_operations_in_interchange_form() {
    assert_eq!(
        read(r#"{"op": "delete", "name": "seg1"}"#),
        Command::Delete {
            name: "seg1".into()
        }
    );
    assert_eq!(
        read(r#"{"op": "set_role", "name": "seg1", "role": "construction"}"#),
        Command::SetRole {
            name: "seg1".into(),
            role: GeometryRole::Construction
        }
    );
    assert_eq!(
        read(
            r#"{"op": "constrain", "kind": "join", "name": "j", "a": "seg1.end", "b": "seg2.start"}"#
        ),
        Command::Constrain {
            name: Some("j".into()),
            constraint: ConstraintSpec::Congruence(PointPair {
                a: "seg1.end".into(),
                b: "seg2.start".into()
            }),
        }
    );
    assert_eq!(
        read(
            r#"{"op": "constrain", "kind": "equal_length", "a": "seg1.length", "b": "arc1.radius"}"#
        ),
        Command::Constrain {
            name: None,
            constraint: ConstraintSpec::EqualLength(LenPair {
                a: "seg1.length".into(),
                b: "arc1.radius".into()
            }),
        }
    );
}

// --- Interchange reading is strict ---

#[test]
fn misspelled_optional_field_is_rejected() {
    let e = read_err(
        r#"{"op": "create", "kind": "segment", "start": [0, 0], "end": [1, 0], "rol": "construction"}"#,
    );
    assert!(e.contains("rol"), "{e}");
}

#[test]
fn point_with_three_coordinates_is_rejected() {
    read_err(r#"{"op": "create", "kind": "segment", "start": [0, 0, 0], "end": [1, 0]}"#);
}

#[test]
fn unknown_fields_wrong_shapes_and_missing_fields_are_rejected() {
    for text in [
        r#"{"op": "delete", "name": "seg1", "force": true}"#,
        r#"{"op": "rename", "name": "seg1"}"#,
        r#"{"op": "set_role", "name": "seg1", "role": "draft"}"#,
        r#"{"op": "constrain", "kind": "parallel", "a": "seg1.direction"}"#,
        r#"{"op": "constrain", "kind": "parallel", "a": 1, "b": "seg2.direction"}"#,
        r#"{"op": "constrain", "kind": "parallel", "name": 7, "a": "x", "b": "y"}"#,
        r#"{"op": "constrain", "kind": "parallel", "role": "result", "a": "x", "b": "y"}"#,
        r#"{"op": "create", "kind": "segment", "name": null, "start": [0, 0], "end": [1, 0]}"#,
        r#"{"op": "create", "kind": "arc", "center": [0, 0], "radius": "1", "start_heading": 0, "sweep": 1}"#,
        r#"[{"op": "delete", "name": "seg1"}]"#,
    ] {
        read_err(text);
    }
}

// --- Declarative subset ---

#[test]
fn name_is_required_in_the_subset() {
    let text =
        r#"{"op": "constrain", "kind": "parallel", "a": "seg1.direction", "b": "seg2.direction"}"#;
    assert!(DeclCommand::from_json(text).is_err());
    assert!(Command::from_json(text).is_ok());
}

#[test]
fn rename_is_outside_the_subset() {
    for text in [
        r#"{"op": "rename", "name": "seg1", "new_name": "x_axis"}"#,
        r#"{"op": "delete", "name": "seg1"}"#,
        r#"{"op": "set_role", "name": "seg1", "role": "result"}"#,
    ] {
        let e = DeclCommand::from_json(text).unwrap_err().to_string();
        assert!(e.contains("declarative"), "{e}");
    }
}

#[test]
fn subset_list_is_checked_as_a_whole() {
    let ok = r#"[{"op": "create", "kind": "segment", "name": "a", "start": [0, 0], "end": [1, 0]},
                 {"op": "create", "kind": "segment", "name": "b", "start": [1, 0], "end": [1, 1]}]"#;
    assert_eq!(DeclCommand::list_from_json(ok).unwrap().len(), 2);
    let bad = r#"[{"op": "create", "kind": "segment", "name": "a", "start": [0, 0], "end": [1, 0]},
                  {"op": "rename", "name": "a", "new_name": "b"}]"#;
    assert!(DeclCommand::list_from_json(bad).is_err());
}

#[test]
fn decl_command_converts_to_command() {
    let d = DeclCommand::Create {
        name: "a".into(),
        role: GeometryRole::Construction,
        geometry: segment(Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)),
    };
    let c: Command = d.clone().into();
    assert_eq!(
        c,
        Command::Create {
            name: Some("a".into()),
            role: GeometryRole::Construction,
            geometry: segment(Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)),
        }
    );
    assert_eq!(Command::from_json(&d.to_json().unwrap()).unwrap(), c);
}

// --- Interchange round trip is lossless ---

fn every_command(v: f64) -> Vec<Command> {
    let p = Point2::new(v, -v);
    vec![
        Command::Create {
            name: None,
            role: GeometryRole::Result,
            geometry: segment(p, Point2::new(1.0 / 3.0, 0.1)),
        },
        Command::Create {
            name: Some("fillet".into()),
            role: GeometryRole::Construction,
            geometry: GeometrySpec::Arc(ArcParams {
                center: p,
                radius: v.abs(),
                start_heading: v,
                sweep: -v,
            }),
        },
        Command::Constrain {
            name: Some("j".into()),
            constraint: ConstraintSpec::Congruence(PointPair {
                a: "seg1.end".into(),
                b: "seg2.at(0.1)".into(),
            }),
        },
        Command::Constrain {
            name: None,
            constraint: ConstraintSpec::VectorsParallel(VecPair {
                a: "seg1.direction".into(),
                b: "arc1.tangent(0.5)".into(),
            }),
        },
        Command::Constrain {
            name: None,
            constraint: ConstraintSpec::VectorsOrthogonal(VecPair {
                a: "seg1.direction".into(),
                b: "seg2.direction".into(),
            }),
        },
        Command::Constrain {
            name: None,
            constraint: ConstraintSpec::FixPoint(FixParams {
                point: "seg1.start".into(),
                target: p,
            }),
        },
        Command::Constrain {
            name: None,
            constraint: ConstraintSpec::EqualLength(LenPair {
                a: "seg1.length".into(),
                b: "arc1.radius".into(),
            }),
        },
        Command::Delete {
            name: "seg1".into(),
        },
        Command::Rename {
            name: "seg1".into(),
            new_name: "x_axis".into(),
        },
        Command::SetRole {
            name: "seg1".into(),
            role: GeometryRole::Construction,
        },
    ]
}

/// Bit-exact comparison (`PartialEq` on f64 equates 0.0 and -0.0).
fn same(a: &Command, b: &Command) -> bool {
    a == b && format!("{a:?}") == format!("{b:?}")
}

#[test]
fn every_command_round_trips_exactly() {
    for v in [0.1, 1.0 / 3.0, 1e-300, -0.0, 0.0, 12345.678, f64::MAX] {
        for cmd in every_command(v) {
            let text = cmd.to_json().unwrap();
            let back = Command::from_json(&text).unwrap();
            assert!(same(&back, &cmd), "{text}\n{back:?}\n{cmd:?}");
        }
    }
}

#[test]
fn negative_zero_survives() {
    let cmd = &every_command(-0.0)[0];
    let Command::Create {
        geometry: GeometrySpec::Segment(p),
        ..
    } = Command::from_json(&cmd.to_json().unwrap()).unwrap()
    else {
        unreachable!()
    };
    assert!(p.start.x.is_sign_negative() && p.start.x == 0.0);
}

#[test]
fn non_finite_value_cannot_be_written() {
    for v in [f64::NAN, f64::INFINITY] {
        for cmd in &every_command(v)[..2] {
            assert!(cmd.to_json().is_err(), "{cmd:?}");
        }
        assert!(every_command(v)[5].to_json().is_err());
    }
}

#[test]
fn out_of_range_number_is_not_read() {
    read_err(r#"{"op": "create", "kind": "segment", "start": [1e999, 0], "end": [1, 0]}"#);
}

#[test]
fn written_form_is_flat_with_reserved_fields_first() {
    let cmd = Command::Create {
        name: Some("seg1".into()),
        role: GeometryRole::Result,
        geometry: segment(Point2::new(0.0, 0.0), Point2::new(3.0, 0.0)),
    };
    assert_eq!(
        cmd.to_json().unwrap(),
        r#"{"op":"create","kind":"segment","name":"seg1","role":"result","start":[0.0,0.0],"end":[3.0,0.0]}"#
    );
}

// --- Reserved names ---

#[test]
fn no_kind_uses_a_reserved_field_name() {
    assert_eq!(crate::schema::reserved_clashes(), vec![]);
}
