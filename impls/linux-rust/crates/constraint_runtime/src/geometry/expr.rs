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
//! **Adding a kind:** implement `eval_*` + `eval_*_jacobian` per formula family
//! (connectivity defaults to all [`SLOTS`](super::chart::ChartSlot); override only
//! when sparse, e.g. `endpoint_point_param_indices`), then add one block to
//! [`register_expr_kinds!`] (keyed enums, Expr dispatch,
//! [`CurveFormulaKey`](super::capabilities::CurveFormulaKey) /
//! [`EndpointFormulaKey`](super::capabilities::EndpointFormulaKey)).

use super::shell::GeometryId;

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

/// Dispatch for a single keyed catalog: separate value vs Jacobian.
///
/// - `jac_static`: Jacobian ignores chart/State (affine / constant derivatives).
/// - `jac_chart`: Jacobian needs an unpacked chart.
/// Connectivity defaults to all chart slots.
macro_rules! impl_keyed_expr {
    (
        $Expr:ident, $Keyed:ident => $Value:ty;
        eval: $eval:ident;
        jac_static: $jac:ident => $Jac:ty;
        $($var:ident => $kind:ty),+ $(,)?
    ) => {
        impl $Expr {
            pub fn evaluate(
                &self,
                store: &dyn $crate::store::GeometryStore,
                p: &$crate::state::State,
            ) -> $Value {
                let g = store.expect_geometry(self.geometry);
                match &self.formula {
                    $(
                        $Keyed::$var(f) => {
                            let chart = $crate::mapper::Mapper::unpack_chart::<$kind>(g, p);
                            <$kind>::$eval(*f, chart)
                        }
                    )+
                }
            }

            pub fn param_indices(
                &self,
                store: &dyn $crate::store::GeometryStore,
            ) -> Vec<usize> {
                let local = match &self.formula {
                    $(
                        $Keyed::$var(_f) => {
                            $crate::geometry::ChartSlot::union_indices(<$kind>::SLOTS)
                        }
                    )+
                };
                $crate::mapper::Mapper::globalize_indices(
                    store.expect_geometry(self.geometry),
                    local,
                )
            }

            pub fn $jac(
                &self,
                _store: &dyn $crate::store::GeometryStore,
                _p: &$crate::state::State,
            ) -> $Jac {
                match &self.formula {
                    $($Keyed::$var(f) => <$kind>::$jac(*f),)+
                }
            }
        }
    };

    (
        $Expr:ident, $Keyed:ident => $Value:ty;
        eval: $eval:ident;
        jac_chart: $jac:ident => $Jac:ty;
        $($var:ident => $kind:ty),+ $(,)?
    ) => {
        impl $Expr {
            pub fn evaluate(
                &self,
                store: &dyn $crate::store::GeometryStore,
                p: &$crate::state::State,
            ) -> $Value {
                let g = store.expect_geometry(self.geometry);
                match &self.formula {
                    $(
                        $Keyed::$var(f) => {
                            let chart = $crate::mapper::Mapper::unpack_chart::<$kind>(g, p);
                            <$kind>::$eval(*f, chart)
                        }
                    )+
                }
            }

            pub fn param_indices(
                &self,
                store: &dyn $crate::store::GeometryStore,
            ) -> Vec<usize> {
                let local = match &self.formula {
                    $(
                        $Keyed::$var(_f) => {
                            $crate::geometry::ChartSlot::union_indices(<$kind>::SLOTS)
                        }
                    )+
                };
                $crate::mapper::Mapper::globalize_indices(
                    store.expect_geometry(self.geometry),
                    local,
                )
            }

            pub fn $jac(
                &self,
                store: &dyn $crate::store::GeometryStore,
                p: &$crate::state::State,
            ) -> $Jac {
                let g = store.expect_geometry(self.geometry);
                match &self.formula {
                    $(
                        $Keyed::$var(f) => {
                            let chart = $crate::mapper::Mapper::unpack_chart::<$kind>(g, p);
                            <$kind>::$jac(*f, chart)
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
                store: &dyn $crate::store::GeometryStore,
                p: &$crate::state::State,
            ) -> $crate::geom::Point2 {
                let g = store.expect_geometry(self.geometry);
                match &self.formula {
                    $(
                        PointFormulaKeyed::$curve_var(f) => {
                            let chart = $crate::mapper::Mapper::unpack_chart::<$kind>(g, p);
                            <$kind>::eval_curve_point(*f, chart)
                        }
                        $(
                            PointFormulaKeyed::$endpoint_var(f) => {
                                let chart = $crate::mapper::Mapper::unpack_chart::<$kind>(g, p);
                                <$kind>::eval_endpoint_point(*f, chart)
                            }
                        )?
                    )+
                }
            }

            pub fn param_indices(
                &self,
                store: &dyn $crate::store::GeometryStore,
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
                _store: &dyn $crate::store::GeometryStore,
                _p: &$crate::state::State,
            ) -> $crate::geom::PointJacobian {
                match &self.formula {
                    $(
                        PointFormulaKeyed::$curve_var(f) => {
                            <$kind>::eval_curve_point_jacobian(*f)
                        }
                        $(
                            PointFormulaKeyed::$endpoint_var(f) => {
                                <$kind>::eval_endpoint_point_jacobian(*f)
                            }
                        )?
                    )+
                }
            }
        }

        impl_keyed_expr! {
            VecExpr, VecFormulaKeyed => $crate::geom::Vec2;
            eval: eval_vec;
            jac_static: eval_vec_jacobian => $crate::geom::VecJacobian;
            $($vec_var => $kind),+
        }

        impl_keyed_expr! {
            LengthExpr, LengthFormulaKeyed => $crate::geom::Length;
            eval: eval_length;
            jac_chart: eval_length_jacobian => $crate::geom::ScalarJacobian;
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
