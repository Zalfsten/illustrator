//! Kind-owned chart: Vector layout (`pack` / `unpack`) and named local slots.

use std::ops::Range;

use crate::geom::{Angle, Length, Point2, Vec2};

/// Packable atom in the global parameter vector: fixed DOF width and flatten order.
///
/// Chart `pack` / `unpack` and local parameter indices go through this trait so
/// component order lives with the layout layer, not re-stated at each kind call site.
/// Planar atoms use [`Point2::COMP_X`] / [`Point2::COMP_Y`] (same on [`Vec2`]).
pub trait StateAtom: Sized + Copy {
    /// Number of contiguous Vector scalars for one atom.
    const DOF_COUNT: usize;

    /// Write this atom into `out` (length must be [`Self::DOF_COUNT`]).
    fn write_params(self, out: &mut [f64]);

    /// Read an atom from a packed slice (length must be [`Self::DOF_COUNT`]).
    fn read_params(params: &[f64]) -> Self;
}

impl StateAtom for Vec2 {
    const DOF_COUNT: usize = 2;

    fn write_params(self, out: &mut [f64]) {
        debug_assert_eq!(out.len(), Self::DOF_COUNT);
        out[Vec2::COMP_X] = self.x;
        out[Vec2::COMP_Y] = self.y;
    }

    fn read_params(params: &[f64]) -> Self {
        debug_assert_eq!(params.len(), Self::DOF_COUNT);
        Self::new(params[Vec2::COMP_X], params[Vec2::COMP_Y])
    }
}

impl StateAtom for Point2 {
    const DOF_COUNT: usize = 2;

    fn write_params(self, out: &mut [f64]) {
        debug_assert_eq!(out.len(), Self::DOF_COUNT);
        out[Point2::COMP_X] = self.x;
        out[Point2::COMP_Y] = self.y;
    }

    fn read_params(params: &[f64]) -> Self {
        debug_assert_eq!(params.len(), Self::DOF_COUNT);
        Self::new(params[Point2::COMP_X], params[Point2::COMP_Y])
    }
}

impl StateAtom for Angle {
    const DOF_COUNT: usize = 1;

    fn write_params(self, out: &mut [f64]) {
        debug_assert_eq!(out.len(), Self::DOF_COUNT);
        out[0] = self.radians();
    }

    fn read_params(params: &[f64]) -> Self {
        debug_assert_eq!(params.len(), Self::DOF_COUNT);
        Self::new(params[0])
    }
}

impl StateAtom for Length {
    const DOF_COUNT: usize = 1;

    fn write_params(self, out: &mut [f64]) {
        debug_assert_eq!(out.len(), Self::DOF_COUNT);
        out[0] = self.value();
    }

    fn read_params(params: &[f64]) -> Self {
        debug_assert_eq!(params.len(), Self::DOF_COUNT);
        Self::new(params[0])
    }
}

/// Pack / unpack a fixed Init tuple of [`StateAtom`]s into a contiguous chart block.
///
/// Used by [`chart_atoms!`] so the macro only names slots; flatten order follows the tuple.
pub(crate) trait PackAtoms: Sized {
    const DOF_COUNT: usize;
    fn write_all(self, out: &mut [f64]);
    fn read_all(params: &[f64]) -> Self;
}

impl<A: StateAtom> PackAtoms for (A,) {
    const DOF_COUNT: usize = A::DOF_COUNT;

    fn write_all(self, out: &mut [f64]) {
        debug_assert_eq!(out.len(), Self::DOF_COUNT);
        self.0.write_params(out);
    }

    fn read_all(params: &[f64]) -> Self {
        debug_assert_eq!(params.len(), Self::DOF_COUNT);
        (A::read_params(params),)
    }
}

impl<A: StateAtom, B: StateAtom> PackAtoms for (A, B) {
    const DOF_COUNT: usize = A::DOF_COUNT + B::DOF_COUNT;

    fn write_all(self, out: &mut [f64]) {
        debug_assert_eq!(out.len(), Self::DOF_COUNT);
        let (a, b) = self;
        a.write_params(&mut out[..A::DOF_COUNT]);
        b.write_params(&mut out[A::DOF_COUNT..]);
    }

    fn read_all(params: &[f64]) -> Self {
        debug_assert_eq!(params.len(), Self::DOF_COUNT);
        (
            A::read_params(&params[..A::DOF_COUNT]),
            B::read_params(&params[A::DOF_COUNT..]),
        )
    }
}

impl<A: StateAtom, B: StateAtom, C: StateAtom> PackAtoms for (A, B, C) {
    const DOF_COUNT: usize = A::DOF_COUNT + B::DOF_COUNT + C::DOF_COUNT;

    fn write_all(self, out: &mut [f64]) {
        debug_assert_eq!(out.len(), Self::DOF_COUNT);
        let (a, b, c) = self;
        let i1 = A::DOF_COUNT;
        let i2 = i1 + B::DOF_COUNT;
        a.write_params(&mut out[..i1]);
        b.write_params(&mut out[i1..i2]);
        c.write_params(&mut out[i2..]);
    }

    fn read_all(params: &[f64]) -> Self {
        debug_assert_eq!(params.len(), Self::DOF_COUNT);
        let i1 = A::DOF_COUNT;
        let i2 = i1 + B::DOF_COUNT;
        (
            A::read_params(&params[..i1]),
            B::read_params(&params[i1..i2]),
            C::read_params(&params[i2..]),
        )
    }
}

impl<A: StateAtom, B: StateAtom, C: StateAtom, D: StateAtom> PackAtoms for (A, B, C, D) {
    const DOF_COUNT: usize = A::DOF_COUNT + B::DOF_COUNT + C::DOF_COUNT + D::DOF_COUNT;

