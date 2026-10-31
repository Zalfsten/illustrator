use constraint_runtime::{GeometryRole, Point2, Vec2};

use super::*;
use crate::catalog::AttrSpec;
use crate::kinds::{
    ArcParams, FixParams, LabelParams, LabelSet, LenPair, PointPair, SegmentParams, VecPair,
};
use crate::wire::{WireHAlign, WireMarkup, WireVAlign};

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
    assert_eq!(GeometrySpec::TAGS, ["segment", "arc", "label"]);
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
            look: Default::default(),
            geometry: segment(Point2::new(0.0, 0.0), Point2::new(3.0, 0.0)),
        }
    );
}

#[test]
fn label_in_interchange_form() {
    assert_eq!(
        read(
            r#"{"op": "create", "kind": "label", "origin": [1, 0], "heading": 0,
                "source": "$x$", "offset": [2, 3]}"#
        ),
        Command::Create {
            name: None,
            role: GeometryRole::Result,
            look: Default::default(),
            geometry: GeometrySpec::Label(LabelParams {
                origin: Point2::new(1.0, 0.0),
                heading: 0.0,
                source: "$x$".into(),
                markup: WireMarkup::Latex,
                halign: WireHAlign::Left,
                valign: WireVAlign::First,
                offset: Vec2::new(2.0, 3.0),
                size: None,
            }),
        }
    );
}

