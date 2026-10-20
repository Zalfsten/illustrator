use super::*;
use crate::matrix::BlockSparseMatrix;

/// Dense GᵀG / Jᵀr reference from `add_to_dense`.
fn dense_reference(j: &BlockSparseMatrix, r: &Vector) -> (Vec<f64>, Vec<f64>) {
    let (m, n) = (j.nrows(), j.ncols());
    let mut d = vec![0.0; m * n];
    j.add_to_dense(&mut d);
    let mut g = vec![0.0; n * n];
    let mut jtr = vec![0.0; n];
    for a in 0..n {
        for row in 0..m {
            jtr[a] += d[row * n + a] * r[row];
            for c in 0..n {
                g[a * n + c] += d[row * n + a] * d[row * n + c];
            }
        }
    }
    (g, jtr)
}

fn check(j: &BlockSparseMatrix, r: &Vector) {
    let n = j.ncols();
    let mut ns = NormalSystem::new(j);
    ns.assemble(j, r);
    let (g, jtr) = dense_reference(j, r);
    for (a, (got, want)) in ns.rhs().iter().zip(&jtr).enumerate() {
        assert!((got - want).abs() < 1e-12, "g[{a}]: {got} vs {want}");
    }
    let mut seen = vec![0.0; n * n];
    for c in 0..n {
        for s in ns.col_ptr()[c]..ns.col_ptr()[c + 1] {
            let r_ = ns.row_idx()[s];
            assert!(r_ <= c, "pattern is upper triangular");
            seen[r_ * n + c] = ns.values()[s];
        }
        assert_eq!(ns.row_idx()[ns.diag_slot(c)], c);
    }
    for a in 0..n {
        for c in a..n {
            let want = g[a * n + c];
            assert!(
                (seen[a * n + c] - want).abs() < 1e-12,
                "G[{a},{c}]: {} vs {want}",
                seen[a * n + c]
            );
        }
    }
}

fn fill(j: &mut BlockSparseMatrix, seed: f64) {
    for bi in 0..j.num_blocks() {
        let (m, k) = (j.block(bi).nrows(), j.block(bi).ncols());
        let mut v = j.block_mut(bi);
        for i in 0..m {
            for k_ in 0..k {
                v[(i, k_)] = ((bi * 7 + i * 3 + k_) as f64 * seed).sin();
            }
        }
    }
}

#[test]
fn matches_dense_normal_equations() {
    let mut j = BlockSparseMatrix::new(6);
    j.push_block(2, &[0, 1, 4]);
    j.push_block(1, &[3, 2]);
    j.push_block(2, &[4, 1]);
    // Column 5 is untouched: only its diagonal slot exists.
    fill(&mut j, 0.37);
    let r = Vector::from(vec![0.5, -1.0, 2.0, 0.25, -0.75]);
    check(&j, &r);
}

#[test]
fn duplicate_columns_accumulate() {
    let mut j = BlockSparseMatrix::new(3);
    j.push_block(2, &[2, 0, 2, 1]);
    fill(&mut j, 0.91);
    let r = Vector::from(vec![1.5, -0.5]);
    check(&j, &r);
}

#[test]
fn assemble_and_factor_solve_do_not_allocate() {
    use crate::solver::sparse_chol::SparseLlt;
    use crate::test_alloc::allocations;

    let mut j = BlockSparseMatrix::new(40);
    for b in 0..30 {
        j.push_block(2, &[b, b + 1, b + 5, (b * 7) % 40]);
    }
    fill(&mut j, 0.53);
    let r = Vector::from(vec![0.3; j.nrows()]);
    let mut ns = NormalSystem::new(&j);
    let mut llt = SparseLlt::new(ns.n(), ns.col_ptr().to_vec(), ns.row_idx().to_vec());
    let mut damped = vec![0.0; ns.values().len()];
    let mut x = vec![0.0; ns.n()];

    let mut step = |ns: &mut NormalSystem| {
        ns.assemble(&j, &r);
        damped.copy_from_slice(ns.values());
        for i in 0..ns.n() {
            damped[ns.diag_slot(i)] += 1.0;
        }
        x.copy_from_slice(ns.rhs());
        assert!(llt.factor_solve(&damped, &mut x));
    };
    step(&mut ns);
    let n = allocations(|| {
        for _ in 0..3 {
            step(&mut ns);
        }
    });
    assert_eq!(n, 0, "normal assembly + sparse LLᵀ allocated {n} times");
}
