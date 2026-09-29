//! Model E constraint runtime: global state vector, connectivity assembly,
//! analytical Jacobians, and a swappable solver port (owned LM + Ceres).

pub mod assemble;
pub mod ceres_solver;
pub mod connectivity;
pub mod constraint;
pub mod document;
pub mod expr;
pub mod geom;
pub mod geometries;
pub mod interact;
pub mod mesh;
pub mod solver;
pub mod store;

pub use assemble::{AssembledSystem, SoftContribution, structural_residual_norm};
pub use ceres_solver::CeresSolver;
pub use connectivity::Connectivity;
pub use constraint::Constraint;
pub use document::{Document, DocumentStructure, State};
pub use expr::{PointExpr, VecExpr};
pub use geom::{
    CurveParam, Geometry, GeometryBody, GeometryId, GeometryRole, Point2, Vec2,
};
pub use geometries::Segment;
pub use store::GeometryStore;
pub use interact::{DragSession, InteractiveSolve, SoftTerms, SolveError};
pub use mesh::{Mesh, MeshEdge, MeshFilter};
pub use solver::{
    LmSolver, SolveOutcome, SolverOptions, SolverPort, StubSolver, SystemEval, ffi,
};

pub const SOLVE_TOL: f64 = 1e-6;