    fn write_all(self, out: &mut [f64]) {
        debug_assert_eq!(out.len(), Self::DOF_COUNT);
        let (a, b, c, d) = self;
        let i1 = A::DOF_COUNT;
        let i2 = i1 + B::DOF_COUNT;
        let i3 = i2 + C::DOF_COUNT;
        a.write_params(&mut out[..i1]);
        b.write_params(&mut out[i1..i2]);
        c.write_params(&mut out[i2..i3]);
        d.write_params(&mut out[i3..]);
    }

    fn read_all(params: &[f64]) -> Self {
        debug_assert_eq!(params.len(), Self::DOF_COUNT);
        let i1 = A::DOF_COUNT;
        let i2 = i1 + B::DOF_COUNT;
        let i3 = i2 + C::DOF_COUNT;
        (
            A::read_params(&params[..i1]),
            B::read_params(&params[i1..i2]),
            C::read_params(&params[i2..i3]),
            D::read_params(&params[i3..]),
        )
    }
}

/// Contiguous local DOF window for one [`StateAtom`] inside a chart parameter block.
///
/// Slot offsets define pack order and formula-local index ranges. Prefer
/// [`chart_atoms!`] so kinds only name atoms; pack / unpack / [`SLOTS`] follow.
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

    /// Local index immediately after this slot (start of the next atom).
    pub const fn end(self) -> usize {
        self.offset + self.dof
    }

    pub const fn range(self) -> Range<usize> {
        self.offset..self.offset + self.dof
    }

    /// Global-within-chart index of component `comp` (e.g. [`Point2::COMP_X`]).
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

/// Geometry kind as a chart: packs init into Vector.
///
/// Document identity and shared shell ops live on [`super::Geometry`] (object-safe).
/// Concrete kinds call [`crate::geometry::shell::geometry_shell_instance`] so they
/// satisfy [`super::GeometryKind`] (and thus the [`super::Geometry`] blanket).
///
/// Prefer [`chart_atoms!`] to declare named [`StateAtom`] slots and derive
/// [`PARAMETER_COUNT`] / `pack` / `unpack`.
pub trait Chart {
    /// Kind-specific construction inputs (e.g. segment endpoints).
    type Init;

    /// Number of contiguous Vector scalars for one instance of this kind.
    const PARAMETER_COUNT: usize;

    /// Contiguous initial parameters (length = [`Self::PARAMETER_COUNT`]).
    fn pack(init: Self::Init) -> impl AsRef<[f64]>;

    /// Inverse of [`Self::pack`].
    fn unpack(params: &[f64]) -> Self::Init;

    /// Global index window for a block that would start at `offset`.
    fn param_block_at(offset: usize) -> Range<usize> {
        offset..offset + Self::PARAMETER_COUNT
    }
}

/// Declare named chart atoms; derive slots, [`SLOTS`](ChartSlot), and [`Chart`] pack/unpack.
///
/// Pack / unpack / [`PARAMETER_COUNT`](Chart::PARAMETER_COUNT) come from [`PackAtoms`] on
/// the Init tuple; this macro only names contiguous [`ChartSlot`]s.
///
/// ```ignore
/// chart_atoms! {
///     Segment {
///         start: Point2,
///         end: Point2,
///     }
/// }
/// ```
macro_rules! chart_atoms {
    ($ty:ty {
        $first:ident : $first_atom:ty
        $(, $name:ident : $atom:ty)* $(,)?
    }) => {
        impl $ty {
            #[allow(non_upper_case_globals)]
            pub const $first: $crate::geometry::ChartSlot =
                $crate::geometry::ChartSlot::for_atom::<$first_atom>(0);
        }
        $crate::geometry::chart::chart_atoms!(@chain_slots $ty; $first; $($name: $atom),*);

        impl $ty {
            /// All chart slots in declaration order (layout metadata only).
            pub const SLOTS: &'static [$crate::geometry::ChartSlot] =
                &[Self::$first $(, Self::$name)*];
        }

        impl $crate::geometry::Chart for $ty {
            // Trailing comma keeps a single atom as a 1-tuple for [`PackAtoms`].
            type Init = ($first_atom, $($atom,)*);

            const PARAMETER_COUNT: usize =
                <Self::Init as $crate::geometry::chart::PackAtoms>::DOF_COUNT;

            fn pack(init: Self::Init) -> impl AsRef<[f64]> {
                let mut buf = [0.0; Self::PARAMETER_COUNT];
                $crate::geometry::chart::PackAtoms::write_all(init, &mut buf);
                buf
            }

            fn unpack(params: &[f64]) -> Self::Init {
                debug_assert_eq!(params.len(), Self::PARAMETER_COUNT);
                <Self::Init as $crate::geometry::chart::PackAtoms>::read_all(params)
            }
        }
    };

    (@chain_slots $ty:ty; $_prev:ident; ) => {};
    (@chain_slots $ty:ty; $prev:ident; $name:ident : $atom:ty $(, $rest_name:ident : $rest_atom:ty)*) => {
        impl $ty {
            #[allow(non_upper_case_globals)]
            pub const $name: $crate::geometry::ChartSlot =
                $crate::geometry::ChartSlot::for_atom::<$atom>(Self::$prev.end());
        }
        $crate::geometry::chart::chart_atoms!(@chain_slots $ty; $name; $($rest_name: $rest_atom),*);
    };
}
pub(crate) use chart_atoms;

#[cfg(test)]
#[path = "tests_chart.rs"]
mod tests_chart;
