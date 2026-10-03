//! Opaque point/vector/length expressions (Model B).
//!
//! Shared formula *tags* live in [`super::capabilities`]. This module keys them
//! by geometry kind for evaluate dispatch ([`register_expr_kinds!`]) and owns
//! the geometry identity so constraints and Interact never match on formula kinds.
//! Local→global index lifts use [`crate::mapper::Mapper`].
//!
//! Stack Jacobian carriers live in [`crate::geom`]
//! ([`crate::geom::PointJacobian`] / [`crate::geom::VecJacobian`] /
//! [`crate::geom::ScalarJacobian`]); this module only returns them from
//! evaluate helpers.
//!
//! **Adding a kind:** implement one `eval_*` per formula family (returns
//! [`FormulaEval`]), optional sparse `*_param_indices` overrides, then add one
//! block to [`register_expr_kinds!`] (generates keyed enums, Expr dispatch, and
//! [`super::capabilities::CurveFormulaKey`] / [`EndpointFormulaKey`](super::capabilities::EndpointFormulaKey)).

use super::shell::GeometryId;

/// Value + analytical Jacobian for one formula evaluation.
///
/// Connectivity indices stay separate (often chart-independent; default = all
/// chart slots) so [`param_indices`](PointExpr::param_indices) needs no State.
#[derive(Clone, Debug)]
pub(crate) struct FormulaEval<V, J> {
    pub value: V,
    pub jac: J,
}

impl<V, J> FormulaEval<V, J> {
    pub(crate) fn new(value: V, jac: J) -> Self {
        Self { value, jac }
    }
}

/// Opaque point expression: map from State parameters to a [`crate::geom::Point2`].
///
/// Construct via capability factories (e.g. [`super::EndpointExprs::start_expr`] on a kind).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointExpr {
    geometry: GeometryId,
    formula: PointFormulaKeyed,
}

impl PointExpr {
    pub(crate) fn new(geometry: GeometryId, formula: PointFormulaKeyed) -> Self {
        Self { geometry, formula }
    }

    pub fn referenced_geometry(&self) -> GeometryId {
        self.geometry
    }
}

/// Opaque free-vector expression: map from State parameters to a [`crate::geom::Vec2`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VecExpr {
    geometry: GeometryId,
    formula: VecFormulaKeyed,
}

impl VecExpr {
    pub(crate) fn new(geometry: GeometryId, formula: VecFormulaKeyed) -> Self {
        Self { geometry, formula }
    }

    pub fn referenced_geometry(&self) -> GeometryId {
        self.geometry
    }
}

/// Opaque length expression: map from State parameters to a [`crate::geom::Length`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LengthExpr {
    geometry: GeometryId,
    formula: LengthFormulaKeyed,
}

impl LengthExpr {
    pub(crate) fn new(geometry: GeometryId, formula: LengthFormulaKeyed) -> Self {
        Self { geometry, formula }
    }

    pub fn referenced_geometry(&self) -> GeometryId {
        self.geometry
    }
}

/// Dispatch for a single keyed catalog whose kind eval returns [`FormulaEval`].
///
/// Connectivity defaults to all chart [`SLOTS`](super::ChartSlot) unless
/// `indices:` names a kind override (e.g. sparse endpoints).
macro_rules! impl_keyed_expr {
    (
        $Expr:ident, $Keyed:ident => $Value:ty;
        eval: $eval:ident;
        jac: $jac_method:ident => $Jac:ty;
        $($var:ident => $kind:ty),+ $(,)?
    ) => {
        impl_keyed_expr! {
            $Expr, $Keyed => $Value;
            eval: $eval;
            jac: $jac_method => $Jac;
            indices_default;
            $($var => $kind),+
        }
    };

    (
        $Expr:ident, $Keyed:ident => $Value:ty;
        eval: $eval:ident;
        jac: $jac_method:ident => $Jac:ty;
        indices_default;
        $($var:ident => $kind:ty),+ $(,)?
    ) => {
        impl $Expr {
            pub fn evaluate(
                &self,
                store: &impl $crate::store::GeometryStore,
                p: &$crate::state::State,
            ) -> $Value {
                let g = store.expect_geometry(self.geometry);
                match &self.formula {
                    $(
                        $Keyed::$var(f) => {
                            let chart = $crate::mapper::Mapper::unpack_chart::<$kind>(g, p);
                            <$kind>::$eval(*f, chart).value
                        }
                    )+
                }
            }

            pub fn param_indices(
                &self,
                store: &impl $crate::store::GeometryStore,
            ) -> Vec<usize> {
                let local = match &self.formula {
                    $(
                        $Keyed::$var(_f) => $crate::geometry::ChartSlot::union_indices(<$kind>::SLOTS),
                    )+
                };
                $crate::mapper::Mapper::globalize_indices(
                    store.expect_geometry(self.geometry),
                    local,
                )
            }

            pub fn $jac_method(
                &self,
                store: &impl $crate::store::GeometryStore,
                p: &$crate::state::State,
            ) -> $Jac {
                let g = store.expect_geometry(self.geometry);
                match &self.formula {
                    $(
                        $Keyed::$var(f) => {
                            let chart = $crate::mapper::Mapper::unpack_chart::<$kind>(g, p);
                            <$kind>::$eval(*f, chart).jac
                        }
                    )+
                }
            }
        }
    };
}