#[test]
fn label_without_source_is_rejected() {
    let e = read_err(r#"{"op": "create", "kind": "label", "origin": [0, 0], "heading": 0}"#);
    assert!(e.contains("source"), "{e}");
}

#[test]
fn label_with_bad_enum_unknown_field_or_null_size_is_rejected() {
    for text in [
        r#"{"op": "create", "kind": "label", "origin": [0, 0], "heading": 0, "source": "", "halign": "middle"}"#,
        r#"{"op": "create", "kind": "label", "origin": [0, 0], "heading": 0, "source": "", "colour": "red"}"#,
        r#"{"op": "create", "kind": "label", "origin": [0, 0], "heading": 0, "source": "", "size": null}"#,
    ] {
        read_err(text);
    }
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
            look: Default::default(),
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
        r#"{"op": "set", "kind": "label", "name": "lbl1", "source": "x"}"#,
        r#"{"op": "set_document", "scale": 5}"#,
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
        look: Default::default(),
        geometry: segment(Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)),
    };
    let c: Command = d.clone().into();
    assert_eq!(
        c,
        Command::Create {
            name: Some("a".into()),
            role: GeometryRole::Construction,
            look: Default::default(),
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
            look: Default::default(),
            geometry: segment(p, Point2::new(1.0 / 3.0, 0.1)),
        },
        Command::Create {
            name: Some("fillet".into()),
            role: GeometryRole::Construction,
            look: Default::default(),
            geometry: GeometrySpec::Arc(ArcParams {
                center: p,
                radius: v.abs(),
                start_heading: v,
                sweep: -v,
            }),
        },
        Command::Create {
            name: Some("lbl".into()),
            role: GeometryRole::Result,
            look: Default::default(),
            geometry: GeometrySpec::Label(LabelParams {
                origin: p,
                heading: v,
                source: "a\\b\n$\\vec v$ \"q\"".into(),
                markup: WireMarkup::Text,
                halign: WireHAlign::Center,
                valign: WireVAlign::Last,
                offset: Vec2::new(-v, v),
                size: Some(v.abs().max(1.0)),
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
        Command::Set {
            name: "lbl1".into(),
            attrs: AttrSpec::Label(LabelSet {
                source: Some("x\n\\y".into()),
                markup: Some(WireMarkup::Text),
                halign: Some(WireHAlign::Right),
                valign: Some(WireVAlign::Last),
                offset: Some(Vec2::new(v, -v)),
                size: Some(Some(v.abs().max(1.0))),
            }),
        },
        Command::Set {
            name: "lbl1".into(),
            attrs: AttrSpec::Label(LabelSet {
                size: Some(None),
                ..Default::default()
            }),
        },
        Command::SetDocument {
            scale: Some(v.abs().max(1.0)),
            font_size: None,
            preamble: Some("\\newcommand{\\R}{\\mathbb{R}}\n".into()),
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
        for cmd in &every_command(v)[..3] {
            assert!(cmd.to_json().is_err(), "{cmd:?}");
        }
        assert!(every_command(v)[6].to_json().is_err());
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
        look: Default::default(),
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

// --- set and set_document ---

#[test]
fn set_in_interchange_form() {
    assert_eq!(
        read(r#"{"op": "set", "kind": "label", "name": "lbl1", "valign": "last"}"#),
        Command::Set {
            name: "lbl1".into(),
            attrs: AttrSpec::Label(LabelSet {
                valign: Some(WireVAlign::Last),
                ..Default::default()
            }),
        }
    );
}

#[test]
fn set_needs_an_attribute_a_name_and_a_kind_with_attributes() {
    let e = read_err(r#"{"op": "set", "kind": "label", "name": "lbl1"}"#);
    assert!(e.contains("no attribute"), "{e}");
    read_err(r#"{"op": "set", "kind": "label", "source": "x"}"#);
    read_err(r#"{"op": "set", "name": "lbl1", "source": "x"}"#);
    read_err(r#"{"op": "set", "kind": "segment", "name": "seg1", "start": [0, 0]}"#);
    read_err(r#"{"op": "set", "kind": "label", "name": "lbl1", "origin": [0, 0]}"#);
}

#[test]
fn null_size_reads_as_inherit() {
    assert_eq!(
        read(r#"{"op": "set", "kind": "label", "name": "lbl1", "size": null}"#),
        Command::Set {
            name: "lbl1".into(),
            attrs: AttrSpec::Label(LabelSet {
                size: Some(None),
                ..Default::default()
            }),
        }
    );
}

#[test]
fn set_document_in_interchange_form() {
    assert_eq!(
        read(r#"{"op": "set_document", "scale": 5}"#),
        Command::SetDocument {
            scale: Some(5.0),
            font_size: None,
            preamble: None,
        }
    );
    let e = read_err(r#"{"op": "set_document"}"#);
    assert!(e.contains("no setting"), "{e}");
    read_err(r#"{"op": "set_document", "zoom": 2}"#);
}

#[test]
fn null_is_only_accepted_for_size() {
    for text in [
        r#"{"op": "set", "kind": "label", "name": "lbl1", "source": null}"#,
        r#"{"op": "set", "kind": "label", "name": "lbl1", "markup": null, "source": "x"}"#,
        r#"{"op": "set", "kind": "label", "name": "lbl1", "offset": null}"#,
        r#"{"op": "set_document", "preamble": null}"#,
        r#"{"op": "set_document", "scale": null}"#,
    ] {
        read_err(text);
    }
}

// --- Style fields and style commands ---

use constraint_runtime::{Arrow, Dash, Rgb, StylePatch, StyleProps, StyleRef};

const RED: Rgb = Rgb(0xc0, 0, 0);

#[test]
fn styled_segment_in_interchange_form() {
    let Command::Create { look, geometry, .. } = read(
        r#"{"op": "create", "kind": "segment", "start": [0, 0], "end": [3, 0], "style": "axis", "dash": "dashed"}"#,
    ) else {
        panic!("not a create");
    };
    assert_eq!(geometry, segment(Point2::new(0.0, 0.0), Point2::new(3.0, 0.0)));
    assert_eq!(
        look,
        StyleRef {
            style: "axis".into(),
            overrides: StyleProps {
                dash: Some(Dash::Dashed),
                ..StyleProps::default()
            },
        }
    );
}

#[test]
fn create_without_style_fields_refers_to_default() {
    let Command::Create { look, .. } =
        read(r#"{"op": "create", "kind": "arc", "center": [0, 0], "radius": 1, "start_heading": 0, "sweep": 1}"#)
    else {
        panic!("not a create");
    };
    assert_eq!(look, StyleRef::default());
}

#[test]
fn unknown_dash_pattern_is_rejected() {
    let e = read_err(
        r#"{"op": "create", "kind": "segment", "start": [0, 0], "end": [3, 0], "dash": "dashdotdot"}"#,
    );
    assert!(e.contains("dashdotdot"), "{e}");
}

#[test]
fn misspelled_colour_is_rejected() {
    let e = read_err(
        r##"{"op": "create", "kind": "segment", "start": [0, 0], "end": [3, 0], "colour": "#c00000"}"##,
    );
    assert!(e.contains("colour"), "{e}");
}

#[test]
fn null_style_fields_are_rejected_on_create() {
    read_err(r#"{"op": "create", "kind": "segment", "start": [0, 0], "end": [3, 0], "width": null}"#);
    read_err(r#"{"op": "create", "kind": "segment", "start": [0, 0], "end": [3, 0], "style": 3}"#);
}

#[test]
fn styled_create_round_trips_bit_for_bit() {
    let cmd = DeclCommand::Create {
        name: "x".into(),
        role: GeometryRole::Result,
        look: StyleRef {
            style: "axis".into(),
            overrides: StyleProps {
                width: Some(0.1 + 0.2),
                color: Some(RED),
                dash: Some(Dash::DashDot),
                arrow: Some(Arrow::Both),
            },
        },
        geometry: segment(Point2::new(0.0, 0.0), Point2::new(1.0 / 3.0, 0.0)),
    };
    let text = cmd.to_json().unwrap();
    assert_eq!(
        text,
        r##"{"op":"create","kind":"segment","name":"x","role":"result","start":[0.0,0.0],"end":[0.3333333333333333,0.0],"style":"axis","width":0.30000000000000004,"color":"#c00000","dash":"dashdot","arrow":"both"}"##
    );
    let back = DeclCommand::from_json(&text).unwrap();
    assert_eq!(back, cmd);
    assert_eq!(back.to_json().unwrap(), text);
}

#[test]
fn default_style_is_not_written() {
    let cmd = DeclCommand::Create {
        name: "x".into(),
        role: GeometryRole::Result,
        look: StyleRef::default(),
        geometry: segment(Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)),
    };
    let text = cmd.to_json().unwrap();
    assert!(!text.contains("style"), "{text}");
}

#[test]
fn style_definition_in_interchange_form() {
    assert_eq!(
        read(r#"{"op": "define_style", "style": "axis", "width": 0.8, "arrow": "end"}"#),
        Command::DefineStyle {
            style: "axis".into(),
            patch: StylePatch {
                width: Some(Some(0.8)),
                arrow: Some(Some(Arrow::End)),
                ..StylePatch::default()
            },
        }
    );
}

#[test]
fn null_removes_a_style_property() {
    assert_eq!(
        read(r#"{"op": "define_style", "style": "default", "width": null}"#),
        Command::DefineStyle {
            style: "default".into(),
            patch: StylePatch {
                width: Some(None),
                ..StylePatch::default()
            },
        }
    );
}

#[test]
fn style_commands_round_trip() {
    for text in [
        r##"{"op":"define_style","style":"axis","width":0.8,"color":"#c00000","dash":"dotted","arrow":null}"##,
        r#"{"op":"delete_style","style":"hidden"}"#,
        r#"{"op":"set_style","name":"seg1","style":"axis"}"#,
        r#"{"op":"set_style","name":"seg1","width":null,"color":null,"dash":null,"arrow":null}"#,
    ] {
        assert_eq!(read(text).to_json().unwrap(), text);
    }
}

#[test]
fn style_commands_need_a_property() {
    let e = read_err(r#"{"op": "define_style", "style": "axis"}"#);
    assert!(e.contains("no style property"), "{e}");
    read_err(r#"{"op": "set_style", "name": "seg1"}"#);
    read_err(r#"{"op": "define_style", "width": 1}"#);
    read_err(r#"{"op": "delete_style", "style": "a", "width": 1}"#);
    read_err(r#"{"op": "set_style", "name": "seg1", "colour": null}"#);
}

#[test]
fn short_colour_is_an_invalid_value() {
    let e = read_err(r##"{"op": "set_style", "name": "seg1", "color": "#c00"}"##);
    assert!(e.contains("invalid value for color"), "{e}");
}

#[test]
fn style_commands_are_outside_the_subset() {
    for text in [
        r#"{"op": "set_style", "name": "seg1", "style": "axis"}"#,
        r#"{"op": "define_style", "style": "axis", "width": 1}"#,
        r#"{"op": "delete_style", "style": "axis"}"#,
    ] {
        let e = DeclCommand::from_json(text).unwrap_err().to_string();
        assert!(e.contains("not in the declarative subset"), "{e}");
    }
}
