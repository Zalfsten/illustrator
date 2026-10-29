use super::*;
use crate::command::{DeclCommand, ExportError, Family, HistoryError, LoadReport};
use crate::kinds::{ArcParams, FixParams, LenPair, PointPair, SegmentParams, VecPair};
use constraint_runtime::{Address, AddressError, GeometryRole, Point2, Vec2};

fn segment(start: (f64, f64), end: (f64, f64)) -> Command {
    Command::Create {
        name: None,
        role: GeometryRole::Result,
        geometry: GeometrySpec::Segment(SegmentParams {
            start: Point2::new(start.0, start.1),
            end: Point2::new(end.0, end.1),
        }),
    }
}

fn constrain(constraint: ConstraintSpec) -> Command {
    Command::Constrain {
        name: None,
        constraint,
    }
}

fn vec_pair(a: &str, b: &str) -> VecPair {
    VecPair {
        a: a.into(),
        b: b.into(),
    }
}

fn parallel(a: &str, b: &str) -> Command {
    constrain(ConstraintSpec::VectorsParallel(vec_pair(a, b)))
}

fn perpendicular(a: &str, b: &str) -> Command {
    constrain(ConstraintSpec::VectorsOrthogonal(vec_pair(a, b)))
}

fn join(a: &str, b: &str) -> Command {
    constrain(ConstraintSpec::Congruence(PointPair {
        a: a.into(),
        b: b.into(),
    }))
}

fn fix(point: &str, target: (f64, f64), name: Option<&str>) -> Command {
    Command::Constrain {
        name: name.map(Into::into),
        constraint: ConstraintSpec::FixPoint(FixParams {
            point: point.into(),
            target: Point2::new(target.0, target.1),
        }),
    }
}

fn equal_length(a: &str, b: &str) -> Command {
    constrain(ConstraintSpec::EqualLength(LenPair {
        a: a.into(),
        b: b.into(),
    }))
}