/// Generate kind-keyed formula enums, Expr dispatch, and capability keying impls.
///
/// Each entry names a chart type and the enum variants used for curve points,
/// optional endpoint points, free vectors, and lengths.
macro_rules! register_expr_kinds {
    ($(
        $kind:ty {
            curve: $curve_var:ident,
            $(endpoint: $endpoint_var:ident,)?
            vec: $vec_var:ident,
            length: $length_var:ident $(,)?
        }
    ),+ $(,)?) => {
        /// Kind-keyed wrapper around curve / endpoint formula catalogs.
        #[derive(Clone, Copy, Debug, PartialEq)]
        pub(crate) enum PointFormulaKeyed {
            $(
                $curve_var($crate::geometry::capabilities::CurvePointFormula),
                $($endpoint_var($crate::geometry::capabilities::EndpointPointFormula),)?
            )+
        }

        /// Kind-keyed wrapper around the shared curve [`VecFormula`] catalog.
        #[derive(Clone, Copy, Debug, PartialEq)]
        pub(crate) enum VecFormulaKeyed {
            $(
                $vec_var($crate::geometry::capabilities::VecFormula),
            )+
        }

        /// Kind-keyed wrapper around the shared curve [`CurveLengthFormula`] catalog.
        #[derive(Clone, Copy, Debug, PartialEq)]
        pub(crate) enum LengthFormulaKeyed {
            $(
                $length_var($crate::geometry::capabilities::CurveLengthFormula),
            )+
        }

        $(
            impl $crate::geometry::capabilities::CurveFormulaKey for $kind {
                fn curve_point(
                    f: $crate::geometry::capabilities::CurvePointFormula,
                ) -> PointFormulaKeyed {
                    PointFormulaKeyed::$curve_var(f)
                }

                fn vec(f: $crate::geometry::capabilities::VecFormula) -> VecFormulaKeyed {
                    VecFormulaKeyed::$vec_var(f)
                }

                fn length(
                    f: $crate::geometry::capabilities::CurveLengthFormula,
                ) -> LengthFormulaKeyed {
                    LengthFormulaKeyed::$length_var(f)
                }
            }

            $(
                impl $crate::geometry::capabilities::EndpointFormulaKey for $kind {
                    fn endpoint_point(
                        f: $crate::geometry::capabilities::EndpointPointFormula,
                    ) -> PointFormulaKeyed {
                        PointFormulaKeyed::$endpoint_var(f)
                    }
                }
            )?
        )+

        // PointExprs span two catalogs (curve + optional endpoint).
        impl PointExpr {
            pub fn evaluate(
                &self,
                store: &impl $crate::store::GeometryStore,
                p: &$crate::state::State,
            ) -> $crate::geom::Point2 {
                let g = store.expect_geometry(self.geometry);
                match &self.formula {
                    $(
                        PointFormulaKeyed::$curve_var(f) => {
                            let chart = $crate::mapper::Mapper::unpack_chart::<$kind>(g, p);
                            <$kind>::eval_curve_point(*f, chart).value
                        }
                        $(
                            PointFormulaKeyed::$endpoint_var(f) => {
                                let chart = $crate::mapper::Mapper::unpack_chart::<$kind>(g, p);
                                <$kind>::eval_endpoint_point(*f, chart).value
                            }
                        )?
                    )+
                }
            }

            pub fn param_indices(
                &self,
                store: &impl $crate::store::GeometryStore,
            ) -> Vec<usize> {
                let local = match &self.formula {
                    $(
                        PointFormulaKeyed::$curve_var(_f) => {
                            $crate::geometry::ChartSlot::union_indices(<$kind>::SLOTS)
                        }
                        $(
                            PointFormulaKeyed::$endpoint_var(f) => {
                                <$kind>::endpoint_point_param_indices(*f)
                            }
                        )?
                    )+
                };
                $crate::mapper::Mapper::globalize_indices(
                    store.expect_geometry(self.geometry),
                    local,
                )
            }

            pub fn eval_point_jacobian(
                &self,
                store: &impl $crate::store::GeometryStore,
                p: &$crate::state::State,
            ) -> $crate::geom::PointJacobian {
                let g = store.expect_geometry(self.geometry);
                match &self.formula {
                    $(
                        PointFormulaKeyed::$curve_var(f) => {
                            let chart = $crate::mapper::Mapper::unpack_chart::<$kind>(g, p);
                            <$kind>::eval_curve_point(*f, chart).jac
                        }
                        $(
                            PointFormulaKeyed::$endpoint_var(f) => {
                                let chart = $crate::mapper::Mapper::unpack_chart::<$kind>(g, p);
                                <$kind>::eval_endpoint_point(*f, chart).jac
                            }
                        )?
                    )+
                }
            }
        }

        impl_keyed_expr! {
            VecExpr, VecFormulaKeyed => $crate::geom::Vec2;
            eval: eval_vec;
            jac: eval_vec_jacobian => $crate::geom::VecJacobian;
            $($vec_var => $kind),+
        }

        impl_keyed_expr! {
            LengthExpr, LengthFormulaKeyed => $crate::geom::Length;
            eval: eval_length;
            jac: eval_length_jacobian => $crate::geom::ScalarJacobian;
            $($length_var => $kind),+
        }
    };
}

// Kind registry: add new geometry kinds here (one block each).
// Also installs CurveFormulaKey / EndpointFormulaKey for listed kinds.
register_expr_kinds! {
    crate::geometry_kinds::Segment {
        curve: SegmentCurve,
        endpoint: SegmentEndpoint,
        vec: Segment,
        length: Segment,
    },
}
