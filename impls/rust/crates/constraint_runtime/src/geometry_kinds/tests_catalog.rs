use super::*;

#[test]
fn capability_sets_match_kinds() {
    assert_eq!(
        GeometryKind::Segment.caps(),
        Caps::CURVE.with_endpoints().with_line()
    );
    assert_eq!(
        GeometryKind::Arc.caps(),
        Caps::CURVE.with_endpoints().with_circular()
    );
    assert_eq!(GeometryKind::Label.caps(), Caps::NONE.with_anchor().with_line());
}

/// Every geometry kind (extend with the catalog).
const ALL_KINDS: [GeometryKind; 3] = [GeometryKind::Segment, GeometryKind::Arc, GeometryKind::Label];

#[test]
fn name_prefixes_are_valid_stems() {
    for kind in ALL_KINDS {
        let p = kind.name_prefix();
        assert!(crate::names::is_valid_name(p), "{kind:?}: {p:?}");
        assert!(
            !p.ends_with(|c: char| c.is_ascii_digit()),
            "{kind:?}: {p:?}"
        );
    }
}
