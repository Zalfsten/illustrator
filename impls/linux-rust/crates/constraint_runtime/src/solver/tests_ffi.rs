use super::*;
use crate::assemble::AssembledSystem;
use crate::constraint_kinds::FixPoint;
use crate::document::Document;
use crate::geom::Point2;
use crate::geometry::{EndpointExprs, GeometryRole};
use crate::geometry_kinds::Segment;
use crate::vector::Vector;
use std::os::raw::c_void;

#[test]
fn c_calling_pattern_fills_residual() {
    let mut doc = Document::new();
    let s0 = doc.add_geometry::<Segment>(
        GeometryRole::Result,
        (Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)),
    );
    doc.add_constraint(FixPoint {
        point: Segment::start_expr(s0),
        target: Point2::new(0.0, 0.0),
    });
    let soft: [crate::assemble::SoftContribution; 0] = [];
    let sys = AssembledSystem::from_document(&doc, &soft);
    let mut p = doc.vector().clone();
    let handle = EvalHandle {
        eval: &sys,
        state: &mut p as *mut Vector,
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
