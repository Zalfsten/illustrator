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
}
