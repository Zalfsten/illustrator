use super::*;
use crate::pick::nearest_line;
use crate::test_util::*;
use constraint_runtime::{Rgb, StyleProps};
use sketch_session::Session;

fn look() -> StyleRef {
    StyleRef {
        style: "axis".into(),
        overrides: StyleProps {
            color: Some(Rgb(0xc0, 0, 0)),
            ..StyleProps::default()
        },
    }
}

fn command(s: &Session, tool: ToolKind, filled: &[Filled]) -> (Command, String) {
    tool.command(s.document(), filled, &look()).unwrap()
}

#[test]
fn segment_uses_the_look() {
    let s = sample();
    let (a, b) = (Point2::new(5.0, 5.0), Point2::new(6.0, 5.0));
    let (cmd, detail) = command(&s, ToolKind::Segment, &[Filled::Position(a), Filled::Position(b)]);
    assert_eq!(
        cmd,
        Command::Create {
            name: None,
            role: constraint_runtime::GeometryRole::Result,
            look: look(),
            geometry: GeometrySpec::Segment(SegmentParams { start: a, end: b }),
        }
    );
    assert_eq!(detail, "");
}

#[test]
fn join_takes_two_point_addresses() {
    let s = sample();
    let (cmd, detail) = command(&s, ToolKind::Join, &[filled(&s, "seg2.end"), filled(&s, "seg3.start")]);
    assert_eq!(
        cmd,
        Command::Constrain {
            name: None,
            constraint: ConstraintSpec::Congruence(PointPair { a: "seg2.end".into(), b: "seg3.start".into() }),
        }
    );
    assert_eq!(detail, " (seg2.end = seg3.start)");
}

#[test]
fn parallel_and_orthogonal_use_directions() {
    let s = sample();
    let pair = [filled(&s, "seg1"), filled(&s, "seg2")];
    let vec_pair = VecPair { a: "seg1.direction".into(), b: "seg2.direction".into() };
    let (par, detail) = command(&s, ToolKind::Parallel, &pair);
    assert_eq!(par, Command::Constrain { name: None, constraint: ConstraintSpec::VectorsParallel(vec_pair.clone()) });
    assert_eq!(detail, " (seg1, seg2)");
    let (orth, _) = command(&s, ToolKind::Orthogonal, &pair);
    assert_eq!(orth, Command::Constrain { name: None, constraint: ConstraintSpec::VectorsOrthogonal(vec_pair) });
}

#[test]
fn fix_holds_the_current_location() {
    let s = sample();
    let (cmd, detail) = command(&s, ToolKind::Fix, &[filled(&s, "seg2.end")]);
    assert_eq!(
        cmd,
        Command::Constrain {
            name: None,
            constraint: ConstraintSpec::FixPoint(FixParams { point: "seg2.end".into(), target: Point2::new(2.0, 1.0) }),
        }
    );
    assert_eq!(detail, " (seg2.end)");
}

#[test]
fn brush_sets_style_and_all_four_overrides() {
    let s = sample();
    let (cmd, _) = command(&s, ToolKind::Brush, &[filled(&s, "seg3")]);
    assert_eq!(
        cmd,
        Command::SetStyle {
            name: "seg3".into(),
            style: Some("axis".into()),
            patch: StylePatch { width: Some(None), color: Some(Some(Rgb(0xc0, 0, 0))), dash: Some(None), arrow: Some(None) },
        }
    );
}

#[test]
fn an_arc_does_not_fit_a_direction_slot() {
    let s = session(vec![arc("arc1", (0.0, 0.0)), segment("seg1", (0.0, 3.0), (1.0, 3.0))]);
    let doc = s.document();
    assert!(!SlotKind::Direction.fits(&entry(&s, "arc1"), doc));
    assert!(SlotKind::Curve.fits(&entry(&s, "arc1"), doc));
    assert!(SlotKind::Direction.fits(&entry(&s, "seg1"), doc));
    assert!(SlotKind::Point.fits(&entry(&s, "seg1.end"), doc));
    assert!(!SlotKind::Point.fits(&entry(&s, "seg1"), doc));
    assert!(!SlotKind::Position.fits(&entry(&s, "seg1.end"), doc));
}

#[test]
fn nearest_line_skips_a_nearer_arc() {
    // The arc passes through (1, 0); the line y = -0.3 is farther away.
    let s = session(vec![arc("arc1", (0.0, 0.0)), segment("seg1", (0.0, -0.3), (2.0, -0.3))]);
    let doc = s.document();
    let sh = shown(doc, constraint_runtime::MeshFilter::INCLUDE_CONSTRUCTION);
    let at = Point2::new(1.0, -0.05);
    assert_eq!(crate::pick::nearest_curve(&sh.scene(), at, 0.5).and_then(|id| doc.geometry_name(id)), Some("arc1"));
    assert_eq!(nearest_line(doc, &sh.scene(), at, 0.5).and_then(|id| doc.geometry_name(id)), Some("seg1"));
}
