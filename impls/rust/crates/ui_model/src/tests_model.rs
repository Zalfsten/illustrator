use super::*;
use crate::test_util::*;
use constraint_runtime::GeometryRole;

const ALL: MeshFilter = MeshFilter { include_construction: true };

fn p(x: f64, y: f64) -> Point2 {
    Point2::new(x, y)
}

fn pick_at(at: Point2, mode: PickMode) -> Intent {
    Intent::Pick { at, tolerance: 0.1, mode }
}

/// Apply `intent` with the Canvas showing everything.
fn run(ui: &mut UiModel, s: &mut Session, intent: Intent) -> Effect {
    let sh = shown(s.document(), ALL);
    ui.apply(intent, s, &sh.scene()).unwrap()
}

fn selected(ui: &UiModel, s: &Session) -> String {
    ui.selection().describe(s.document())
}

#[test]
fn toggle_adds_and_removes() {
    let mut s = sample();
    let mut ui = UiModel::default();
    run(&mut ui, &mut s, pick_at(p(0.5, 0.0), PickMode::Replace));
    run(&mut ui, &mut s, pick_at(p(5.5, 5.0), PickMode::Toggle));
    assert_eq!(selected(&ui, &s), "seg1, seg3");
    run(&mut ui, &mut s, pick_at(p(0.5, 0.0), PickMode::Toggle));
    assert_eq!(selected(&ui, &s), "seg3");
}

#[test]
fn replace_pick_on_empty_space_clears() {
    let mut s = sample();
    let mut ui = UiModel::default();
    run(&mut ui, &mut s, pick_at(p(0.5, 0.0), PickMode::Replace));
    assert_eq!(run(&mut ui, &mut s, pick_at(p(3.0, 3.0), PickMode::Toggle)), Effect::Nothing);
    assert_eq!(selected(&ui, &s), "seg1", "a toggle pick on empty space changes nothing");
    run(&mut ui, &mut s, pick_at(p(3.0, 3.0), PickMode::Replace));
    assert!(ui.selection().is_empty());
}

#[test]
fn clear_empties_the_selection() {
    let mut s = sample();
    let mut ui = UiModel::default();
    run(&mut ui, &mut s, pick_at(p(0.5, 0.0), PickMode::Replace));
    run(&mut ui, &mut s, Intent::Clear);
    assert!(ui.selection().is_empty());
}

#[test]
fn add_keeps_the_earlier_selection() {
    let mut s = sample();
    let mut ui = UiModel::default();
    run(&mut ui, &mut s, pick_at(p(5.5, 5.0), PickMode::Replace));
    let around_seg1 = |mode| Intent::Box { a: p(-0.5, -0.5), b: p(1.5, 0.5), mode };
    run(&mut ui, &mut s, around_seg1(BoxMode::Add));
    assert_eq!(selected(&ui, &s), "seg3, seg1");
    run(&mut ui, &mut s, around_seg1(BoxMode::Replace));
    assert_eq!(selected(&ui, &s), "seg1");
}

#[test]
fn one_undo_step() {
    let mut s = sample();
    let mut ui = UiModel::default();
    let doc = s.document();
    ui.selection_mut().select_name(doc, "seg1", PickMode::Replace);
    ui.selection_mut().select_name(doc, "par1", PickMode::Toggle);
    ui.selection_mut().select_name(doc, "seg2", PickMode::Toggle);
    let Effect::Deleted(mut gone) = run(&mut ui, &mut s, Intent::Delete) else {
        panic!("deleted expected")
    };
    gone.sort();
    assert_eq!(gone, ["j1", "par1", "seg1", "seg2"]);
    assert!(ui.selection().is_empty());
    assert_eq!(s.document().geometries().len(), 1);
    s.undo().unwrap();
    for name in ["seg1", "seg2", "j1", "par1"] {
        assert!(s.document().lookup(name).is_some(), "{name} back after one undo");
    }
}

#[test]
fn a_point_deletes_the_curves_at_it() {
    let mut s = sample();
    let mut ui = UiModel::default();
    ui.selection_mut().select_name(s.document(), "seg1.end", PickMode::Replace);
    let Effect::Deleted(mut gone) = run(&mut ui, &mut s, Intent::Delete) else {
        panic!("deleted expected")
    };
    gone.sort();
    assert_eq!(gone, ["j1", "par1", "seg1", "seg2"]);
    assert!(s.document().lookup("seg3").is_some());
    assert!(ui.selection().is_empty());
    s.undo().unwrap();
    assert!(s.document().lookup("seg1").is_some() && s.document().lookup("seg2").is_some(), "one step");
}

