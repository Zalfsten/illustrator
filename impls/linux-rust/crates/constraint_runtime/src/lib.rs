//! Model E constraint runtime: global state vector, connectivity assembly,
//! analytical Jacobians, and a swappable solver port (owned LM + Ceres).

pub mod assemble;
pub mod ceres_solver;
pub mod connectivity;
pub mod constraint;
pub mod document;
pub mod geom;
pub mod geometry;
pub mod geometry_kinds;
pub mod interact;
pub mod mapper;
pub mod mesh;
pub mod solver;
pub mod state;
pub mod store;

pub use assemble::{structural_residual_norm, AssembledSystem, SoftContribution};
pub use ceres_solver::CeresSolver;
pub use connectivity::Connectivity;
pub use constraint::Constraint;
pub use document::{Document, DocumentStructure};
pub use geom::{
    Angle, CurveParam, Length, Point2, PointJacobian, ScalarJacobian, Vec2, VecJacobian,
    MAX_POINT_JACOBIAN_PARAMS, MAX_SCALAR_JACOBIAN_PARAMS,
};
pub use geometry::{
    AsShell, Chart, ChartSlot, CurveExprs, EndpointExprs, Geometry, GeometryId, GeometryKind,
    GeometryRole, GeometryShell, IdGenerator, LengthExpr, PointExpr, StateAtom, VecExpr,
};
pub use geometry_kinds::Segment;
pub use interact::{DragSession, InteractiveSolve, SoftTerms, SolveError};
pub use mapper::Mapper;
pub use mesh::{Mesh, MeshEdge, MeshFilter};
pub use solver::{ffi, LmSolver, SolveOutcome, SolverOptions, SolverPort, StubSolver, SystemEval};
pub use state::State;
pub use store::GeometryStore;

pub const SOLVE_TOL: f64 = 1e-6;
