//! Kind-owned chart: State layout (`pack` / `unpack`) and named local slots.

use std::ops::Range;

use crate::geom::{Point2, Vec2};

/// Packable atom in the global State vector: fixed DOF width and `[x, y]` flatten order.
///
/// Chart `pack` / `unpack` and local parameter indices go through this trait so
/// component order lives with the layout layer, not re-stated at each kind call site.
pub trait StateAtom: Sized + Copy {
    /// Number of contiguous State scalars for one atom.
    const DOF_COUNT: usize;
    /// Component index of \(x\) within a packed atom (`0`).
    const COMP_X: usize = 0;
    /// Component index of \(y\) within a packed atom (`1`).
    const COMP_Y: usize = 1;

    /// Write this atom into `out` (length must be [`Self::DOF_COUNT`]).
    fn write_params(self, out: &mut [f64]);

    /// Read an atom from a packed slice (length must be [`Self::DOF_COUNT`]).
    fn read_params(params: &[f64]) -> Self;
}

impl StateAtom for Vec2 {
    const DOF_COUNT: usize = 2;

    fn write_params(self, out: &mut [f64]) {
        debug_assert_eq!(out.len(), Self::DOF_COUNT);
        out[Self::COMP_X] = self.x;
        out[Self::COMP_Y] = self.y;
    }

    fn read_params(params: &[f64]) -> Self {
        debug_assert_eq!(params.len(), Self::DOF_COUNT);
        Self::new(params[Self::COMP_X], params[Self::COMP_Y])
    }
}

impl StateAtom for Point2 {
    const DOF_COUNT: usize = 2;

    fn write_params(self, out: &mut [f64]) {
        debug_assert_eq!(out.len(), Self::DOF_COUNT);
        out[Self::COMP_X] = self.x;
        out[Self::COMP_Y] = self.y;
    }

    fn read_params(params: &[f64]) -> Self {
        debug_assert_eq!(params.len(), Self::DOF_COUNT);
        Self::new(params[Self::COMP_X], params[Self::COMP_Y])
    }
}

/// Contiguous local DOF window for one [`StateAtom`] inside a chart parameter block.
///
/// Slot offsets define pack order and formula-local index ranges. Kinds name
/// slots with [`ChartSlot::for_atom`] / [`ChartSlot::then`]; pack / unpack come
/// from [`AtomTuple`] via [`Chart`] defaults.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ChartSlot {
    offset: usize,
    dof: usize,
}

impl ChartSlot {
    /// Slot for atom type `A` starting at local index `offset`.
    pub const fn for_atom<A: StateAtom>(offset: usize) -> Self {
        Self {
            offset,
            dof: A::DOF_COUNT,
        }
    }

    /// Next slot for atom type `A`, contiguous after `self`.
    pub const fn then<A: StateAtom>(self) -> Self {
        Self::for_atom::<A>(self.end())
    }

    pub const fn offset(self) -> usize {
        self.offset
    }

    pub const fn dof(self) -> usize {
        self.dof
    }

    /// Local index immediately after this slot (start of the next atom).
    pub const fn end(self) -> usize {
        self.offset + self.dof
    }

    pub const fn range(self) -> Range<usize> {
        self.offset..self.offset + self.dof
    }

    /// Global-within-chart index of component `comp` (e.g. [`Point2::COMP_X`](StateAtom::COMP_X)).
    pub const fn component(self, comp: usize) -> usize {
        self.offset + comp
    }

    /// Formula-local parameter indices covered by this slot.
    pub fn indices(self) -> Vec<usize> {
        self.range().collect()
    }

    /// Concatenate indices of several slots in declaration order.
    pub fn union_indices(slots: &[ChartSlot]) -> Vec<usize> {
        let mut out = Vec::new();
        for &s in slots {
            out.extend(s.indices());
        }
        out
    }
}

/// Ordered [`StateAtom`] tuple used as a chart's [`Chart::Init`].
///
/// Provides flatten width and pack / unpack; [`Chart`] defaults delegate here.
pub trait AtomTuple: Sized + Copy {
    const DOF_COUNT: usize;

    fn pack(self) -> Vec<f64>;

    fn unpack(params: &[f64]) -> Self;
}

impl<A: StateAtom, B: StateAtom> AtomTuple for (A, B) {
    const DOF_COUNT: usize = A::DOF_COUNT + B::DOF_COUNT;

    fn pack(self) -> Vec<f64> {
        let (a, b) = self;
        let n_a = A::DOF_COUNT;
        let mut buf = vec![0.0; n_a + B::DOF_COUNT];
        a.write_params(&mut buf[..n_a]);
        b.write_params(&mut buf[n_a..]);
        buf
    }

    fn unpack(params: &[f64]) -> Self {
        debug_assert_eq!(params.len(), Self::DOF_COUNT);
        let n_a = A::DOF_COUNT;
        (
            A::read_params(&params[..n_a]),
            B::read_params(&params[n_a..]),
        )
    }
}

/// Geometry kind as a chart: packs init into State.
///
/// Document identity and shared shell ops live on [`super::Geometry`] (object-safe).
/// Concrete kinds call [`crate::geometry::shell::geometry_shell_instance`] so they
/// satisfy [`super::GeometryKind`] (and thus the [`super::Geometry`] blanket).
///
/// Layout: set [`Self::Init`] to an [`AtomTuple`] and declare named [`ChartSlot`]s;
/// [`PARAMETER_COUNT`] / `pack` / `unpack` default from the tuple.
pub trait Chart {
    /// Kind-specific construction inputs (e.g. segment endpoints).
    type Init: AtomTuple;

    /// Number of contiguous State scalars for one instance of this kind.
    const PARAMETER_COUNT: usize = Self::Init::DOF_COUNT;

    /// Contiguous initial parameters (length = [`Self::PARAMETER_COUNT`]).
    fn pack(init: Self::Init) -> impl AsRef<[f64]> {
        init.pack()
    }

    /// Inverse of [`Self::pack`].
    fn unpack(params: &[f64]) -> Self::Init {
        Self::Init::unpack(params)
    }

    /// Global index window for a block that would start at `offset`.
    fn param_block_at(offset: usize) -> Range<usize> {
        offset..offset + Self::PARAMETER_COUNT
    }
}

#[cfg(test)]
#[path = "tests_chart.rs"]
mod tests_chart;
