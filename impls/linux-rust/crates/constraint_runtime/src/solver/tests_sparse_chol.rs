use super::*;

/// Upper-triangular CSC of a dense symmetric matrix, keeping structural
/// nonzeros plus the full diagonal.
fn upper_csc(a: &[Vec<f64>]) -> (Vec<usize>, Vec<usize>, Vec<f64>) {
    let n = a.len();
    let (mut col_ptr, mut row_idx, mut vals) = (vec![0], Vec::new(), Vec::new());
    for c in 0..n {
        for (r, row) in a.iter().enumerate().take(c + 1) {
            if r == c || row[c] != 0.0 {
                row_idx.push(r);
                vals.push(row[c]);
            }
        }
        col_ptr.push(row_idx.len());
    }
    (col_ptr, row_idx, vals)
}

/// A = BᵀB + I for a sparse B; column 5 is untouched by B (diagonal only).
fn spd_fixture() -> Vec<Vec<f64>> {
    let n = 6;
    let b_rows: [&[(usize, f64)]; 4] = [
        &[(0, 1.0), (2, -2.0)],
        &[(1, 0.5), (2, 1.0), (3, 3.0)],
        &[(0, -1.0), (4, 2.0)],
        &[(3, 1.0), (4, -0.5)],
    ];
    let mut a = vec![vec![0.0; n]; n];
    for row in b_rows {
        for &(i, bi) in row {
            for &(k, bk) in row {
                a[i][k] += bi * bk;
            }
        }
    }
    for (i, r) in a.iter_mut().enumerate() {
        r[i] += 1.0;
    }
    a
}

#[test]
fn solves_against_dense_spd_reference() {
    let a = spd_fixture();
    let (col_ptr, row_idx, vals) = upper_csc(&a);
    let mut llt = SparseLlt::new(a.len(), col_ptr, row_idx);
    assert_eq!(llt.nnz(), vals.len());
    let b = [1.0, -2.0, 0.5, 3.0, -1.0, 4.0];
    for scale in [1.0, 10.0] {
        // Refactorise with new values on the same pattern.
        let scaled: Vec<f64> = vals.iter().map(|v| v * scale).collect();
        let mut x = b.to_vec();
        assert!(llt.factor_solve(&scaled, &mut x));
        for i in 0..a.len() {
            let ax: f64 = (0..a.len()).map(|k| scale * a[i][k] * x[k]).sum();
            assert!((ax - b[i]).abs() < 1e-12, "row {i}: {ax} vs {}", b[i]);
        }
    }
    // Diagonal-only column: x5 = b5 / A55.
    let mut x = b.to_vec();
    llt.factor_solve(&vals, &mut x);
    assert!((x[5] - 4.0).abs() < 1e-12);
}

#[test]
fn reports_non_positive_definite() {
    let a = vec![vec![1.0, 2.0], vec![2.0, 1.0]];
    let (col_ptr, row_idx, vals) = upper_csc(&a);
    let mut llt = SparseLlt::new(2, col_ptr, row_idx);
    let mut x = vec![1.0, 1.0];
    assert!(!llt.factor_solve(&vals, &mut x));
}

#[test]
fn empty_system_is_trivial() {
    let mut llt = SparseLlt::new(0, vec![0], Vec::new());
    let mut x: Vec<f64> = Vec::new();
    assert!(llt.factor_solve(&[], &mut x));
}
