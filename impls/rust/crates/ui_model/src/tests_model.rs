use super::*;
use crate::test_util::*;
use crate::tool::{SlotKind, ToolKind};
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

// --- Tools ---

fn choose(ui: &mut UiModel, s: &mut Session, tool: ToolKind) -> Effect {
    run(ui, s, Intent::ChooseTool(tool))
}

fn try_run(ui: &mut UiModel, s: &mut Session, intent: Intent) -> Result<Effect, UiError> {
    let sh = shown(s.document(), ALL);
    ui.apply(intent, s, &sh.scene())
}

fn select(ui: &mut UiModel, s: &Session, names: &[&str]) {
    for n in names {
        ui.selection_mut().select_name(s.document(), n, PickMode::Toggle);
    }
}

fn constraints(s: &Session) -> usize {
    s.document().constraints().len()
}

#[test]
fn parallel_asks_for_two_directions() {
    let mut s = sample();
    let mut ui = UiModel::default();
    assert_eq!(choose(&mut ui, &mut s, ToolKind::Parallel), Effect::Selection);
    assert_eq!(ui.tool(), ToolKind::Parallel);
    assert!(ui.filled().is_empty());
    assert_eq!(ToolKind::Parallel.slots(), [SlotKind::Direction, SlotKind::Direction]);
    assert_eq!(constraints(&s), 2);
}

#[test]
fn an_arc_does_not_fill_a_direction_slot() {
    let mut s = session(vec![arc("arc1", (0.0, 0.0)), segment("seg1", (0.0, 3.0), (1.0, 3.0))]);
    let mut ui = UiModel::default();
    choose(&mut ui, &mut s, ToolKind::Parallel);
    // On the arc, far from the segment.
    let e = run(&mut ui, &mut s, pick_at(p(std::f64::consts::FRAC_1_SQRT_2, std::f64::consts::FRAC_1_SQRT_2), PickMode::Replace));
    assert_eq!(e, Effect::Refused("Parallel: needs a direction".into()));
    assert!(ui.filled().is_empty());
}

#[test]
fn select_first_then_parallel() {
    let mut s = sample();
    let mut ui = UiModel::default();
    select(&mut ui, &s, &["seg2", "seg3"]);
    let Effect::Issued(out) = choose(&mut ui, &mut s, ToolKind::Parallel) else { panic!("issued expected") };
    assert_eq!(out.created.as_deref(), Some("par2"));
    assert_eq!(ui.last_detail(), " (seg2, seg3)");
    assert!(ui.selection().is_empty());
    assert_eq!(ui.tool(), ToolKind::Parallel, "the tool stays active");
    assert!(ui.filled().is_empty());
}

#[test]
fn partly_filled_from_the_selection() {
    let mut s = sample();
    let mut ui = UiModel::default();
    select(&mut ui, &s, &["seg1"]);
    assert_eq!(choose(&mut ui, &mut s, ToolKind::Parallel), Effect::Selection);
    assert_eq!(ui.filled(), [filled(&s, "seg1")]);
    assert!(ui.selection().is_empty());
}

#[test]
fn unfitting_entries_are_ignored() {
    let mut s = session(vec![arc("arc1", (0.0, 0.0)), segment("seg1", (0.0, 3.0), (1.0, 3.0))]);
    let mut ui = UiModel::default();
    select(&mut ui, &s, &["arc1", "seg1", "seg1.end"]);
    choose(&mut ui, &mut s, ToolKind::Parallel);
    assert_eq!(ui.filled(), [filled(&s, "seg1")]);
}

#[test]
fn tool_first() {
    let mut s = sample();
    let mut ui = UiModel::default();
    choose(&mut ui, &mut s, ToolKind::Join);
    assert_eq!(run(&mut ui, &mut s, pick_at(p(2.0, 1.0), PickMode::Replace)), Effect::Slot);
    let Effect::Issued(out) = run(&mut ui, &mut s, pick_at(p(5.0, 5.0), PickMode::Replace)) else { panic!() };
    assert_eq!(out.created.as_deref(), Some("join1"));
    assert_eq!(ui.last_detail(), " (seg2.end = seg3.start)");
}

#[test]
fn the_same_object_twice_is_refused() {
    let mut s = sample();
    let mut ui = UiModel::default();
    choose(&mut ui, &mut s, ToolKind::Parallel);
    run(&mut ui, &mut s, pick_at(p(0.5, 0.0), PickMode::Replace));
    let e = run(&mut ui, &mut s, pick_at(p(0.6, 0.0), PickMode::Replace));
    assert_eq!(e, Effect::Refused("Parallel: seg1 is already picked".into()));
    assert_eq!(ui.filled().len(), 1);
}

