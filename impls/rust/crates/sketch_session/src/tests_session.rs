use super::*;
use crate::command::HistoryError;
use constraint_runtime::{AddressError, Point2, Vec2};

fn segment(start: (f64, f64), end: (f64, f64)) -> Command {
    Command::CreateSegment {
        start: Point2::new(start.0, start.1),
        end: Point2::new(end.0, end.1),
        role: GeometryRole::Result,
        name: None,
    }
}

fn parallel(a: &str, b: &str) -> Command {
    Command::Parallel {
        a: a.into(),
        b: b.into(),
        name: None,
    }
}

fn perpendicular(a: &str, b: &str) -> Command {
    Command::Perpendicular {
        a: a.into(),
        b: b.into(),
        name: None,
    }
}

fn join(a: &str, b: &str) -> Command {
    Command::Join {
        a: a.into(),
        b: b.into(),
        name: None,
    }
}

fn arc(name: Option<&str>) -> Command {
    Command::CreateArc {
        center: Point2::new(0.0, 0.0),
        radius: 1.0,
        start_heading: 0.0,
        sweep: 1.0,
        role: GeometryRole::Result,
        name: name.map(Into::into),
    }
}

/// Session with `seg1` (0,0)→(1,0) and `seg2` (2,0)→(3,1).
fn two_segments() -> Session {
    let mut s = Session::new(Document::new());
    s.apply(segment((0.0, 0.0), (1.0, 0.0))).unwrap();
    s.apply(segment((2.0, 0.0), (3.0, 1.0))).unwrap();
    s
}

fn vec_at(s: &Session, address: &str) -> Vec2 {
    let doc = s.document();
    match doc.resolve_address(address).unwrap() {
        Address::Vector(e) => e.evaluate(doc, doc.vector()),
        other => panic!("{address}: {other:?}"),
    }
}

/// Names of all objects (sorted) and the State values: the drawing's content.
fn content(s: &Session) -> (Vec<String>, Vec<f64>) {
    let doc = s.document();
    let mut names: Vec<String> = doc
        .geometries()
        .values()
        .map(|g| doc.geometry_name(g.id()).unwrap().to_owned())
        .chain(
            doc.constraints()
                .values()
                .map(|c| doc.constraint_name(c.id).unwrap().to_owned()),
        )
        .collect();
    names.sort();
    (names, doc.vector().as_slice().to_vec())
}

#[test]
fn create_segment_with_an_automatic_name() {
    let mut s = Session::new(Document::new());
    let out = s.apply(segment((0.0, 0.0), (1.0, 0.0))).unwrap();
    assert_eq!(out.created.as_deref(), Some("seg1"));
    assert!(out.removed.is_empty());
}

#[test]
fn create_with_an_explicit_name() {
    let mut s = Session::new(Document::new());
    let out = s.apply(arc(Some("fillet"))).unwrap();
    assert_eq!(out.created.as_deref(), Some("fillet"));
    assert!(matches!(
        s.document().lookup("fillet"),
        Some(ObjectId::Geometry(_))
    ));
}

#[test]
fn parallel_lines_by_direction() {
    let mut s = two_segments();
    let out = s
        .apply(parallel("seg1.direction", "seg2.direction"))
        .unwrap();
    assert_eq!(out.created.as_deref(), Some("par1"));
    let (a, b) = (vec_at(&s, "seg1.direction"), vec_at(&s, "seg2.direction"));
    assert!(a.cross(b).abs() < 1e-6, "{a:?} ∦ {b:?}");
}

#[test]
fn wrong_address_family_is_rejected() {
    let mut s = two_segments();
    assert_eq!(
        s.apply(parallel("seg1.start", "seg2.direction")),
        Err(CommandError::WrongFamily {
            address: "seg1.start".into(),
            expected: Family::Vector
        })
    );
}

#[test]
fn deleting_a_geometry_reports_cascaded_constraints() {
    let mut s = two_segments();
    s.apply(parallel("seg1.direction", "seg2.direction"))
        .unwrap();
    s.apply(join("seg1.end", "seg2.start")).unwrap();
    let out = s
        .apply(Command::Delete {
            name: "seg1".into(),
        })
        .unwrap();
    let mut removed = out.removed;
    removed.sort();
    assert_eq!(removed, ["join1", "par1", "seg1"]);
    assert_eq!(content(&s).0, ["seg2"]);
}

#[test]
fn rename_through_a_command() {
    let mut s = two_segments();
    s.apply(Command::Rename {
        name: "seg1".into(),
        new_name: "x_axis".into(),
    })
    .unwrap();
    assert!(s.document().lookup("x_axis").is_some());
    assert!(s.document().lookup("seg1").is_none());
}

#[test]
fn set_role_of_a_geometry() {
    let mut s = two_segments();
    s.apply(Command::SetRole {
        name: "seg1".into(),
        role: GeometryRole::Construction,
    })
    .unwrap();
    let Some(ObjectId::Geometry(id)) = s.document().lookup("seg1") else {
        panic!("seg1 missing");
    };
    assert_eq!(
        s.document().geometry(id).unwrap().role(),
        GeometryRole::Construction
    );
}

