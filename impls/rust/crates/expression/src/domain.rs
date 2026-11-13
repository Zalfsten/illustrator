//! Sound domain check over a closed range by interval bisection.

use crate::dual::Dual2;
use crate::error::DomainCheckError;
use crate::interval::Interval;
use crate::tree::{eval, Node};

/// Maximum bisection depth below the whole range.
const MAX_DEPTH: u32 = 32;

/// Sub-ranges narrower than this (relative to `max(1, |x|)`) are not split.
const MIN_RELATIVE_WIDTH: f64 = 1e-12;

/// Interval evaluations per check. Proven sub-ranges count too: overestimation
/// (`log(cosh(x) - sinh(x))`) can need millions of them, and the check runs on
/// user input.
pub(crate) const MAX_EVALUATIONS: usize = 20_000;

/// Proves value, first and second derivative finite on `[x0, x1]`, or
/// reports the first sub-range (depth first, left to right) where the proof
/// failed, or the part not yet proven when [`MAX_EVALUATIONS`] is spent.
///
/// Like [`crate::Expression::eval`], a sub-range fails when any node is bad,
/// not only the result: `log(x)^0` must not pass where `log` is undefined.
pub(crate) fn check_domain(nodes: &[Node], x0: f64, x1: f64) -> Result<(), DomainCheckError> {
    if !(x0.is_finite() && x1.is_finite() && x0 < x1) {
        return Err(DomainCheckError::InvalidRange);
    }
    let mut scratch: Vec<Dual2<Interval>> = Vec::with_capacity(nodes.len());
    let mut stack = vec![(x0, x1, 0u32)];
    let mut evaluations = 0;
    while let Some((lo, hi, depth)) = stack.pop() {
        if evaluations == MAX_EVALUATIONS {
            return Err(DomainCheckError::TooComplex { range: (lo, x1) });
        }
        evaluations += 1;
        let (_, bad) = eval(nodes, Dual2::variable(Interval::new(lo, hi)), &mut scratch);
        let Some(bad) = bad else {
            continue;
        };
        let mid = 0.5 * (lo + hi);
        let narrow = hi - lo < MIN_RELATIVE_WIDTH * mid.abs().max(1.0);
        if depth == MAX_DEPTH || narrow || mid <= lo || mid >= hi {
            return Err(DomainCheckError::NotProven {
                range: (lo, hi),
                function: nodes[bad].name(),
            });
        }
        stack.push((mid, hi, depth + 1));
        stack.push((lo, mid, depth + 1));
    }
    Ok(())
}

#[cfg(test)]
#[path = "tests_domain.rs"]
mod tests_domain;
