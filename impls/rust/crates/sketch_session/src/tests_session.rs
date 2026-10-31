use super::*;
use crate::command::{DeclCommand, ExportError, Family, HistoryError, LoadReport};
use crate::kinds::{ArcParams, FixParams, LenPair, PointPair, SegmentParams, VecPair};
use constraint_runtime::{Address, AddressError, GeometryRole, Point2, Vec2};

fn segment(start: (f64, f64), end: (f64, f64)) -> Command {
    Command::Create {
        name: None,
        role: GeometryRole::Result,
        look: Default::default(),
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
        look: Default::default(),
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
        Command::Create {
            role, look, geometry, ..
        } => Command::Create {
            name: Some(n.into()),
            role,
            look,
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
            look: Default::default(),
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
        look: Default::default(),
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
                ..
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

// --- Labels and set ---

fn label_cmd(source: &str) -> Command {
    Command::Create {
        name: None,
        role: GeometryRole::Result,
        look: Default::default(),
        geometry: GeometrySpec::Label(crate::kinds::LabelParams {
            origin: Point2::new(1.0, 0.0),
            heading: 0.0,
            source: source.into(),
            markup: Default::default(),
            halign: Default::default(),
            valign: Default::default(),
            offset: Vec2::zero(),
            size: None,
        }),
    }
}

fn set_label(name: &str, set: crate::kinds::LabelSet) -> Command {
    Command::Set {
        name: name.into(),
        attrs: crate::catalog::AttrSpec::Label(set),
    }
}

fn label_attrs<'a>(s: &'a Session, name: &str) -> &'a constraint_runtime::LabelAttrs {
    let doc = s.document();
    let Some(ObjectId::Geometry(g)) = doc.lookup(name) else {
        panic!("{name} is not a geometry");
    };
    doc.label_attrs(g).unwrap()
}

#[test]
fn create_a_label_with_an_automatic_name() {
    let mut s = Session::new(Document::new());
    let out = s.apply(label_cmd("A")).unwrap();
    assert_eq!(out.created.as_deref(), Some("lbl1"));
    assert_eq!(label_attrs(&s, "lbl1").source, "A");
}

#[test]
fn set_changes_a_labels_source_only() {
    let mut s = Session::new(Document::new());
    s.apply(label_cmd("a")).unwrap();
    let values = values_of(&s, &["lbl1"]);
    let mut before = label_attrs(&s, "lbl1").clone();
    s.apply(set_label(
        "lbl1",
        crate::kinds::LabelSet {
            source: Some("$\\alpha$".into()),
            ..Default::default()
        },
    ))
    .unwrap();
    before.source = "$\\alpha$".into();
    assert_eq!(label_attrs(&s, "lbl1"), &before);
    assert_eq!(values_of(&s, &["lbl1"]), values);
}

#[test]
fn several_attributes_are_one_undo_step() {
    use crate::wire::WireHAlign;
    let mut s = Session::new(Document::new());
    s.apply(label_cmd("a")).unwrap();
    let before = label_attrs(&s, "lbl1").clone();
    s.apply(set_label(
        "lbl1",
        crate::kinds::LabelSet {
            halign: Some(WireHAlign::Center),
            offset: Some(Vec2::new(0.0, 3.0)),
            ..Default::default()
        },
    ))
    .unwrap();
    assert_eq!(label_attrs(&s, "lbl1").offset, Vec2::new(0.0, 3.0));
    s.undo().unwrap();
    assert_eq!(label_attrs(&s, "lbl1"), &before);
    s.redo().unwrap();
    assert_eq!(label_attrs(&s, "lbl1").halign, constraint_runtime::HAlign::Center);
}

#[test]
fn set_on_a_constraint_or_another_kind_is_the_wrong_object_type() {
    let mut s = two_segments();
    s.apply(parallel("seg1.direction", "seg2.direction")).unwrap();
    let source = || crate::kinds::LabelSet {
        source: Some("x".into()),
        ..Default::default()
    };
    let before = content(&s);
    for (name, expected) in [("par1", "geometry"), ("seg1", "label")] {
        let e = s.apply(set_label(name, source())).unwrap_err();
        assert_eq!(
            e,
            CommandError::WrongObjectType {
                name: name.into(),
                expected
            }
        );
    }
    assert_eq!(content(&s), before);
}

#[test]
fn null_size_restores_inheritance_and_zero_is_invalid() {
    let mut s = Session::new(Document::new());
    s.apply(label_cmd("a")).unwrap();
    let size = |v| crate::kinds::LabelSet {
        size: Some(v),
        ..Default::default()
    };
    s.apply(set_label("lbl1", size(Some(8.0)))).unwrap();
    assert_eq!(label_attrs(&s, "lbl1").size, Some(8.0));
    let e = s.apply(set_label("lbl1", size(Some(0.0)))).unwrap_err();
    assert!(matches!(e, CommandError::InvalidValue { field: "size", .. }), "{e:?}");
    assert_eq!(label_attrs(&s, "lbl1").size, Some(8.0));
    s.apply(set_label("lbl1", size(None))).unwrap();
    assert_eq!(label_attrs(&s, "lbl1").size, None);
}

#[test]
fn set_inside_a_frozen_component_succeeds_without_changing_values() {
    let mut doc = conflicted_document();
    let lbl = doc.add_geometry::<constraint_runtime::Label>(
        GeometryRole::Result,
        (Point2::new(1.0, 0.0), constraint_runtime::HeadingAngle::new(0.0)),
    );
    let origin = point_in(&doc, &format!("{}.origin", doc.geometry_name(lbl).unwrap()));
    doc.add_constraint(constraint_runtime::Congruence {
        a: origin,
        b: point_in(&doc, "seg1.end"),
    });
    let mut s = Session::new(doc);
    assert_eq!(s.conflicts().len(), 1);
    let values = s.document().vector().clone();
    s.apply(set_label(
        "lbl1",
        crate::kinds::LabelSet {
            source: Some("frozen".into()),
            ..Default::default()
        },
    ))
    .unwrap();
    assert_eq!(label_attrs(&s, "lbl1").source, "frozen");
    assert_eq!(s.document().vector().as_slice(), values.as_slice());
}

// --- Document settings ---

fn set_document(scale: Option<f64>, font_size: Option<f64>, preamble: Option<&str>) -> Command {
    Command::SetDocument {
        scale,
        font_size,
        preamble: preamble.map(Into::into),
    }
}

#[test]
fn fresh_session_has_default_settings() {
    let s = Session::new(Document::new());
    assert_eq!(s.settings(), &constraint_runtime::DocumentSettings::default());
    assert_eq!(s.settings().scale, 10.0);
    assert_eq!(s.settings().font_size, 10.0);
    assert_eq!(s.settings().preamble, "");
}

#[test]
fn set_document_changes_only_given_settings() {
    let mut s = Session::new(Document::new());
    s.apply(set_document(Some(5.0), None, None)).unwrap();
    assert_eq!(s.settings().scale, 5.0);
    assert_eq!(s.settings().font_size, 10.0);
    s.apply(set_document(None, Some(12.0), Some("\\newcommand{\\R}{\\mathbb{R}}")))
        .unwrap();
    assert_eq!(s.settings().scale, 5.0);
    assert_eq!(s.settings().font_size, 12.0);
    assert_eq!(s.settings().preamble, "\\newcommand{\\R}{\\mathbb{R}}");
}

#[test]
fn out_of_range_settings_are_rejected() {
    let mut s = Session::new(Document::new());
    for (cmd, field) in [
        (set_document(Some(-1.0), None, None), "scale"),
        (set_document(Some(0.0), None, None), "scale"),
        (set_document(None, Some(0.0), None), "font_size"),
        (set_document(None, Some(f64::NAN), None), "font_size"),
    ] {
        let e = s.apply(cmd).unwrap_err();
        assert!(
            matches!(e, CommandError::InvalidValue { field: f, .. } if f == field),
            "{e:?}"
        );
    }
    assert_eq!(s.settings(), &constraint_runtime::DocumentSettings::default());
    assert!(!s.can_undo());
}

#[test]
fn undo_and_redo_a_settings_change() {
    let mut s = Session::new(Document::new());
    s.apply(set_document(Some(20.0), None, None)).unwrap();
    s.undo().unwrap();
    assert_eq!(s.settings().scale, 10.0);
    s.redo().unwrap();
    assert_eq!(s.settings().scale, 20.0);
}

#[test]
fn exported_label_carries_its_attributes_after_set() {
    use crate::wire::WireMarkup;
    let mut s = Session::new(Document::new());
    s.apply(label_cmd("a")).unwrap();
    s.apply(set_label(
        "lbl1",
        crate::kinds::LabelSet {
            source: Some("b".into()),
            markup: Some(WireMarkup::Text),
            ..Default::default()
        },
    ))
    .unwrap();
    let list = s.declarative_commands().unwrap();
    let text = list[0].to_json().unwrap();
    let v: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(v["source"], "b");
    assert_eq!(v["markup"], "text");
    for field in ["halign", "valign", "offset", "origin", "heading"] {
        assert!(v.get(field).is_some(), "{field} missing in {text}");
    }
    assert!(v.get("size").is_none(), "inherited size written: {text}");
}

#[test]
fn exported_labels_re_execute_to_the_same_list() {
    let mut s = Session::new(Document::new());
    s.apply(segment((0.0, 0.0), (1.0, 0.0))).unwrap();
    s.apply(label_cmd("own size")).unwrap();
    s.apply(label_cmd("inherited")).unwrap();
    s.apply(set_label(
        "lbl1",
        crate::kinds::LabelSet {
            size: Some(Some(8.0)),
            ..Default::default()
        },
    ))
    .unwrap();
    s.apply(join("lbl1.origin", "seg1.end")).unwrap();
    s.apply(parallel("lbl2.direction", "seg1.direction")).unwrap();
    let list = s.declarative_commands().unwrap();
    let own: serde_json::Value = serde_json::from_str(&list[1].to_json().unwrap()).unwrap();
    assert_eq!(own["size"], 8.0);
    assert!(close(&list_value(&replay(&list)), &list_value(&list), 1e-9));
}

// --- Style commands ---

use constraint_runtime::{Arrow, Dash, Rgb, Style, StyleProps, StyleRef};

const RED: Rgb = Rgb(0xc0, 0, 0);

fn cmd(text: &str) -> Command {
    Command::from_json(text).unwrap_or_else(|e| panic!("{text}: {e}"))
}

fn apply(s: &mut Session, text: &str) -> Result<Outcome, CommandError> {
    s.apply(cmd(text))
}

/// Session with style `axis` (width 0.8, arrow end) and segment `x` using it.
fn axis_session() -> Session {
    let mut s = Session::new(Document::new());
    apply(&mut s, r#"{"op": "define_style", "style": "axis", "width": 0.8, "arrow": "end"}"#).unwrap();
    apply(
        &mut s,
        r#"{"op": "create", "kind": "segment", "name": "x", "start": [0, 0], "end": [1, 0], "style": "axis"}"#,
    )
    .unwrap();
    s
}

#[test]
fn define_style_creates_a_style() {
    let mut s = Session::new(Document::new());
    let before = s.document().vector().clone();
    apply(&mut s, r#"{"op": "define_style", "style": "vector", "width": 0.8, "arrow": "end"}"#).unwrap();
    assert_eq!(
        s.styles().get("vector"),
        Some(&StyleProps {
            width: Some(0.8),
            arrow: Some(Arrow::End),
            ..StyleProps::default()
        })
    );
    assert_eq!(s.document().vector(), &before);
}

#[test]
fn define_style_changes_one_property() {
    let mut s = axis_session();
    apply(&mut s, r#"{"op": "define_style", "style": "axis", "width": 1.2}"#).unwrap();
    let p = s.styles().get("axis").unwrap();
    assert_eq!((p.width, p.arrow), (Some(1.2), Some(Arrow::End)));
}

#[test]
fn define_style_removes_a_property() {
    let mut s = two_segments();
    apply(&mut s, r#"{"op": "define_style", "style": "default", "width": 0.6}"#).unwrap();
    assert_eq!(s.style_of("seg1").unwrap().1.width, 0.6);
    apply(&mut s, r#"{"op": "define_style", "style": "default", "width": null}"#).unwrap();
    assert_eq!(s.styles().get("default").unwrap().width, None);
    assert_eq!(s.style_of("seg1").unwrap().1.width, Style::BUILT_IN.width);
}

#[test]
fn invalid_style_name() {
    let mut s = Session::new(Document::new());
    let e = apply(&mut s, r#"{"op": "define_style", "style": "2nd", "width": 1}"#).unwrap_err();
    assert_eq!(e, CommandError::Name(NameError::Invalid));
    assert!(!s.can_undo());
}

#[test]
fn delete_an_unused_style() {
    let mut s = Session::new(Document::new());
    apply(&mut s, r#"{"op": "define_style", "style": "hidden", "dash": "dashed"}"#).unwrap();
    apply(&mut s, r#"{"op": "delete_style", "style": "hidden"}"#).unwrap();
    assert!(!s.styles().contains("hidden"));
}

#[test]
fn delete_a_used_style() {
    let mut s = axis_session();
    let e = apply(&mut s, r#"{"op": "delete_style", "style": "axis"}"#).unwrap_err();
    assert_eq!(
        e,
        CommandError::StyleInUse {
            style: "axis".into(),
            users: vec!["x".into()],
        }
    );
    assert!(e.to_string().starts_with("style in use"), "{e}");
    assert!(s.styles().contains("axis"));
}

#[test]
fn delete_default_or_an_unknown_style() {
    let mut s = Session::new(Document::new());
    let e = apply(&mut s, r#"{"op": "delete_style", "style": "default"}"#).unwrap_err();
    assert!(matches!(e, CommandError::StyleInUse { .. }), "{e:?}");
    let e = apply(&mut s, r#"{"op": "delete_style", "style": "nope"}"#).unwrap_err();
    assert_eq!(e, CommandError::UnknownStyle("nope".into()));
}

#[test]
fn set_style_assigns_a_style_and_keeps_overrides() {
    let mut s = Session::new(Document::new());
    apply(&mut s, r#"{"op": "define_style", "style": "axis", "width": 0.8}"#).unwrap();
    apply(
        &mut s,
        r##"{"op": "create", "kind": "segment", "start": [0, 0], "end": [1, 0], "color": "#c00000"}"##,
    )
    .unwrap();
    apply(&mut s, r#"{"op": "set_style", "name": "seg1", "style": "axis"}"#).unwrap();
    let (r, _) = s.style_of("seg1").unwrap();
    assert_eq!(r.style, "axis");
    assert_eq!(r.overrides.color, Some(RED));
}

#[test]
fn set_style_removes_an_override() {
    let mut s = Session::new(Document::new());
    apply(
        &mut s,
        r##"{"op": "create", "kind": "segment", "start": [0, 0], "end": [1, 0], "color": "#c00000", "dash": "dotted"}"##,
    )
    .unwrap();
    apply(&mut s, r#"{"op": "set_style", "name": "seg1", "color": null}"#).unwrap();
    let (r, _) = s.style_of("seg1").unwrap();
    assert_eq!(r.overrides.color, None);
    assert_eq!(r.overrides.dash, Some(Dash::Dotted));
}

#[test]
fn set_style_on_a_constraint() {
    let mut s = two_segments();
    s.apply(parallel("seg1.direction", "seg2.direction")).unwrap();
    let e = apply(&mut s, r#"{"op": "set_style", "name": "par1", "dash": "dashed"}"#).unwrap_err();
    assert!(matches!(e, CommandError::WrongObjectType { .. }), "{e:?}");
}

#[test]
fn unknown_style_in_set_style() {
    let mut s = two_segments();
    let before = s.style_of("seg1").unwrap();
    let e = apply(&mut s, r#"{"op": "set_style", "name": "seg1", "style": "nope"}"#).unwrap_err();
    assert_eq!(e, CommandError::UnknownStyle("nope".into()));
    assert!(e.to_string().starts_with("unknown style"), "{e}");
    assert_eq!(s.style_of("seg1").unwrap(), before);
}

#[test]
fn zero_width_is_an_invalid_value() {
    let mut s = axis_session();
    for text in [
        r#"{"op": "define_style", "style": "axis", "width": 0}"#,
        r#"{"op": "set_style", "name": "x", "width": -1}"#,
        r#"{"op": "create", "kind": "segment", "start": [0, 0], "end": [1, 0], "width": 0}"#,
    ] {
        let e = apply(&mut s, text).unwrap_err();
        assert!(matches!(e, CommandError::InvalidValue { field: "width", .. }), "{text}: {e:?}");
    }
    assert_eq!(s.styles().get("axis").unwrap().width, Some(0.8));
    assert_eq!(s.document().geometries().len(), 1);
}

#[test]
fn create_with_a_style_and_an_override() {
    let mut s = axis_session();
    apply(
        &mut s,
        r##"{"op": "create", "kind": "segment", "name": "y", "start": [0, 1], "end": [1, 1], "style": "axis", "color": "#c00000"}"##,
    )
    .unwrap();
    let (r, _) = s.style_of("y").unwrap();
    assert_eq!(r.style, "axis");
    assert_eq!(r.overrides.color, Some(RED));
}

#[test]
fn create_with_an_unknown_style() {
    let mut s = Session::new(Document::new());
    let e = apply(
        &mut s,
        r#"{"op": "create", "kind": "segment", "start": [0, 0], "end": [1, 0], "style": "nope"}"#,
    )
    .unwrap_err();
    assert_eq!(e, CommandError::UnknownStyle("nope".into()));
    assert_eq!(s.document().geometries().len(), 0);
    assert!(!s.can_undo());
}

#[test]
fn every_kind_carries_a_style() {
    let mut s = Session::new(Document::new());
    s.apply(segment((0.0, 0.0), (1.0, 0.0))).unwrap();
    s.apply(arc(None)).unwrap();
    apply(&mut s, r#"{"op": "create", "kind": "label", "origin": [0, 0], "heading": 0, "source": "a"}"#).unwrap();
    for name in ["seg1", "arc1", "lbl1"] {
        assert_eq!(s.style_of(name).unwrap().0, StyleRef::default(), "{name}");
    }
}

#[test]
fn style_commands_are_history_steps() {
    let mut s = axis_session();
    apply(&mut s, r#"{"op": "define_style", "style": "axis", "width": 1.2}"#).unwrap();
    s.undo().unwrap();
    assert_eq!(s.styles().get("axis").unwrap().width, Some(0.8));
    s.redo().unwrap();
    assert_eq!(s.styles().get("axis").unwrap().width, Some(1.2));

    apply(&mut s, r##"{"op": "set_style", "name": "x", "style": "default", "color": "#c00000"}"##).unwrap();
    s.undo().unwrap();
    assert_eq!(s.style_of("x").unwrap().0, StyleRef {
        style: "axis".into(),
        overrides: StyleProps::default(),
    });

    apply(&mut s, r#"{"op": "define_style", "style": "hidden", "dash": "dashed"}"#).unwrap();
    apply(&mut s, r#"{"op": "delete_style", "style": "hidden"}"#).unwrap();
    s.undo().unwrap();
    assert!(s.styles().contains("hidden"));
}

#[test]
fn resolved_style_of_a_geometry() {
    let mut s = Session::new(Document::new());
    apply(&mut s, r#"{"op": "define_style", "style": "axis", "width": 0.8}"#).unwrap();
    apply(
        &mut s,
        r##"{"op": "create", "kind": "segment", "name": "x", "start": [0, 0], "end": [1, 0], "style": "axis", "color": "#c00000"}"##,
    )
    .unwrap();
    let (r, resolved) = s.style_of("x").unwrap();
    assert_eq!(r.style, "axis");
    assert_eq!(
        r.overrides,
        StyleProps {
            color: Some(RED),
            ..StyleProps::default()
        }
    );
    assert_eq!(
        resolved,
        Style {
            width: 0.8,
            color: RED,
            dash: Dash::Solid,
            arrow: Arrow::None,
        }
    );
    assert!(matches!(s.style_of("nope"), Err(CommandError::UnknownName(_))));
}

#[test]
fn style_after_set_style() {
    let mut s = Session::new(Document::new());
    apply(&mut s, r#"{"op": "define_style", "style": "axis", "width": 0.8}"#).unwrap();
    s.apply(segment((0.0, 0.0), (1.0, 0.0))).unwrap();
    apply(&mut s, r##"{"op": "set_style", "name": "seg1", "style": "axis", "color": "#c00000"}"##).unwrap();
    let list = s.declarative_commands().unwrap();
    let text = list[0].to_json().unwrap();
    assert!(text.ends_with(r##""style":"axis","color":"#c00000"}"##), "{text}");
    assert!(!text.contains("width") && !text.contains("dash") && !text.contains("arrow"), "{text}");
}
