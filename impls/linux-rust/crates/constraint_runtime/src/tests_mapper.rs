use super::*;
use crate::geometry::{GeometryId, GeometryRole, GeometryShell};
use crate::geometry_kinds::Segment;

#[test]
fn local_indices_applies_param_offset() {
    let g = Segment {
        shell: GeometryShell::new(GeometryId(0), GeometryRole::Result, 4),
    };
    assert_eq!(Mapper::local_indices(&g, [0, 1]), vec![4, 5]);
}

#[test]
fn join_indices_concatenates_in_order() {
    let idx = Mapper::join_indices([vec![0, 1], vec![4, 5]]);
    assert_eq!(idx, vec![0, 1, 4, 5]);
    assert_eq!(idx.len(), 4);
}

#[test]
fn geometry_indices_is_dense_block_as_list() {
    let g = Segment {
        shell: GeometryShell::new(GeometryId(0), GeometryRole::Result, 4),
    };
    let idx = Mapper::geometry_indices(&g);
    assert_eq!(idx, vec![4, 5, 6, 7]);
    assert_eq!(idx, Mapper::chart_param_block(&g).collect::<Vec<_>>());
}
