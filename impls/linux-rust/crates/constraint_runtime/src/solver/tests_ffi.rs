use super::*;
use crate::assemble::AssembledSystem;
use crate::constraint::Constraint;
use crate::document::Document;
use crate::geom::{GeometryRole, Point2};
use crate::geometries::{GeometryBody, Segment};
use crate::state::State;
use std::os::raw::c_void;

#[test]
fn c_calling_pattern_fills_residual() {
    let mut doc = Document::new();
    let s0 = doc.add_geometry(
        GeometryRole::Result,
        GeometryBody::Segment,
        &Segment::params(Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)),
    );
    doc.add_constraint(Constraint::FixPoint {
        point: Segment::start_expr(s0),
        target: Point2::new(0.0, 0.0),
    });
    let soft: [crate::assemble::SoftContribution; 0] = [];
    let sys = AssembledSystem::from_document(&doc, &soft);
    let mut p = doc.state().clone();
    let handle = EvalHandle {
        eval: &sys,
        state: &mut p as *mut State,
    };
    let mut r = vec![0.0; sys.n_residuals()];
    let rc = unsafe {
        residual_trampoline(
            &handle as *const _ as *mut c_void,
            p.len(),
            p.as_ptr(),
            r.len(),
            r.as_mut_ptr(),
        )
    };
    assert_eq!(rc, 0);
    assert!(r.iter().all(|x| x.abs() < 1e-12));
}
