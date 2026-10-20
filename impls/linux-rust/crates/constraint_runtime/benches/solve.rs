//! Hot-path benchmarks: Jacobian fill, structural solve, one drag step on the
//! staircase chain from `tests/common/staircase.rs`.

#[path = "../tests/common/staircase.rs"]
mod staircase;

use constraint_runtime::{
    AssembledSystem, DragSession, EndpointExprs, InteractiveSolve, LmSolver,
    Segment, SoftContribution, SoftTerms, SolverOptions, SolverPort, SystemEval,
};
use staircase::staircase;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

const SIZES: &[usize] = &[10, 30, 100, 500];

fn interactive<S: SolverPort>(solver: S) -> InteractiveSolve<S> {
    InteractiveSolve {
        solver,
        soft_weights: SoftTerms::default(),
        opts: SolverOptions::default(),
    }
}

fn bench_jacobian(c: &mut Criterion) {
    let mut group = c.benchmark_group("jacobian");
    for &n in SIZES {
        let (doc, _) = staircase(n);
        let soft: [SoftContribution; 0] = [];
        let sys = AssembledSystem::new(&doc, &soft);
        let p = doc.clone_vector();
        let mut j = sys.new_jacobian();
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| sys.jacobian(&p, &mut j))
        });
    }
    group.finish();
}

fn bench_structural<S: SolverPort + Clone>(c: &mut Criterion, name: &str, solver: S) {
    let mut group = c.benchmark_group(format!("structural/{name}"));
    group.sample_size(10);
    for &n in SIZES {
        let (doc, _) = staircase(n);
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter_batched(
                || doc.clone(),
                |mut d| {
                    let mut interact = interactive(solver.clone());
                    interact.structural_only(&mut d).expect("structural solve");
                },
                criterion::BatchSize::LargeInput,
            )
        });
    }
    group.finish();
}

fn bench_drag<S: SolverPort + Clone>(c: &mut Criterion, name: &str, solver: S) {
    let mut group = c.benchmark_group(format!("drag/{name}"));
    group.sample_size(10);
    for &n in SIZES {
        let (mut doc, ids) = staircase(n);
        let mut interact = interactive(solver.clone());
        interact.structural_only(&mut doc).expect("initial solve");
        let free = Segment::end_expr(*ids.last().unwrap());
        let q = free.evaluate(&doc, doc.vector());
        let mut drag = DragSession::new();
        drag.pointer_down(&doc, free);
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter_batched(
                || doc.clone(),
                |mut d| {
                    // A failed commit restores the fallback; both outcomes are timed.
                    let _ = drag.pointer_move(&mut interact, &mut d, (q.x + 0.3, q.y + 0.2));
                },
                criterion::BatchSize::LargeInput,
            )
        });
    }
    group.finish();
}

fn bench_lm(c: &mut Criterion) {
    bench_structural(c, "lm", LmSolver);
    bench_drag(c, "lm", LmSolver);
}

criterion_group!(benches, bench_jacobian, bench_lm);
criterion_main!(benches);
