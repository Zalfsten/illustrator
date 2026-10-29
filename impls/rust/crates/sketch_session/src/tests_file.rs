use super::*;
use crate::command::Command;
use constraint_runtime::{
    Address, Congruence, CurveParam, GeometryRole, NameError, Point2, Segment,
};

/// Session built from interchange commands, one per entry.
fn session(commands: &[&str]) -> Session {
    let mut s = Session::new(Document::new());
    for c in commands {
        s.apply(Command::from_json(c).unwrap()).unwrap();
    }
    s
}

/// A version 1 file around `commands` (each an interchange object).
fn file(commands: &[&str]) -> Vec<u8> {
    format!(
        r#"{{"format": "illustrator-sketch", "version": 1, "commands": [{}]}}"#,
        commands.join(",")
    )
    .into_bytes()
}

fn load_err(bytes: &[u8]) -> LoadError {
    match Session::load(bytes) {
        Ok(_) => panic!("loaded"),
        Err(e) => e,
    }
}

/// Index and error of a failing command.
fn command_err(bytes: &[u8]) -> (usize, CommandLoadError) {
    match load_err(bytes) {
        LoadError::Command { index, error } => (index, error),
        other => panic!("{other:?}"),
    }
}

fn bits(s: &Session) -> Vec<u64> {
    s.document().vector().iter().map(|v| v.to_bits()).collect()
}

const SEG1: &str =
    r#"{"op": "create", "kind": "segment", "name": "seg1", "start": [0, 0], "end": [1, 0]}"#;
const SEG2: &str =
    r#"{"op": "create", "kind": "segment", "name": "seg2", "start": [1, 0], "end": [1, 1]}"#;
const JOIN1: &str =
    r#"{"op": "constrain", "kind": "join", "name": "join1", "a": "seg1.end", "b": "seg2.start"}"#;

// --- Saving ---

#[test]
fn saved_bytes_of_two_joined_segments() {
    let s = session(&[SEG1, SEG2, JOIN1]);
    let expected = r#"{
  "format": "illustrator-sketch",
  "version": 1,
  "commands": [
    {"op":"create","kind":"segment","name":"seg1","role":"result","start":[0.0,0.0],"end":[1.0,0.0]},
    {"op":"create","kind":"segment","name":"seg2","role":"result","start":[1.0,0.0],"end":[1.0,1.0]},
    {"op":"constrain","kind":"join","name":"join1","a":"seg1.end","b":"seg2.start"}
  ]
}
"#;
    assert_eq!(String::from_utf8(s.save().unwrap()).unwrap(), expected);
}

#[test]
fn saved_bytes_of_an_empty_drawing() {
    let s = Session::new(Document::new());
    let expected =
        "{\n  \"format\": \"illustrator-sketch\",\n  \"version\": 1,\n  \"commands\": []\n}\n";
    assert_eq!(String::from_utf8(s.save().unwrap()).unwrap(), expected);
}

#[test]
fn one_command_per_line() {
    let text = String::from_utf8(session(&[SEG1, SEG2, JOIN1]).save().unwrap()).unwrap();
    let lines: Vec<&str> = text.lines().filter(|l| l.contains("\"op\"")).collect();
    assert_eq!(lines.len(), 3);
    for line in lines {
        let one = line.trim().trim_end_matches(',');
        assert!(crate::command::DeclCommand::from_json(one).is_ok(), "{one}");
    }
}

#[test]
fn unaddressable_drawing_cannot_be_saved() {
    let mut doc = Document::new();
    let p = Point2::new;
    let a = doc.add_geometry::<Segment>(GeometryRole::Result, (p(0.0, 0.0), p(2.0, 0.0)));
    let b = doc.add_geometry::<Segment>(GeometryRole::Result, (p(1.0, 0.0), p(1.0, 1.0)));
    doc.add_constraint(Congruence {
        a: doc.on_curve_expr(a, CurveParam::Uniform(0.5)).unwrap(),
        b: doc.start_expr(b).unwrap(),
    });
    match Session::new(doc).save() {
        Err(SaveError::Export(ExportError::NotAddressable { constraint })) => {
            assert_eq!(constraint, "join1")
        }
        other => panic!("{other:?}"),
    }
}

// --- Envelope, format and version ---

