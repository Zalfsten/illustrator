//! Model E constraint runtime: global state vector, connectivity assembly,
//! analytical Jacobians, and a swappable solver port.

pub mod assemble;
pub mod connectivity;
pub mod constraint;
pub mod document;
pub mod geom;
pub mod interact;
pub mod solver;

pub use assemble::{AssembledSystem, SoftContribution, structural_residual_norm};
pub use connectivity::Connectivity;
pub use constraint::{Constraint, EndpointRef, SegmentEnd};
pub use document::{Document, DocumentStructure, State};
pub use geom::{Point2, PointExpr, Segment, SegmentEndKind, SegmentId, SEGMENT_STRIDE};
pub use interact::{DragSession, InteractiveSolve, SoftTerms, SolveError};
pub use solver::{
    LmSolver, SolveOutcome, SolverOptions, SolverPort, StubSolver, SystemEval, ffi,
};

pub const SOLVE_TOL: f64 = 1e-6;
