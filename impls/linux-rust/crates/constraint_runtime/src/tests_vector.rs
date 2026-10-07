use super::*;

#[test]
fn double_index_vector() {
    let mut p = Vector::zeros(3);
    p[1] = 4.0;
    assert_eq!(p[1], 4.0);
}

#[test]
fn block_slice_and_scatter_slice() {
    let mut p = Vector::zeros(6);
    {
        let mut view = p.as_dense();
        let mut block = view.block_slice(2, 3);
        block.set(0, 1.0);
        block.set(2, 3.0);
    }
    assert_eq!(p.as_slice()[2], 1.0);
    assert_eq!(p.as_slice()[4], 3.0);

    let conn = [1usize, 5];
    {
        let mut view = p.as_dense();
        let mut s = view.scatter_slice(&conn);
        s.set(0, 7.0);
        s.set(1, 8.0);
    }
    assert_eq!(p.as_slice()[1], 7.0);
    assert_eq!(p.as_slice()[5], 8.0);
}