fn arc(name: Option<&str>) -> Command {
    Command::Create {
        name: name.map(Into::into),
        role: GeometryRole::Result,
        geometry: GeometrySpec::Arc(ArcParams {
            center: Point2::new(0.0, 0.0),
            radius: 1.0,
            start_heading: 0.0,
            sweep: 1.0,
        }),
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
    if let Command::Create { name, .. } = &mut cmd {
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
        s.apply(fix(point, (x, 0.0), None)).unwrap();
    }
    s.apply(equal_length("seg1.length", "seg2.length")).unwrap();
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
    s.apply(fix("seg1.start", (0.5, 0.5), Some("anchor")))
        .unwrap();
    let out = s.apply(equal_length("seg1.length", "seg2.length")).unwrap();
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

// --- Drawing as declarative command list ---

fn named(cmd: Command, n: &str) -> Command {
    match cmd {
        Command::Create { role, geometry, .. } => Command::Create {
            name: Some(n.into()),
            role,
            geometry,
        },
        Command::Constrain { constraint, .. } => Command::Constrain {
            name: Some(n.into()),
            constraint,
        },
        other => other,
    }
}

fn construction(cmd: Command) -> Command {
    match cmd {
        Command::Create { name, geometry, .. } => Command::Create {
            name,
            role: GeometryRole::Construction,
            geometry,
        },
        other => other,
    }
}

/// `seg1`, construction `x_axis`, arc `fillet`, `join1`, `par1`.
fn export_scene() -> Session {
    let mut s = Session::new(Document::new());
    s.apply(segment((0.0, 0.0), (1.0, 0.0))).unwrap();
    s.apply(named(
        construction(segment((0.0, -1.0), (3.0, -1.0))),
        "x_axis",
    ))
    .unwrap();
    s.apply(Command::Create {
        name: Some("fillet".into()),
        role: GeometryRole::Result,
        geometry: GeometrySpec::Arc(ArcParams {
            center: Point2::new(1.0, 1.0),
            radius: 1.0,
            start_heading: -std::f64::consts::FRAC_PI_2,
            sweep: std::f64::consts::FRAC_PI_2,
        }),
    })
    .unwrap();
    s.apply(join("seg1.end", "fillet.start")).unwrap();
    s.apply(parallel("seg1.direction", "x_axis.direction"))
        .unwrap();
    s
}

/// Equal up to numbers, which may differ by `tol`.
fn close(a: &serde_json::Value, b: &serde_json::Value, tol: f64) -> bool {
    use serde_json::Value::*;
    match (a, b) {
        (Number(x), Number(y)) => (x.as_f64().unwrap() - y.as_f64().unwrap()).abs() <= tol,
        (Array(x), Array(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(x, y)| close(x, y, tol))
        }
        (Object(x), Object(y)) => {
            x.len() == y.len()
                && x.iter()
                    .all(|(k, v)| y.get(k).is_some_and(|w| close(v, w, tol)))
        }
        _ => a == b,
    }
}

fn list_value(list: &[DeclCommand]) -> serde_json::Value {
    serde_json::from_str(&DeclCommand::list_to_json(list).unwrap()).unwrap()
}

/// Execute `list` in a new session and export again.
fn replay(list: &[DeclCommand]) -> Vec<DeclCommand> {
    let mut t = Session::new(Document::new());
    for c in list {
        t.apply(c.clone().into()).unwrap();
    }
    t.declarative_commands().unwrap()
}

#[test]
fn export_lists_geometries_then_constraints_in_creation_order() {
    let list = export_scene().declarative_commands().unwrap();
    let head: Vec<(&str, &str, Option<GeometryRole>)> = list
        .iter()
        .map(|c| match c {
            DeclCommand::Create {
                name,
                role,
                geometry,
            } => (name.as_str(), geometry.tag(), Some(*role)),
            DeclCommand::Constrain { name, constraint } => (name.as_str(), constraint.tag(), None),
        })
        .collect();
    assert_eq!(
        head,
        [
            ("seg1", "segment", Some(GeometryRole::Result)),
            ("x_axis", "segment", Some(GeometryRole::Construction)),
            ("fillet", "arc", Some(GeometryRole::Result)),
            ("join1", "join", None),
            ("par1", "parallel", None),
        ]
    );
    assert_eq!(
        list[3],
        DeclCommand::Constrain {
            name: "join1".into(),
            constraint: ConstraintSpec::Congruence(PointPair {
                a: "seg1.end".into(),
                b: "fillet.start".into()
            })
        }
    );
    assert_eq!(
        list[4],
        DeclCommand::Constrain {
            name: "par1".into(),
            constraint: ConstraintSpec::VectorsParallel(vec_pair(
                "seg1.direction",
                "x_axis.direction"
            ))
        }
    );
}

#[test]
fn exported_list_re_executes_to_the_same_list() {
    let list = export_scene().declarative_commands().unwrap();
    let text = DeclCommand::list_to_json(&list).unwrap();
    let again = replay(&DeclCommand::list_from_json(&text).unwrap());
    assert!(
        close(&list_value(&list), &list_value(&again), 1e-9),
        "{text}\n{}",
        DeclCommand::list_to_json(&again).unwrap()
    );
}

#[test]
fn export_after_rename_and_delete() {
    let mut s = two_segments();
    s.apply(segment((3.0, 1.0), (4.0, 1.0))).unwrap();
    s.apply(join("seg2.end", "seg3.start")).unwrap();
    s.apply(Command::Rename {
        name: "seg2".into(),
        new_name: "base".into(),
    })
    .unwrap();
    s.apply(Command::Delete {
        name: "seg3".into(),
    })
    .unwrap();
    let names: Vec<String> = s
        .declarative_commands()
        .unwrap()
        .into_iter()
        .map(|c| match c {
            DeclCommand::Create { name, .. } | DeclCommand::Constrain { name, .. } => name,
        })
        .collect();
    assert_eq!(names, ["seg1", "base"]);
}

#[test]
fn constraint_without_address_cannot_be_exported() {
    use constraint_runtime::{CurveParam, FixPoint, Segment};
    let mut doc = Document::new();
    let id = doc.add_geometry::<Segment>(
        GeometryRole::Result,
        (Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)),
    );
    doc.add_constraint(FixPoint {
        point: doc.on_curve_expr(id, CurveParam::Uniform(0.5)).unwrap(),
        target: Point2::new(0.5, 0.0),
    });
    assert_eq!(
        Session::new(doc).declarative_commands(),
        Err(ExportError::NotAddressable {
            constraint: "fix1".into()
        })
    );
}

// --- Conflicted components ---

fn segment_in(doc: &mut Document, a: (f64, f64), b: (f64, f64)) {
    doc.add_geometry::<constraint_runtime::Segment>(
        GeometryRole::Result,
        (Point2::new(a.0, a.1), Point2::new(b.0, b.1)),
    );
}

fn point_in(doc: &Document, address: &str) -> constraint_runtime::PointExpr {
    match doc.resolve_address(address).unwrap() {
        Address::Point(e) => e,
        other => panic!("{address}: {other:?}"),
    }
}

/// A drawing no command could produce: `seg1` and `seg2` joined (`join1`) with
/// `seg1.start` fixed to two targets (`fix1` met, `fix2` not), so that
/// component is conflicted; `seg3` and `seg4` are free.
fn conflicted_document() -> Document {
    let mut doc = Document::new();
    segment_in(&mut doc, (0.0, 0.0), (1.0, 0.0));
    segment_in(&mut doc, (1.0, 0.0), (1.0, 1.0));
    segment_in(&mut doc, (5.0, 0.0), (6.0, 0.0));
    segment_in(&mut doc, (5.0, 2.0), (7.0, 3.0));
    let start = point_in(&doc, "seg1.start");
    doc.add_constraint(constraint_runtime::FixPoint {
        point: start,
        target: Point2::new(0.0, 0.0),
    });
    doc.add_constraint(constraint_runtime::FixPoint {
        point: start,
        target: Point2::new(0.0, 1.0),
    });
    doc.add_constraint(constraint_runtime::Congruence {
        a: point_in(&doc, "seg1.end"),
        b: point_in(&doc, "seg2.start"),
    });
    doc
}

/// Values of the geometries named `names`, as bits.
fn values_of(s: &Session, names: &[&str]) -> Vec<u64> {
    let doc = s.document();
    names
        .iter()
        .flat_map(|n| {
            let Some(ObjectId::Geometry(g)) = doc.lookup(n) else {
                panic!("{n} is not a geometry");
            };
            doc.vector()[doc.expect_geometry(g).param_block()]
                .iter()
                .map(|v| v.to_bits())
                .collect::<Vec<_>>()
        })
        .collect()
}

fn frozen(name: &str) -> CommandError {
    CommandError::Frozen { name: name.into() }
}

#[test]
fn conflicted_document_stays_frozen_at_start() {
    let doc = conflicted_document();
    let before = doc.vector().clone();
    let s = Session::new(doc);
    assert_eq!(s.document().vector().as_slice(), before.as_slice());
    assert_eq!(s.conflicts().len(), 1);
    assert!(!s.can_undo());
}

#[test]
fn constraint_inside_a_frozen_component_is_rejected() {
    let mut s = Session::new(conflicted_document());
    let before = content(&s);
    let e = s
        .apply(parallel("seg1.direction", "seg2.direction"))
        .unwrap_err();
    assert_eq!(e, frozen("seg1"));
    assert_eq!(content(&s), before);
    assert!(!s.can_undo());
}

#[test]
fn joining_to_a_frozen_component_is_rejected() {
    let mut s = Session::new(conflicted_document());
    let before = content(&s);
    let e = s.apply(join("seg3.start", "seg1.end")).unwrap_err();
    assert_eq!(e, frozen("seg1"));
    assert_eq!(content(&s), before);
}

#[test]
fn contradiction_elsewhere_does_not_block_commands() {
    let mut s = Session::new(conflicted_document());
    let frozen_values = values_of(&s, &["seg1", "seg2"]);
    s.apply(parallel("seg3.direction", "seg4.direction"))
        .unwrap();
    let (a, b) = (vec_at(&s, "seg3.direction"), vec_at(&s, "seg4.direction"));
    assert!((a.x * b.y - a.y * b.x).abs() < 1e-6);
    assert_eq!(values_of(&s, &["seg1", "seg2"]), frozen_values);
    assert_eq!(s.conflicts().len(), 1);
}

#[test]
fn new_contradiction_next_to_a_frozen_component_is_unsatisfiable() {
    let mut s = Session::new(conflicted_document());
    s.apply(fix("seg3.start", (5.0, 0.0), None)).unwrap();
    let before = content(&s);
    let e = s.apply(fix("seg3.start", (9.0, 9.0), None)).unwrap_err();
    assert!(matches!(e, CommandError::Unsatisfiable { .. }), "{e:?}");
    assert_eq!(content(&s), before);
}

#[test]
fn rename_inside_a_frozen_component_succeeds() {
    let mut s = Session::new(conflicted_document());
    let values = values_of(&s, &["seg1", "seg2"]);
    s.apply(Command::Rename {
        name: "seg1".into(),
        new_name: "base".into(),
    })
    .unwrap();
    assert_eq!(values_of(&s, &["base", "seg2"]), values);
    assert_eq!(s.conflicts()[0].geometries, ["base", "seg2"]);
}

#[test]
fn deleting_the_contradicting_constraint_unfreezes_the_component() {
    let mut s = Session::new(conflicted_document());
    s.apply(Command::Delete {
        name: "fix2".into(),
    })
    .unwrap();
    assert!(s.conflicts().is_empty());
    s.apply(parallel("seg1.direction", "seg2.direction"))
        .unwrap();
}

#[test]
fn deleting_the_satisfied_fix_solves_the_component() {
    let mut s = Session::new(conflicted_document());
    s.apply(Command::Delete {
        name: "fix1".into(),
    })
    .unwrap();
    assert!(s.conflicts().is_empty());
    let doc = s.document();
    let p = point_in(doc, "seg1.start").evaluate(doc, doc.vector());
    assert!((p - Point2::new(0.0, 1.0)).length() < 1e-6, "{p:?}");
}

#[test]
fn deleting_an_unrelated_constraint_keeps_the_component_frozen() {
    let mut s = Session::new(conflicted_document());
    let values = values_of(&s, &["seg1", "seg2"]);
    s.apply(Command::Delete {
        name: "join1".into(),
    })
    .unwrap();
    let conflicts = s.conflicts();
    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].geometries, ["seg1"]);
    assert_eq!(values_of(&s, &["seg1", "seg2"]), values);
}