#[test]
fn failed_create_with_a_taken_name_changes_nothing() {
    let mut s = two_segments();
    let before = content(&s);
    let mut cmd = segment((5.0, 5.0), (6.0, 5.0));
    if let Command::CreateSegment { name, .. } = &mut cmd {
        *name = Some("seg1".into());
    }
    assert_eq!(s.apply(cmd), Err(CommandError::Name(NameError::Taken)));
    assert_eq!(content(&s), before);
}

#[test]
fn invalid_name_on_create_is_rejected() {
    let mut s = Session::new(Document::new());
    assert_eq!(
        s.apply(arc(Some("my.arc"))),
        Err(CommandError::Name(NameError::Invalid))
    );
    assert!(content(&s).0.is_empty());
}

/// Regression for #28: the contradicting constraint must not stay behind.
///
/// `seg1` is pinned and `seg2` keeps its length, so neither direction can
/// collapse to zero (a zero vector is both parallel and perpendicular).
#[test]
fn contradicting_constraint_is_rolled_back() {
    let mut s = two_segments();
    for (point, x) in [("seg1.start", 0.0), ("seg1.end", 1.0)] {
        s.apply(Command::Fix {
            point: point.into(),
            target: Point2::new(x, 0.0),
            name: None,
        })
        .unwrap();
    }
    s.apply(Command::EqualLength {
        a: "seg1.length".into(),
        b: "seg2.length".into(),
        name: None,
    })
    .unwrap();
    s.apply(parallel("seg1.direction", "seg2.direction"))
        .unwrap();
    let before = content(&s);
    let err = s
        .apply(perpendicular("seg1.direction", "seg2.direction"))
        .unwrap_err();
    assert!(
        matches!(err, CommandError::Unsatisfiable { residual } if residual > 0.0),
        "{err:?}"
    );
    assert_eq!(content(&s), before);
    assert!(s.document().lookup("orth1").is_none());
}

#[test]
fn unknown_name() {
    let mut s = two_segments();
    assert_eq!(
        s.apply(Command::Delete {
            name: "nope".into()
        }),
        Err(CommandError::UnknownName("nope".into()))
    );
}

#[test]
fn missing_capability() {
    let mut s = two_segments();
    s.apply(arc(None)).unwrap();
    assert_eq!(
        s.apply(parallel("arc1.direction", "seg1.direction")),
        Err(CommandError::Address(AddressError::MissingCapability {
            name: "arc1".into(),
            capability: "line"
        }))
    );
}

#[test]
fn set_role_on_a_constraint_is_the_wrong_object_type() {
    let mut s = two_segments();
    s.apply(parallel("seg1.direction", "seg2.direction"))
        .unwrap();
    assert_eq!(
        s.apply(Command::SetRole {
            name: "par1".into(),
            role: GeometryRole::Construction
        }),
        Err(CommandError::WrongObjectType {
            name: "par1".into(),
            expected: "geometry"
        })
    );
}

#[test]
fn fix_and_equal_length() {
    let mut s = two_segments();
    s.apply(Command::Fix {
        point: "seg1.start".into(),
        target: Point2::new(0.5, 0.5),
        name: Some("anchor".into()),
    })
    .unwrap();
    let out = s
        .apply(Command::EqualLength {
            a: "seg1.length".into(),
            b: "seg2.length".into(),
            name: None,
        })
        .unwrap();
    assert_eq!(out.created.as_deref(), Some("eqlen1"));
    let doc = s.document();
    let len = |a: &str| match doc.resolve_address(a).unwrap() {
        Address::Length(e) => e.evaluate(doc, doc.vector()).value(),
        _ => unreachable!(),
    };
    assert!((len("seg1.length") - len("seg2.length")).abs() < 1e-6);
    let Address::Point(start) = doc.resolve_address("seg1.start").unwrap() else {
        unreachable!()
    };
    let p = start.evaluate(doc, doc.vector());
    assert!((p.x - 0.5).abs() < 1e-6 && (p.y - 0.5).abs() < 1e-6);
}

fn drag(s: &mut Session, point: &str, to: (f64, f64)) {
    s.drag_begin(point).unwrap();
    let target = Point2::new(to.0, to.1);
    s.drag_to(target).unwrap();
    s.drag_end(target).unwrap();
}

#[test]
fn undo_and_redo_a_constraint() {
    let mut s = two_segments();
    let before = content(&s);
    s.apply(join("seg1.end", "seg2.start")).unwrap();
    let after = content(&s);
    assert_ne!(after, before);

    s.undo().unwrap();
    assert_eq!(content(&s), before);
    s.redo().unwrap();
    assert_eq!(content(&s), after);
}

#[test]
fn failed_command_creates_no_step() {
    let mut s = Session::new(Document::new());
    assert!(s
        .apply(Command::Delete {
            name: "nope".into()
        })
        .is_err());
    assert!(!s.can_undo());
}

