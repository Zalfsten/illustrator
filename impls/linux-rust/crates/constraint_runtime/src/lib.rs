//! Model E constraint runtime: global state vector, connectivity assembly,
//! analytical Jacobians, and a swappable solver port (owned LM + Ceres).

pub mod assemble;
pub mod ceres_solver;
pub mod constraint;
pub mod constraint_kinds;
pub mod document;
pub mod geom;
pub mod geometry;
pub mod geometry_kinds;
pub mod id;
pub mod interact;
pub mod mapper;
pub mod matrix;
pub mod mesh;
pub mod solver;
pub mod store;
pub mod vector;

pub use assemble::{structural_residual_norm, AssembledSystem, SoftContribution};
pub use ceres_solver::CeresSolver;
pub use constraint::{ConstraintEntry, ConstraintId, ConstraintKind};
pub use constraint_kinds::{
    Congruence, ConstraintInst, EqualLength, FixPoint, Orthogonal, Parallel,
};
pub use document::Document;
pub use geom::{Angle, CurveParam, Length, Point2, Vec2};
pub use geometry::{
    AsShell, Chart, ChartSlot, CurveExprs, EndpointExprs, GeometryId, GeometryKind, GeometryRole,
    GeometryShell, LengthExpr, PointExpr, StateAtom, VecExpr,
};
pub use geometry_kinds::{GeometryInst, InGeometryCatalog, Segment};
pub use id::IdGenerator;
pub use interact::{DragSession, InteractiveSolve, SoftTerms, SolveError};
pub use mapper::Mapper;
pub use matrix::{DenseMatrix, MatrixWrite, Matrix, MatrixView, ScatterMatrix};
pub use mesh::{Mesh, MeshEdge, MeshFilter};
pub use solver::{ffi, LmSolver, SolveOutcome, SolverOptions, SolverPort, StubSolver, SystemEval};
pub use store::GeometryStore;
pub use vector::{DenseVec, ScatterVec, VecWrite, VecView, Vector};

pub const SOLVE_TOL: f64 = 1e-6;