#[test]
fn undo_restores_the_conflict() {
    let mut s = Session::new(conflicted_document());
    let before = content(&s);
    s.apply(Command::Delete {
        name: "fix1".into(),
    })
    .unwrap();
    assert!(s.conflicts().is_empty());
    s.undo().unwrap();
    assert_eq!(content(&s), before);
    assert_eq!(s.conflicts().len(), 1);
}

#[test]
fn drag_cannot_start_on_a_frozen_point() {
    let mut s = Session::new(conflicted_document());
    let before = content(&s);
    assert_eq!(s.drag_begin("seg2.end").unwrap_err(), frozen("seg2"));
    assert!(!s.is_dragging());
    assert_eq!(content(&s), before);
    assert!(!s.can_undo());

    let values = values_of(&s, &["seg1", "seg2"]);
    drag(&mut s, "seg3.end", (6.0, 1.0));
    assert!(s.can_undo());
    assert_eq!(values_of(&s, &["seg1", "seg2"]), values);
}

#[test]
fn solvable_and_contradicting_components_at_start() {
    let mut doc = conflicted_document();
    let end = point_in(&doc, "seg3.end");
    doc.add_constraint(constraint_runtime::FixPoint {
        point: end,
        target: Point2::new(6.0, 2.0),
    });
    let frozen_values: Vec<u64> = doc.vector()[..8].iter().map(|v| v.to_bits()).collect();
    let s = Session::new(doc);

    let conflicts = s.conflicts();
    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].geometries, ["seg1", "seg2"]);
    assert_eq!(values_of(&s, &["seg1", "seg2"]), frozen_values);
    let doc = s.document();
    let p = point_in(doc, "seg3.end").evaluate(doc, doc.vector());
    assert!((p - Point2::new(6.0, 2.0)).length() < 1e-6, "{p:?}");
    assert!(!s.can_undo());
}