#[test]
fn new_command_clears_redo() {
    let mut s = two_segments();
    s.undo().unwrap();
    assert!(s.can_redo());
    s.apply(segment((5.0, 5.0), (6.0, 5.0))).unwrap();
    assert!(!s.can_redo());
    assert_eq!(s.redo(), Err(HistoryError::NothingToRedo));
}

#[test]
fn nothing_to_undo() {
    let mut s = Session::new(Document::new());
    assert_eq!(s.undo(), Err(HistoryError::NothingToUndo));
    assert_eq!(s.redo(), Err(HistoryError::NothingToRedo));
    assert!(content(&s).0.is_empty());
}

/// Regression for #27: undoing a drag after a structural edit panicked.
#[test]
fn undo_after_a_structural_edit_following_a_drag() {
    let mut s = two_segments();
    let before_drag = content(&s);
    drag(&mut s, "seg2.end", (5.0, 5.0));
    let after_drag = content(&s);
    assert_ne!(after_drag, before_drag);
    s.apply(segment((7.0, 7.0), (8.0, 7.0))).unwrap();

    s.undo().unwrap();
    assert_eq!(content(&s), after_drag, "first undo removes the segment");
    s.undo().unwrap();
    assert_eq!(content(&s), before_drag, "second undo reverts the drag");
}

#[test]
fn drag_and_command_undo_in_order() {
    let mut s = two_segments();
    let before_join = content(&s);
    s.apply(join("seg1.end", "seg2.start")).unwrap();
    let after_join = content(&s);
    drag(&mut s, "seg2.end", (4.0, 3.0));

    s.undo().unwrap();
    assert_eq!(content(&s), after_join);
    s.undo().unwrap();
    assert_eq!(content(&s), before_join);
}

#[test]
fn cancelled_drag_creates_no_step() {
    let mut s = Session::new(Document::new());
    s.apply(segment((0.0, 0.0), (1.0, 0.0))).unwrap();
    let before = content(&s);
    s.drag_begin("seg1.end").unwrap();
    s.drag_to(Point2::new(3.0, 3.0)).unwrap();
    s.drag_cancel();
    assert!(!s.is_dragging());
    assert_eq!(content(&s), before);
    s.undo().unwrap();
    assert!(content(&s).0.is_empty(), "only the create was a step");
}

#[test]
fn commands_and_history_are_rejected_while_dragging() {
    let mut s = two_segments();
    s.drag_begin("seg1.end").unwrap();
    assert_eq!(
        s.apply(segment((5.0, 5.0), (6.0, 5.0))),
        Err(CommandError::Busy)
    );
    assert_eq!(s.undo(), Err(HistoryError::Busy));
    assert_eq!(s.redo(), Err(HistoryError::Busy));
    assert_eq!(s.drag_begin("seg2.end"), Err(CommandError::Busy));
    s.drag_cancel();
    assert!(s.apply(segment((5.0, 5.0), (6.0, 5.0))).is_ok());
}

#[test]
fn drag_begin_needs_a_point_address() {
    let mut s = two_segments();
    assert_eq!(
        s.drag_begin("seg1.direction"),
        Err(CommandError::WrongFamily {
            address: "seg1.direction".into(),
            expected: Family::Point
        })
    );
    assert!(!s.is_dragging());
}

fn already_joined(a: &str, b: &str) -> CommandError {
    CommandError::AlreadyJoined {
        a: a.into(),
        b: b.into(),
    }
}

#[test]
fn joining_a_point_with_itself_is_rejected() {
    let mut s = two_segments();
    let before = content(&s);
    assert_eq!(
        s.apply(join("seg1.end", "seg1.end")),
        Err(already_joined("seg1.end", "seg1.end"))
    );
    assert_eq!(content(&s), before);
    s.undo().unwrap();
    s.undo().unwrap();
    assert!(!s.can_undo(), "the rejected join is no undo step");
}

#[test]
fn joining_across_an_existing_chain_is_rejected() {
    let mut s = two_segments();
    s.apply(segment((1.0, 0.0), (1.0, 1.0))).unwrap();
    s.apply(join("seg1.end", "seg2.start")).unwrap();
    s.apply(join("seg2.start", "seg3.start")).unwrap();
    let before = content(&s);
    assert_eq!(
        s.apply(join("seg1.end", "seg3.start")),
        Err(already_joined("seg1.end", "seg3.start"))
    );
    assert_eq!(content(&s), before);
}

#[test]
fn coincident_unjoined_endpoints_can_be_joined() {
    let mut s = Session::new(Document::new());
    s.apply(segment((0.0, 0.0), (1.0, 0.0))).unwrap();
    s.apply(segment((1.0, 0.0), (2.0, 0.0))).unwrap();
    let outcome = s.apply(join("seg1.end", "seg2.start")).unwrap();
    assert_eq!(outcome.created.as_deref(), Some("join1"));
    let doc = s.document();
    let Address::Point(a) = doc.resolve_address("seg1.end").unwrap() else {
        panic!("point address");
    };
    let Address::Point(b) = doc.resolve_address("seg2.start").unwrap() else {
        panic!("point address");
    };
    assert!(doc.point_class(&a).contains(&b));
}
