use super::*;

#[test]
fn capability_flags_match_kinds() {
    assert!(GeometryKind::Segment.has_line_exprs());
    assert!(!GeometryKind::Segment.has_circular_exprs());
    assert!(GeometryKind::Arc.has_circular_exprs());
    assert!(!GeometryKind::Arc.has_line_exprs());
}
