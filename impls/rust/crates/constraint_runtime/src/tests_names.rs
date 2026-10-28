use super::*;

fn geo(raw: usize) -> ObjectId {
    ObjectId::Geometry(GeometryId::from(raw))
}

fn con(raw: usize) -> ObjectId {
    ObjectId::Constraint(ConstraintId::from(raw))
}

/// Table with geometries named `names[i]` under id `i`.
fn table(names: &[&str]) -> Names {
    let mut t = Names::default();
    for (i, n) in names.iter().enumerate() {
        t.insert(geo(i), (*n).to_owned());
    }
    t
}

#[test]
fn valid_and_invalid_names() {
    for n in ["seg1", "x_axis", "_tmp", "Seg1", "a"] {
        assert!(is_valid_name(n), "{n}");
    }
    for n in ["1seg", "seg.1", "seg-1", "", "län", "a b"] {
        assert!(!is_valid_name(n), "{n:?}");
    }
}

#[test]
fn case_distinguishes_names() {
    let t = table(&["Seg1", "seg1"]);
    assert_eq!(t.lookup("Seg1"), Some(geo(0)));
    assert_eq!(t.lookup("seg1"), Some(geo(1)));
}

#[test]
fn lookup_in_both_directions() {
    let mut t = table(&["seg1"]);
    t.insert(con(0), "par1".into());
    assert_eq!(t.lookup("par1"), Some(con(0)));
    assert_eq!(t.name(con(0)), Some("par1"));
    assert_eq!(t.name(geo(0)), Some("seg1"));
    assert_eq!(t.name(geo(1)), None);
}

#[test]
fn numbers_continue_after_the_highest_in_use() {
    assert_eq!(table(&["seg1", "seg2", "seg5"]).next_auto("seg"), "seg6");
}

#[test]
fn prefixes_are_counted_independently() {
    let t = table(&["seg1", "seg2"]);
    assert_eq!(t.next_auto("arc"), "arc1");
    assert_eq!(t.next_auto("seg"), "seg3");
}

#[test]
fn top_name_is_reused_after_removal() {
    let mut t = table(&["seg1", "seg2"]);
    assert_eq!(t.remove(geo(1)).as_deref(), Some("seg2"));
    assert_eq!(t.lookup("seg2"), None);
    assert_eq!(t.next_auto("seg"), "seg2");
}

#[test]
fn user_chosen_names_count() {
    let mut t = table(&["seg1", "seg2", "seg3"]);
    t.rename(geo(0), "seg7").unwrap();
    assert_eq!(t.next_auto("seg"), "seg8");
}

#[test]
fn non_canonical_numbers_are_not_counted() {
    let t = table(&["seg01", "seg1"]);
    assert_eq!(t.next_auto("seg"), "seg2");
    let t = table(&["seg01"]);
    assert_eq!(t.next_auto("seg"), "seg1", "seg01 is a different name");
}

#[test]
fn overflow_falls_back_to_the_lowest_gap() {
    let t = table(&["seg1", &format!("seg{}", u64::MAX)]);
    assert_eq!(t.next_auto("seg"), "seg2");
}

#[test]
fn rename_to_a_free_name() {
    let mut t = table(&["seg1"]);
    assert_eq!(t.rename(geo(0), "x_axis"), Ok(()));
    assert_eq!(t.lookup("x_axis"), Some(geo(0)));
    assert_eq!(t.lookup("seg1"), None);
    assert_eq!(t.next_auto("seg"), "seg1");
}

#[test]
fn rename_to_a_taken_name_is_rejected() {
    let mut t = table(&["seg1", "seg2"]);
    assert_eq!(t.rename(geo(0), "seg2"), Err(NameError::Taken));
    assert_eq!(t.name(geo(0)), Some("seg1"));
    assert_eq!(t.name(geo(1)), Some("seg2"));
}

#[test]
fn rename_to_an_invalid_name_is_rejected() {
    let mut t = table(&["seg1"]);
    assert_eq!(t.rename(geo(0), "my.axis"), Err(NameError::Invalid));
    assert_eq!(t.name(geo(0)), Some("seg1"));
}

#[test]
fn rename_to_the_current_name_is_a_no_op() {
    let mut t = table(&["seg1"]);
    assert_eq!(t.rename(geo(0), "seg1"), Ok(()));
    assert_eq!(t.lookup("seg1"), Some(geo(0)));
}

#[test]
fn rename_of_an_unknown_object_is_rejected() {
    let mut t = table(&["seg1"]);
    assert_eq!(t.rename(geo(5), "x"), Err(NameError::UnknownObject));
}
