//! Model E constraint runtime: global state vector, connectivity assembly,
//! analytical Jacobians, and a swappable solver port (owned LM + Ceres).

pub mod assemble;
pub mod ceres_solver;
pub mod connectivity;
pub mod constraint;
pub mod document;
pub mod geom;
pub mod interact;
pub mod mesh;
pub mod solver;

pub use assemble::{AssembledSystem, SoftContribution, structural_residual_norm};
pub use ceres_solver::CeresSolver;
pub use connectivity::Connectivity;
pub use constraint::Constraint;
pub use document::{Document, DocumentStructure, State};
pub use geom::{
    Curve, CurveParam, EndpointKind, Geometry, GeometryId, GeometryRole, GeometryStore, Point2,
    PointExpr, Segment, SegmentId, Vec2, VecExpr, SEGMENT_STRIDE,
};
pub use interact::{DragSession, InteractiveSolve, SoftTerms, SolveError};
pub use mesh::{Mesh, MeshEdge, MeshFilter};
pub use solver::{
    LmSolver, SolveOutcome, SolverOptions, SolverPort, StubSolver, SystemEval, ffi,
};

pub const SOLVE_TOL: f64 = 1e-6;
