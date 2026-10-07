use super::*;
use crate::state::State;

#[test]
fn double_index_matrix_and_state() {
    let mut m = Matrix::zeros(2, 3);
    m[(1, 2)] = 5.0;
    assert_eq!(m[(1, 2)], 5.0);
    m[(1, 2)] += 1.0;
    assert_eq!(m.as_slice()[1 * 3 + 2], 6.0);

    let mut p = State::zeros(3);
    p[1] = 4.0;
    assert_eq!(p[1], 4.0);
}

#[test]
fn dense_block_writes_into_parent() {
    let mut m = Matrix::zeros(3, 4);
    {
        let mut view = m.as_dense();
        let mut block = view.dense_block(1, 1, 2, 2);
        block[(0, 0)] = 1.0;
        block[(1, 1)] = 2.0;
    }
    assert_eq!(m[(1, 1)], 1.0);
    assert_eq!(m[(2, 2)], 2.0);
}

#[test]
fn scatter_maps_local_columns() {
    let mut m = Matrix::zeros(2, 5);
    let conn = [1usize, 4];
    {
        let mut view = m.as_dense();
        let mut s = view.scatter(0, 2, &conn);
        s.set(0, 0, 3.0);
        s.set(1, 1, 4.0);
    }
    assert_eq!(m.as_slice()[0 * 5 + 1], 3.0);
    assert_eq!(m.as_slice()[1 * 5 + 4], 4.0);
}

#[test]
fn jac_cols_offsets_columns() {
    let mut m = Matrix::zeros(2, 4);
    {
        let mut view = m.as_dense();
        let mut cols = JacCols::from(&mut view, 2);
        cols.set(0, 0, 1.0);
        cols.set(1, 1, 2.0);
    }
    assert_eq!(m[(0, 2)], 1.0);
    assert_eq!(m[(1, 3)], 2.0);
}

#[test]
fn packed_dense_is_contiguous() {
    let mut buf = vec![0.0; 6];
    {
        let mut d = DenseJac::packed(&mut buf, 2, 3);
        d.set(1, 2, 9.0);
    }
    assert_eq!(buf[1 * 3 + 2], 9.0);
}

#[test]
fn state_dense_block_and_scatter() {
    let mut p = State::zeros(6);
    {
        let mut view = p.as_dense();
        let mut block = view.dense_block(2, 3);
        block.set(0, 1.0);
        block.set(2, 3.0);
    }
    assert_eq!(p.as_slice()[2], 1.0);
    assert_eq!(p.as_slice()[4], 3.0);

    let conn = [1usize, 5];
    {
        let mut view = p.as_dense();
        let mut s = view.scatter(&conn);
        s.set(0, 7.0);
        s.set(1, 8.0);
    }
    assert_eq!(p.as_slice()[1], 7.0);
    assert_eq!(p.as_slice()[5], 8.0);
}
