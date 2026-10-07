use super::*;

#[test]
fn double_index_matrix() {
    let mut m = Matrix::zeros(2, 3);
    m[(1, 2)] = 5.0;
    assert_eq!(m[(1, 2)], 5.0);
    m[(1, 2)] += 1.0;
    assert_eq!(m.as_slice()[1 * 3 + 2], 6.0);
}

#[test]
fn block_slice_writes_into_parent() {
    let mut m = Matrix::zeros(3, 4);
    {
        let mut view = m.as_dense();
        let mut block = view.block_slice(1, 1, 2, 2);
        block[(0, 0)] = 1.0;
        block[(1, 1)] = 2.0;
    }
    assert_eq!(m[(1, 1)], 1.0);
    assert_eq!(m[(2, 2)], 2.0);
}

#[test]
fn scatter_slice_maps_local_columns() {
    let mut m = Matrix::zeros(2, 5);
    let conn = [1usize, 4];
    {
        let mut view = m.as_dense();
        let mut s = view.scatter_slice(0, 2, &conn);
        s.set(0, 0, 3.0);
        s.set(1, 1, 4.0);
    }
    assert_eq!(m.as_slice()[0 * 5 + 1], 3.0);
    assert_eq!(m.as_slice()[1 * 5 + 4], 4.0);
}

#[test]
fn dense_block_slice_column_window() {
    let mut m = Matrix::zeros(2, 4);
    {
        let mut view = m.as_dense();
        let mut cols = view.block_slice(0, 2, 2, 2);
        cols.set(0, 0, 1.0);
        cols.set(1, 1, 2.0);
    }
    assert_eq!(m[(0, 2)], 1.0);
    assert_eq!(m[(1, 3)], 2.0);
}

#[test]
fn scatter_block_slice_column_window() {
    let mut m = Matrix::zeros(1, 6);
    let conn = [0usize, 2, 4, 5];
    {
        let mut view = m.as_dense();
        let mut s = view.scatter_slice(0, 1, &conn);
        let mut cols = s.block_slice(0, 2, 1, 2);
        cols.set(0, 0, 3.0);
        cols.set(0, 1, 4.0);
    }
    assert_eq!(m[(0, 4)], 3.0);
    assert_eq!(m[(0, 5)], 4.0);
}

#[test]
fn scatter_block_slice_and_nested_scatter() {
    let mut m = Matrix::zeros(2, 6);
    let conn = [0usize, 2, 4, 5];
    {
        let mut view = m.as_dense();
        let mut s = view.scatter_slice(0, 2, &conn);
        {
            let mut block = s.block_slice(1, 1, 1, 2);
            block.set(0, 0, 1.0);
            block.set(0, 1, 2.0);
        }
        let nested = [0usize, 2];
        let mut inner = s.scatter_slice(0, 1, &nested);
        inner.set(0, 0, 3.0);
        inner.set(0, 1, 4.0);
    }
    assert_eq!(m[(1, 2)], 1.0);
    assert_eq!(m[(1, 4)], 2.0);
    assert_eq!(m[(0, 0)], 3.0);
    assert_eq!(m[(0, 4)], 4.0);
}

#[test]
fn packed_dense_is_contiguous() {
    let mut buf = vec![0.0; 6];
    {
        let mut d = DenseMatrix::packed(&mut buf, 2, 3);
        d.set(1, 2, 9.0);
    }
    assert_eq!(buf[1 * 3 + 2], 9.0);
}