#[test]
fn minimal_file() {
    let (s, report) =
        Session::load(br#"{"format": "illustrator-sketch", "version": 1, "commands": []}"#)
            .unwrap();
    assert!(s.document().geometries().is_empty());
    assert_eq!(report, LoadReport::default());
}

#[test]
fn unknown_envelope_field_is_rejected() {
    let bytes = br#"{"format": "illustrator-sketch", "version": 1, "commands": [], "author": "x"}"#;
    match load_err(bytes) {
        LoadError::Envelope(e) => assert!(e.contains("author"), "{e}"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn missing_commands_is_rejected() {
    let bytes = br#"{"format": "illustrator-sketch", "version": 1}"#;
    assert!(matches!(load_err(bytes), LoadError::Envelope(_)));
}

#[test]
fn compressed_or_binary_bytes() {
    let zip = b"PK\x03\x04\x14\x00\x00\x00\x08\x00";
    assert!(matches!(load_err(zip), LoadError::NotASketchFile(_)));
    assert!(matches!(
        load_err(b"\xff\xfe"),
        LoadError::NotASketchFile(_)
    ));
}

#[test]
fn syntax_error_names_line_and_column() {
    let bytes = b"{\n  \"format\": \"illustrator-sketch\",\n  \"version\": 1,\n  \"commands\": [\n";
    match load_err(bytes) {
        LoadError::NotASketchFile(e) => assert!(e.contains("line"), "{e}"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn other_json_is_an_unknown_format() {
    assert!(matches!(load_err(b"[]"), LoadError::UnknownFormat));
    assert!(matches!(
        load_err(br#"{"format": "svg", "version": 1, "commands": []}"#),
        LoadError::UnknownFormat
    ));
    assert!(matches!(
        load_err(br#"{"version": 1, "commands": []}"#),
        LoadError::UnknownFormat
    ));
}

#[test]
fn newer_version() {
    match load_err(br#"{"format": "illustrator-sketch", "version": 2, "commands": []}"#) {
        LoadError::UnsupportedVersion(v) => assert_eq!(v, "2"),
        other => panic!("{other:?}"),
    }
    assert!(matches!(
        load_err(br#"{"format": "illustrator-sketch", "version": "1", "commands": []}"#),
        LoadError::UnsupportedVersion(_)
    ));
}

#[test]
fn other_layout_loads() {
    let standard = session(&[SEG1, SEG2, JOIN1]).save().unwrap();
    let value: serde_json::Value = serde_json::from_slice(&standard).unwrap();
    for layout in [
        serde_json::to_vec(&value).unwrap(),
        serde_json::to_vec_pretty(&value).unwrap(),
    ] {
        let (s, _) = Session::load(&layout).unwrap();
        assert_eq!(s.save().unwrap(), standard);
    }
}

// --- Load mode and load errors ---

#[test]
fn values_within_tolerance_are_kept_exactly() {
    let original = session(&[SEG1, SEG2, JOIN1]);
    let (s, report) = Session::load(&original.save().unwrap()).unwrap();
    assert_eq!(bits(&s), bits(&original));
    assert_eq!(report, LoadReport::default());
    assert!(!s.can_undo());
}

#[test]
fn forward_reference_fails() {
    let (index, error) = command_err(&file(&[SEG1, JOIN1, SEG2]));
    assert_eq!(index, 1);
    assert!(
        matches!(error, CommandLoadError::Execute(CommandError::Address(_))),
        "{error:?}"
    );
}

#[test]
fn taken_name() {
    let (index, error) = command_err(&file(&[SEG1, SEG1]));
    assert_eq!(index, 1);
    assert!(
        matches!(
            error,
            CommandLoadError::Execute(CommandError::Name(NameError::Taken))
        ),
        "{error:?}"
    );
}

#[test]
fn join_within_one_congruence_class() {
    let seg = |n: &str, x: f64| {
        format!(
            r#"{{"op": "create", "kind": "segment", "name": "{n}", "start": [{x}, 0], "end": [{x}, 1]}}"#
        )
    };
    let join = |n: &str, a: &str, b: &str| {
        format!(r#"{{"op": "constrain", "kind": "join", "name": "{n}", "a": "{a}", "b": "{b}"}}"#)
    };
    let commands = [
        seg("a", 0.0),
        seg("b", 0.0),
        seg("c", 0.0),
        join("j1", "a.end", "b.start"),
        join("j2", "b.start", "c.start"),
        join("j3", "c.start", "a.end"),
    ];
    let refs: Vec<&str> = commands.iter().map(String::as_str).collect();
    let (index, error) = command_err(&file(&refs));
    assert_eq!(index, 5);
    assert!(
        matches!(
            error,
            CommandLoadError::Execute(CommandError::AlreadyJoined { .. })
        ),
        "{error:?}"
    );
}

#[test]
fn operation_outside_the_subset() {
    let rename = r#"{"op": "rename", "name": "seg1", "new_name": "x"}"#;
    let (index, error) = command_err(&file(&[SEG1, SEG2, JOIN1, rename]));
    assert_eq!(index, 3);
    assert!(matches!(error, CommandLoadError::Read(_)), "{error:?}");
    assert!(error.to_string().contains("declarative"), "{error}");
}

#[test]
fn load_error_names_the_position() {
    let e = load_err(&file(&[SEG1, SEG1]));
    assert!(e.to_string().starts_with("command 1:"), "{e}");
}

// --- Round trip and load report ---

#[test]
fn save_load_save_is_byte_identical() {
    let s = session(&[
        SEG1,
        r#"{"op": "create", "kind": "arc", "name": "fillet", "center": [1, 1], "radius": 1, "start_heading": -1.5707963267948966, "sweep": 1.5707963267948966}"#,
        r#"{"op": "create", "kind": "segment", "name": "helper", "role": "construction", "start": [0, 0], "end": [2, 1]}"#,
        r#"{"op": "constrain", "kind": "join", "name": "j", "a": "seg1.end", "b": "fillet.start"}"#,
        r#"{"op": "constrain", "kind": "parallel", "name": "tan", "a": "seg1.direction", "b": "fillet.tangent(0)"}"#,
        r#"{"op": "constrain", "kind": "fix", "name": "anchor", "point": "seg1.start", "target": [0, 0]}"#,
        r#"{"op": "constrain", "kind": "equal_length", "name": "eq", "a": "seg1.length", "b": "fillet.radius"}"#,
        r#"{"op": "constrain", "kind": "perpendicular", "name": "orth", "a": "seg1.direction", "b": "fillet.normal(0)"}"#,
    ]);
    let first = s.save().unwrap();
    let (loaded, report) = Session::load(&first).unwrap();
    assert_eq!(report, LoadReport::default());
    assert_eq!(bits(&loaded), bits(&s));
    assert_eq!(loaded.save().unwrap(), first);
}

#[test]
fn imprecise_file_is_adjusted() {
    let bytes = file(&[
        r#"{"op": "create", "kind": "segment", "name": "a", "start": [0, 0], "end": [2, 0]}"#,
        r#"{"op": "create", "kind": "segment", "name": "b", "start": [2, 0], "end": [2.1, 3]}"#,
        r#"{"op": "constrain", "kind": "perpendicular", "name": "orth1", "a": "a.direction", "b": "b.direction"}"#,
    ]);
    let (s, report) = Session::load(&bytes).unwrap();
    assert!(report.conflicts.is_empty());
    assert_eq!(report.adjusted.len(), 1);
    assert_eq!(report.adjusted[0].geometries, ["a", "b"]);
    assert!(report.adjusted[0].displacement > 0.0);
    let doc = s.document();
    let dir = |addr: &str| match doc.resolve_address(addr).unwrap() {
        Address::Vector(e) => e.evaluate(doc, doc.vector()),
        other => panic!("{other:?}"),
    };
    let (u, v) = (dir("a.direction"), dir("b.direction"));
    assert!((u.x * v.x + u.y * v.y).abs() < 1e-5);
}

#[test]
fn contradicting_file_is_held() {
    let bytes = file(&[
        SEG1,
        r#"{"op": "constrain", "kind": "fix", "name": "fix1", "point": "seg1.start", "target": [0, 0]}"#,
        r#"{"op": "constrain", "kind": "fix", "name": "fix2", "point": "seg1.start", "target": [0, 1]}"#,
    ]);
    let (s, report) = Session::load(&bytes).unwrap();
    assert!(report.adjusted.is_empty());
    assert_eq!(report.conflicts.len(), 1);
    assert_eq!(report.conflicts[0].geometries, ["seg1"]);
    let values: Vec<f64> = s.document().vector().iter().copied().collect();
    assert_eq!(values, [0.0, 0.0, 1.0, 0.0]);
}