#[test]
fn one_conflicted_component_is_listed() {
    let s = Session::new(conflicted_document());
    let conflicts = s.conflicts();
    assert_eq!(conflicts.len(), 1);
    let c = &conflicts[0];
    assert_eq!(c.geometries, ["seg1", "seg2"]);
    assert_eq!(c.constraints, ["fix1", "fix2", "join1"]);
    assert!(c.residual > constraint_runtime::COMMIT_TOL);
}

#[test]
fn consistent_drawing_lists_no_conflicts() {
    let mut s = two_segments();
    s.apply(parallel("seg1.direction", "seg2.direction"))
        .unwrap();
    assert!(s.conflicts().is_empty());
}

// --- Session start report ---

#[test]
fn drawing_within_tolerance_reports_nothing() {
    let mut doc = Document::new();
    segment_in(&mut doc, (0.0, 0.0), (1.0, 0.0));
    let start = point_in(&doc, "seg1.start");
    doc.add_constraint(constraint_runtime::FixPoint {
        point: start,
        target: Point2::new(0.0, 0.0),
    });
    let (_, report) = Session::start(doc);
    assert_eq!(report, LoadReport::default());
}

#[test]
fn moved_end_of_a_segment() {
    let mut doc = Document::new();
    segment_in(&mut doc, (0.0, 0.0), (1.0, 0.0));
    let end = point_in(&doc, "seg1.end");
    doc.add_constraint(constraint_runtime::FixPoint {
        point: end,
        target: Point2::new(1.0, 0.1),
    });
    let (s, report) = Session::start(doc);
    assert!(report.conflicts.is_empty());
    assert_eq!(report.adjusted.len(), 1);
    assert_eq!(report.adjusted[0].geometries, ["seg1"]);
    assert!((report.adjusted[0].displacement - 0.1).abs() < 1e-6);
    let doc = s.document();
    let start = point_in(doc, "seg1.start").evaluate(doc, doc.vector());
    assert_eq!(start, Point2::new(0.0, 0.0));
}

#[test]
fn contradicting_component_is_reported_as_conflict_not_adjusted() {
    let (s, report) = Session::start(conflicted_document());
    assert!(report.adjusted.is_empty());
    assert_eq!(report.conflicts, s.conflicts());
    assert_eq!(report.conflicts[0].geometries, ["seg1", "seg2"]);
}
