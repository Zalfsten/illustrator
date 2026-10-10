//! Model E constraint runtime: global state vector, connectivity assembly,
//! analytical Jacobians, and a swappable solver port (owned LM + Ceres).

pub mod assemble;
pub mod ceres_solver;
pub mod constraint;
pub mod constraint_kinds;
pub mod document;
pub mod expr;
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
pub use expr::{
    HeadingAngleExpr, LengthExpr, PointExpr, TurnAngleExpr, VecExpr, MAX_FORMULA_LOCAL_DOF,
};
pub use geom::{CurveParam, HeadingAngle, Length, Point2, TurnAngle, Vec2};
pub use geometry::{
    AsShell, Chart, ChartSlot, CurveExprs, EndpointExprs, GeometryId, GeometryKind, GeometryRole,
    GeometryShell, StateAtom,
};
pub use geometry_kinds::{Arc, GeometryInst, InGeometryCatalog, Segment};
pub use id::IdGenerator;
pub use interact::{DragSession, InteractiveSolve, SoftTerms, SolveError};
pub use mapper::Mapper;
pub use matrix::{DenseMatrix, MatrixAccess, Matrix, MatrixView, ScatterMatrix};
pub use mesh::{Mesh, MeshEdge, MeshFilter};
pub use solver::{ffi, LmSolver, SolveOutcome, SolverOptions, SolverPort, StubSolver, SystemEval};
pub use store::GeometryStore;
pub use vector::{DenseVec, ScatterVec, VecAccess, VecView, Vector};

pub const SOLVE_TOL: f64 = 1e-6;