#[test]
fn two_constraints_in_a_row() {
    let mut s = sample();
    let mut ui = UiModel::default();
    choose(&mut ui, &mut s, ToolKind::Fix);
    assert!(matches!(run(&mut ui, &mut s, pick_at(p(0.0, 0.0), PickMode::Replace)), Effect::Issued(_)));
    assert!(matches!(run(&mut ui, &mut s, pick_at(p(2.0, 1.0), PickMode::Replace)), Effect::Issued(_)));
    assert_eq!(constraints(&s), 4);
    assert_eq!(ui.tool(), ToolKind::Fix);
}

#[test]
fn rejected_command_keeps_the_tool() {
    // seg1.start fixed at (0, 0), seg2.end at (2, 1): joining them is unsatisfiable.
    let mut s = sample();
    let mut ui = UiModel::default();
    choose(&mut ui, &mut s, ToolKind::Fix);
    run(&mut ui, &mut s, pick_at(p(0.0, 0.0), PickMode::Replace));
    run(&mut ui, &mut s, pick_at(p(2.0, 1.0), PickMode::Replace));
    let before = constraints(&s);
    choose(&mut ui, &mut s, ToolKind::Join);
    run(&mut ui, &mut s, pick_at(p(0.0, 0.0), PickMode::Replace));
    let e = try_run(&mut ui, &mut s, pick_at(p(2.0, 1.0), PickMode::Replace));
    assert!(matches!(e, Err(UiError::Command(_))), "{e:?}");
    assert_eq!(constraints(&s), before, "drawing unchanged");
    assert_eq!(ui.tool(), ToolKind::Join);
    assert!(ui.filled().is_empty());
}

#[test]
fn cancel_a_half_filled_tool() {
    let mut s = sample();
    let mut ui = UiModel::default();
    choose(&mut ui, &mut s, ToolKind::Parallel);
    run(&mut ui, &mut s, pick_at(p(0.5, 0.0), PickMode::Replace));
    run(&mut ui, &mut s, Intent::Cancel);
    assert_eq!(ui.tool(), ToolKind::Select);
    assert!(ui.filled().is_empty());
    assert_eq!(constraints(&s), 2);
    // In Select, cancel clears the selection.
    select(&mut ui, &s, &["seg1"]);
    run(&mut ui, &mut s, Intent::Cancel);
    assert!(ui.selection().is_empty());
}

#[test]
fn prompt_names_the_next_slot() {
    let mut s = sample();
    let mut ui = UiModel::default();
    assert_eq!(ui.prompt(s.document()), None);
    choose(&mut ui, &mut s, ToolKind::Parallel);
    assert_eq!(ui.prompt(s.document()).as_deref(), Some("Parallel: → pick a direction"));
    run(&mut ui, &mut s, pick_at(p(0.5, 0.0), PickMode::Replace));
    assert_eq!(ui.prompt(s.document()).as_deref(), Some("Parallel: seg1 → pick a direction"));
}

#[test]
fn same_intents_same_commands() {
    let intents = [
        Intent::ChooseTool(ToolKind::Segment),
        pick_at(p(7.0, 0.0), PickMode::Replace),
        pick_at(p(8.0, 1.0), PickMode::Replace),
        Intent::ChooseTool(ToolKind::Join),
        pick_at(p(8.0, 1.0), PickMode::Replace),
        pick_at(p(6.0, 5.0), PickMode::Replace),
    ];
    let (mut s1, mut s2) = (sample(), sample());
    let (mut u1, mut u2) = (UiModel::default(), UiModel::default());
    for i in intents {
        run(&mut u1, &mut s1, i);
        run(&mut u2, &mut s2, i);
    }
    let export = |s: &Session| s.declarative_commands().unwrap();
    assert_eq!(export(&s1), export(&s2));
    assert_eq!(s1.document().geometries().len(), 4);
    assert_eq!(constraints(&s1), 3);
}

#[test]
fn box_is_ignored_in_a_tool_and_clear_ends_the_tool() {
    let mut s = sample();
    let mut ui = UiModel::default();
    choose(&mut ui, &mut s, ToolKind::Join);
    let e = run(&mut ui, &mut s, Intent::Box { a: p(-1.0, -1.0), b: p(9.0, 9.0), mode: BoxMode::Replace });
    assert_eq!(e, Effect::Nothing);
    assert!(ui.selection().is_empty());
    ui.clear();
    assert_eq!(ui.tool(), ToolKind::Select);
}

#[test]
fn undo_drops_a_filled_slot_whose_object_is_gone() {
    let mut s = sample();
    s.apply(segment("seg9", (9.0, 0.0), (10.0, 0.0))).unwrap();
    let mut ui = UiModel::default();
    select(&mut ui, &s, &["seg9"]);
    choose(&mut ui, &mut s, ToolKind::Parallel);
    s.undo().unwrap();
    ui.reconcile(s.document());
    assert!(ui.filled().is_empty());
}
