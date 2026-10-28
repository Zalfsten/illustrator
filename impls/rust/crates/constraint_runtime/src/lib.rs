//! Model E constraint runtime: global state vector, connectivity assembly,
//! analytical Jacobians, and a swappable solver port (owned sparse
//! Levenberg–Marquardt backend, pure Rust).

pub mod assemble;
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

#[cfg(test)]
mod test_alloc;

pub use assemble::{
    structural_residual_norm, AssembledSystem, SoftContribution, SoftLengthFloor, SoftPointFollow,
};
pub use constraint::{ConstraintEntry, ConstraintId, ConstraintKind};
pub use constraint_kinds::{
    Congruence, ConstraintInst, EqualLength, FixPoint, VectorsOrthogonal, VectorsParallel,
};
pub use document::Document;
pub use expr::{
    Expr, HeadingAngleExpr, LengthExpr, PointExpr, TurnAngleExpr, VecExpr, VecJet,
    MAX_FORMULA_LOCAL_DOF,
};
pub use geom::{CurveParam, HeadingAngle, Length, Point2, TurnAngle, Vec2};
pub use geometry::{
    Caps, Chart, ChartSlot, CircularExprs, CurveExprs, EndpointExprs, Geometry, GeometryCaps,
    GeometryId, GeometryRole, GeometryShell, LineExprs, StateAtom,
};
pub use geometry_kinds::{Arc, GeometryInst, GeometryKind, Segment};
pub use id::IdMap;
pub use interact::{DragSession, InteractiveSolve, SoftTerms, SolveError};
pub use matrix::{Block, BlockSparseMatrix, DenseMatrix};
pub use mesh::{Mesh, MeshEdge, MeshFilter, MAX_CHORD_TURN};
pub use solver::{LmSolver, SolveOutcome, SolverOptions, SolverPort, StubSolver, SystemEval};
pub use vector::Vector;

pub const SOLVE_TOL: f64 = 1e-6;
