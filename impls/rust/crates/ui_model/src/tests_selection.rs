use super::*;
use crate::test_util::*;
use sketch_session::Command;

fn labels(sel: &Selection, s: &sketch_session::Session) -> String {
    sel.describe(s.document())
}

#[test]
fn point_shown_by_address() {
    let s = session(vec![segment("seg1", (0.0, 0.0), (1.0, 0.0))]);
    let mut sel = Selection::default();
    assert!(sel.select_name(s.document(), "seg1.end", PickMode::Replace));
    assert_eq!(labels(&sel, &s), "seg1.end");
}

#[test]
fn joined_ends_are_one_entry() {
    let s = sample();
    let doc = s.document();
    let mut sel = Selection::default();
    sel.select_name(doc, "seg1.end", PickMode::Replace);
    sel.select_name(doc, "seg2.start", PickMode::Toggle);
    assert!(sel.is_empty(), "toggling another address of the class removes it");
    sel.select_name(doc, "seg2.start", PickMode::Replace);
    sel.add(entry(&s, "seg1.end"), doc);
    assert_eq!(sel.entries().len(), 1, "no duplicate through another address");
}

#[test]
fn rename_keeps_the_entry() {
    let mut s = sample();
    let mut sel = Selection::default();
    sel.select_name(s.document(), "seg1", PickMode::Replace);
    s.apply(Command::Rename { name: "seg1".into(), new_name: "base".into() }).unwrap();
    sel.reconcile(s.document());
    assert_eq!(labels(&sel, &s), "base");
}

#[test]
fn deleted_objects_leave_the_selection() {
    let mut s = sample();
    let mut sel = Selection::default();
    sel.select_name(s.document(), "seg1", PickMode::Replace);
    sel.select_name(s.document(), "par1", PickMode::Toggle);
    sel.select_name(s.document(), "seg1.start", PickMode::Toggle);
    s.apply(Command::Delete { name: "seg1".into() }).unwrap();
    sel.reconcile(s.document());
    assert!(sel.is_empty(), "{}", labels(&sel, &s));
}

#[test]
fn undo_keeps_existing_objects_selected() {
    let mut s = sample();
    let mut sel = Selection::default();
    sel.select_name(s.document(), "seg1", PickMode::Replace);
    s.apply(segment("seg4", (9.0, 9.0), (10.0, 9.0))).unwrap();
    s.undo().unwrap();
    sel.reconcile(s.document());
    assert_eq!(labels(&sel, &s), "seg1");
}

#[test]
fn select_name_takes_geometries_constraints_and_points() {
    let s = sample();
    let doc = s.document();
    let mut sel = Selection::default();
    assert!(sel.select_name(doc, "seg3", PickMode::Toggle));
    assert!(sel.select_name(doc, "par1", PickMode::Toggle));
    assert!(sel.select_name(doc, "seg2.end", PickMode::Toggle));
    assert!(!sel.select_name(doc, "nope", PickMode::Toggle));
    assert!(!sel.select_name(doc, "seg1.direction", PickMode::Toggle), "not a point");
    assert_eq!(labels(&sel, &s), "seg3, par1, seg2.end");
    assert!(matches!(sel.entries()[0], Entry::Geometry(_)));
    assert!(matches!(sel.entries()[1], Entry::Constraint(_)));
}
