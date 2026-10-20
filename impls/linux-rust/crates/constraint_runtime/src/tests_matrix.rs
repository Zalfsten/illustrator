use super::*;

#[test]
fn packed_dense_is_contiguous() {
    let mut buf = vec![0.0; 6];
    {
        let mut d = DenseMatrix::packed(&mut buf, 2, 3);
        d[(1, 2)] = 9.0;
    }
    assert_eq!(buf[3 + 2], 9.0);
}

#[test]
fn push_block_stacks_rows() {
    let mut j = BlockSparseMatrix::new(5);
    let b0 = j.push_block(2, &[1, 4]);
    let b1 = j.push_block(1, &[0, 2, 3]);
    assert_eq!((b0, b1), (0, 1));
    assert_eq!((j.nrows(), j.ncols(), j.num_blocks()), (3, 5, 2));
    let blk = j.block(1);
    assert_eq!(blk.rows(), 2..3);
    assert_eq!(blk.cols(), &[0, 2, 3]);
    assert_eq!(blk.values(), &[0.0; 3]);
}

#[test]
fn block_reads_like_a_matrix() {
    let mut j = BlockSparseMatrix::new(4);
    j.push_block(2, &[3, 0, 1]);
    j.block_mut(0)
        .set_scaled(1.0, [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]);
    let b = j.block(0);
    assert_eq!((b.nrows(), b.ncols()), (2, 3));
    assert_eq!((b[(0, 2)], b[(1, 0)], b[(1, 2)]), (3.0, 4.0, 6.0));
    let rows: Vec<&[f64]> = b.row_values().collect();
    assert_eq!(rows, [&[1.0, 2.0, 3.0][..], &[4.0, 5.0, 6.0][..]]);
}

#[test]
fn block_write_round_trips_to_dense() {
    let mut j = BlockSparseMatrix::new(5);
    j.push_block(2, &[1, 4]);
    j.push_block(1, &[0, 2, 3]);
    {
        let mut b = j.block_mut(0);
        b[(0, 0)] = 1.0;
        b[(1, 1)] = 2.0;
    }
    j.block_mut(1).set_scaled(-1.0, [[3.0, 4.0, 5.0]]);
    assert_eq!(j.block(0).values(), &[1.0, 0.0, 0.0, 2.0]);
    let n = j.ncols();
    let mut d = vec![0.0; j.nrows() * n];
    j.add_to_dense(&mut d);
    let at = |row: usize, col: usize| d[row * n + col];
    assert_eq!(at(0, 1), 1.0);
    assert_eq!(at(1, 4), 2.0);
    assert_eq!((at(2, 0), at(2, 2), at(2, 3)), (-3.0, -4.0, -5.0));
    let nonzero = d.iter().filter(|v| **v != 0.0).count();
    assert_eq!(nonzero, 5, "untouched entries stay zero");
}

#[test]
fn duplicate_columns_sum_in_dense_and_transpose_mul() {
    let mut j = BlockSparseMatrix::new(3);
    j.push_block(1, &[2, 0, 2]);
    j.block_mut(0).set_scaled(1.0, [[1.5, 7.0, -0.5]]);
    let mut d = vec![0.0; 3];
    j.add_to_dense(&mut d);
    assert_eq!(d, [7.0, 0.0, 1.0]);
    let mut g = vec![9.0; 3];
    j.transpose_mul(&[2.0], &mut g);
    assert_eq!(g, [14.0, 0.0, 2.0], "g overwritten, duplicates summed");
}

#[test]
fn transpose_mul_matches_dense() {
    let mut j = BlockSparseMatrix::new(4);
    j.push_block(2, &[0, 3]);
    j.push_block(1, &[1, 3, 2]);
    j.block_mut(0).set_scaled(1.0, [[1.0, 2.0], [3.0, 4.0]]);
    j.block_mut(1).set_scaled(1.0, [[5.0, 6.0, 7.0]]);
    let r = [1.0, -1.0, 0.5];
    let mut d = vec![0.0; 3 * 4];
    j.add_to_dense(&mut d);
    let mut want = [0.0; 4];
    for (row, ri) in r.iter().enumerate() {
        for (c, w) in want.iter_mut().enumerate() {
            *w += d[row * 4 + c] * ri;
        }
    }
    let mut g = vec![0.0; 4];
    j.transpose_mul(&r, &mut g);
    assert_eq!(g, want);
}

#[test]
fn block_without_columns_has_no_row_values() {
    let mut j = BlockSparseMatrix::new(2);
    j.push_block(1, &[]);
    assert_eq!(j.block(0).row_values().count(), 0);
    let mut g = vec![1.0; 2];
    j.transpose_mul(&[3.0], &mut g);
    assert_eq!(g, [0.0, 0.0]);
}
