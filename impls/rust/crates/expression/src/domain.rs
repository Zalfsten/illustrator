//! Sound domain check over a closed range by interval bisection.

use crate::dual::Dual2;
use crate::error::DomainCheckError;
use crate::interval::Interval;
use crate::number::Number;
use crate::tree::{eval, Node};

/// Maximum bisection depth below the whole range.
const MAX_DEPTH: u32 = 32;

/// Sub-ranges narrower than this (relative to `max(1, |x|)`) are not split.
const MIN_RELATIVE_WIDTH: f64 = 1e-12;

/// Proves value, first and second derivative finite on `[x0, x1]`, or
/// reports the first sub-range (depth first, left to right) where the proof
/// failed.
pub(crate) fn check_domain(nodes: &[Node], x0: f64, x1: f64) -> Result<(), DomainCheckError> {
    if !(x0.is_finite() && x1.is_finite() && x0 < x1) {
        return Err(DomainCheckError::InvalidRange);
    }
    let mut scratch: Vec<Dual2<Interval>> = Vec::with_capacity(nodes.len());
    let mut stack = vec![(x0, x1, 0u32)];
    while let Some((lo, hi, depth)) = stack.pop() {
        let (r, bad) = eval(nodes, Dual2::variable(Interval::new(lo, hi)), &mut scratch);
        if !r.is_bad() {
            continue;
        }
        let mid = 0.5 * (lo + hi);
        let narrow = hi - lo < MIN_RELATIVE_WIDTH * mid.abs().max(1.0);
        if depth == MAX_DEPTH || narrow || mid <= lo || mid >= hi {
            return Err(DomainCheckError::NotProven {
                range: (lo, hi),
                function: bad.and_then(|i| nodes[i].name()),
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
