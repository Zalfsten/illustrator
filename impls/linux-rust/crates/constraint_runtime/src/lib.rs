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
pub mod matrix;
pub mod mesh;
pub mod solver;
pub mod vector;

pub use assemble::{
    structural_residual_norm, AssembledSystem, SoftContribution, SoftLengthFloor, SoftPointFollow,
};
pub use ceres_solver::CeresSolver;
pub use constraint::{ConstraintEntry, ConstraintId, ConstraintKind};
pub use constraint_kinds::{
    Congruence, ConstraintInst, EqualLength, FixPoint, Orthogonal, Parallel,
};
pub use document::Document;
pub use expr::{
    Expr, HeadingAngleExpr, LengthExpr, PointExpr, TurnAngleExpr, VecExpr, VecJet,
    MAX_FORMULA_LOCAL_DOF,
};
pub use geom::{CurveParam, HeadingAngle, Length, Point2, TurnAngle, Vec2};
pub use geometry::{
    Chart, ChartSlot, CurveExprs, EndpointExprs, Geometry, GeometryId, GeometryRole,
    GeometryShell, StateAtom,
};
pub use geometry_kinds::{Arc, GeometryInst, GeometryKind, Segment};
pub use id::IdMap;
pub use interact::{DragSession, InteractiveSolve, SoftTerms, SolveError};
pub use matrix::{DenseMatrix, Matrix, MatrixAccess, ScatterMatrix};
pub use mesh::{Mesh, MeshEdge, MeshFilter, MAX_CHORD_TURN};
pub use solver::{ffi, LmSolver, SolveOutcome, SolverOptions, SolverPort, StubSolver, SystemEval};
pub use vector::Vector;

pub const SOLVE_TOL: f64 = 1e-6;