#[test]
fn a_point_and_its_curve_delete_the_curve_once() {
    let mut s = sample();
    let mut ui = UiModel::default();
    ui.selection_mut().select_name(s.document(), "seg3", PickMode::Replace);
    ui.selection_mut().select_name(s.document(), "seg3.start", PickMode::Toggle);
    assert!(matches!(run(&mut ui, &mut s, Intent::Delete), Effect::Deleted(_)));
    assert!(s.document().lookup("seg3").is_none());
}

#[test]
fn empty_selection_deletes_nothing() {
    let mut s = sample();
    let mut ui = UiModel::default();
    assert_eq!(run(&mut ui, &mut s, Intent::Delete), Effect::Nothing);
    for _ in 0..5 {
        s.undo().unwrap();
    }
    assert!(!s.can_undo(), "the delete created no step");
}

#[test]
fn same_intents_same_selection() {
    let intents = [
        pick_at(p(0.5, 0.0), PickMode::Replace),
        pick_at(p(1.0, 0.0), PickMode::Toggle),
        Intent::Box { a: p(4.0, 4.0), b: p(7.0, 6.0), mode: BoxMode::Add },
        pick_at(p(0.5, 0.0), PickMode::Toggle),
    ];
    let (mut s1, mut s2) = (sample(), sample());
    let (mut u1, mut u2) = (UiModel::default(), UiModel::default());
    for i in intents {
        run(&mut u1, &mut s1, i);
        run(&mut u2, &mut s2, i);
    }
    assert_eq!(selected(&u1, &s1), selected(&u2, &s2));
    assert_eq!(selected(&u1, &s1), "seg1.end, seg3");
}

#[test]
fn switching_to_result_only_deselects_construction() {
    let s = session(vec![
        segment_as("aux", GeometryRole::Construction, (0.0, 0.0), (1.0, 0.0)),
        segment("seg1", (0.0, 1.0), (1.0, 1.0)),
    ]);
    let mut ui = UiModel::default();
    for name in ["aux", "seg1", "aux.end", "seg1.end"] {
        ui.selection_mut().select_name(s.document(), name, PickMode::Toggle);
    }
    ui.hide_invisible(s.document(), MeshFilter::RESULT_ONLY);
    assert_eq!(selected(&ui, &s), "seg1, seg1.end");
}

#[test]
fn hidden_objects_stay_selectable_by_name() {
    let s = session(vec![segment_as("aux", GeometryRole::Construction, (0.0, 0.0), (1.0, 0.0))]);
    let mut ui = UiModel::default();
    let result_only = shown(s.document(), MeshFilter::RESULT_ONLY);
    assert_eq!(crate::pick::pick(s.document(), &result_only.scene(), p(0.5, 0.0), 0.1), None);
    assert!(ui.selection_mut().select_name(s.document(), "aux", PickMode::Replace));
    assert_eq!(selected(&ui, &s), "aux");
}

/// Only curves with an *endpoint* in the class that the Canvas shows: not a
/// curve through the point, not a hidden construction curve (review of PR #108).
#[test]
fn a_point_deletes_only_shown_curves_ending_at_it() {
    let mut s = session(vec![
        segment("seg1", (0.0, 0.0), (1.0, 0.0)),
        segment("seg2", (0.5, -1.0), (1.5, 1.0)),
        join("j1", "seg1.end", "seg2.at(0.5)"),
        segment_as("aux", GeometryRole::Construction, (1.0, 0.0), (3.0, 3.0)),
        join("j2", "seg1.end", "aux.start"),
    ]);
    let mut ui = UiModel::default();
    ui.selection_mut().select_name(s.document(), "seg1.end", PickMode::Replace);
    let sh = shown(s.document(), MeshFilter { include_construction: false });
    let Ok(Effect::Deleted(mut gone)) = ui.apply(Intent::Delete, &mut s, &sh.scene()) else {
        panic!("deleted expected")
    };
    gone.sort();
    assert_eq!(gone, ["j1", "j2", "seg1"]);
    assert!(s.document().lookup("seg2").is_some() && s.document().lookup("aux").is_some());
}

/// With construction shown, the construction curve ending at the point goes too.
#[test]
fn a_point_deletes_shown_construction_curves_ending_at_it() {
    let mut s = session(vec![
        segment("seg1", (0.0, 0.0), (1.0, 0.0)),
        segment_as("aux", GeometryRole::Construction, (1.0, 0.0), (3.0, 3.0)),
        join("j1", "seg1.end", "aux.start"),
    ]);
    let mut ui = UiModel::default();
    ui.selection_mut().select_name(s.document(), "aux.start", PickMode::Replace);
    assert!(matches!(run(&mut ui, &mut s, Intent::Delete), Effect::Deleted(_)));
    assert!(s.document().lookup("seg1").is_none() && s.document().lookup("aux").is_none());
}
