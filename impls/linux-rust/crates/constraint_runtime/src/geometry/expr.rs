//! Opaque point/vector/length/angle expressions (Model B).
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
//! **Adding a kind:** extend the [`register_expr_kinds!`] invocation at the
//! bottom of this file (and implement the kind's `eval_*` / `*_param_indices`
//! inherent methods). Capability macros pick the same variant names.

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

/// Opaque angle expression: map from State parameters to a [`crate::geom::Angle`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AngleExpr {
    geometry: GeometryId,
    formula: AngleFormulaKeyed,
}

impl AngleExpr {
    #[allow(dead_code)] // constructed once a kind registers `angle:` formulas
    pub(crate) fn new(geometry: GeometryId, formula: AngleFormulaKeyed) -> Self {
        Self { geometry, formula }
    }

    pub fn referenced_geometry(&self) -> GeometryId {
        self.geometry
    }
}

/// Generate kind-keyed formula enums and evaluate / connectivity / Jacobian
/// dispatch for opaque expressions.
///
/// Each entry names a chart type and the enum variants used for curve points,
/// optional endpoint points, free vectors, lengths, and optional angles.
macro_rules! register_expr_kinds {
    ($(
        $kind:ty {
            curve: $curve_var:ident,
            $(endpoint: $endpoint_var:ident,)?
            vec: $vec_var:ident,
            length: $length_var:ident,
            $(angle: $angle_var:ident,)?
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

        /// Kind-keyed wrapper around the shared curve [`CurveAngleFormula`] catalog.
        #[derive(Clone, Copy, Debug, PartialEq)]
        pub(crate) enum AngleFormulaKeyed {
            $(
                $($angle_var($crate::geometry::capabilities::CurveAngleFormula),)?
            )+
        }

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
                store: &impl $crate::store::GeometryStore,
            ) -> Vec<usize> {
                let local = match &self.formula {
                    $(
                        PointFormulaKeyed::$curve_var(f) => <$kind>::curve_point_param_indices(*f),
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
                _store: &impl $crate::store::GeometryStore,
                _p: &$crate::state::State,
            ) -> $crate::geom::PointJacobian {
                match &self.formula {
                    $(
                        PointFormulaKeyed::$curve_var(f) => <$kind>::eval_curve_point_jacobian(*f),
                        $(
                            PointFormulaKeyed::$endpoint_var(f) => {
                                <$kind>::eval_endpoint_point_jacobian(*f)
                            }
                        )?
                    )+
                }
            }
        }

        impl VecExpr {
            pub fn evaluate(
                &self,
                store: &impl $crate::store::GeometryStore,
                p: &$crate::state::State,
            ) -> $crate::geom::Vec2 {
                let g = store.expect_geometry(self.geometry);
                match &self.formula {
                    $(
                        VecFormulaKeyed::$vec_var(f) => {
                            let chart = $crate::mapper::Mapper::unpack_chart::<$kind>(g, p);
                            <$kind>::eval_vec(*f, chart)
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
                        VecFormulaKeyed::$vec_var(f) => <$kind>::vec_param_indices(*f),
                    )+
                };
                $crate::mapper::Mapper::globalize_indices(
                    store.expect_geometry(self.geometry),
                    local,
                )
            }

            pub fn eval_vec_jacobian(
                &self,
                _store: &impl $crate::store::GeometryStore,
                _p: &$crate::state::State,
            ) -> $crate::geom::VecJacobian {
                match &self.formula {
                    $(
                        VecFormulaKeyed::$vec_var(f) => <$kind>::eval_vec_jacobian(*f),
                    )+
                }
            }
        }

        impl LengthExpr {
            pub fn evaluate(
                &self,
                store: &impl $crate::store::GeometryStore,
                p: &$crate::state::State,
            ) -> $crate::geom::Length {
                let g = store.expect_geometry(self.geometry);
                match &self.formula {
                    $(
                        LengthFormulaKeyed::$length_var(f) => {
                            let chart = $crate::mapper::Mapper::unpack_chart::<$kind>(g, p);
                            <$kind>::eval_length(*f, chart)
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
                        LengthFormulaKeyed::$length_var(f) => <$kind>::length_param_indices(*f),
                    )+
                };
                $crate::mapper::Mapper::globalize_indices(
                    store.expect_geometry(self.geometry),
                    local,
                )
            }

            pub fn eval_length_jacobian(
                &self,
                store: &impl $crate::store::GeometryStore,
                p: &$crate::state::State,
            ) -> $crate::geom::ScalarJacobian {
                let g = store.expect_geometry(self.geometry);
                match &self.formula {
                    $(
                        LengthFormulaKeyed::$length_var(f) => {
                            let chart = $crate::mapper::Mapper::unpack_chart::<$kind>(g, p);
                            <$kind>::eval_length_jacobian(*f, chart)
                        }
                    )+
                }
            }
        }

        impl AngleExpr {
            // `store`/`p` unused until a kind registers `angle:`.
            #[allow(unused_variables)]
            pub fn evaluate(
                &self,
                store: &impl $crate::store::GeometryStore,
                p: &$crate::state::State,
            ) -> $crate::geom::Angle {
                // Match by value: `&EmptyEnum` is treated as inhabited for exhaustiveness.
                match self.formula {
                    $(
                        $(
                            AngleFormulaKeyed::$angle_var(f) => {
                                let g = store.expect_geometry(self.geometry);
                                let chart = $crate::mapper::Mapper::unpack_chart::<$kind>(g, p);
                                <$kind>::eval_angle(f, chart)
                            }
                        )?
                    )+
                }
            }

            #[allow(unused_variables)]
            pub fn param_indices(
                &self,
                store: &impl $crate::store::GeometryStore,
            ) -> Vec<usize> {
                match self.formula {
                    $(
                        $(
                            AngleFormulaKeyed::$angle_var(f) => {
                                $crate::mapper::Mapper::globalize_indices(
                                    store.expect_geometry(self.geometry),
                                    <$kind>::angle_param_indices(f),
                                )
                            }
                        )?
                    )+
                }
            }

            #[allow(unused_variables)]
            pub fn eval_angle_jacobian(
                &self,
                store: &impl $crate::store::GeometryStore,
                p: &$crate::state::State,
            ) -> $crate::geom::ScalarJacobian {
                match self.formula {
                    $(
                        $(
                            AngleFormulaKeyed::$angle_var(f) => {
                                let g = store.expect_geometry(self.geometry);
                                let chart = $crate::mapper::Mapper::unpack_chart::<$kind>(g, p);
                                <$kind>::eval_angle_jacobian(f, chart)
                            }
                        )?
                    )+
                }
            }
        }
    };
}

// Kind registry: add new geometry kinds here (one block each).
register_expr_kinds! {
    crate::geometry_kinds::Segment {
        curve: SegmentCurve,
        endpoint: SegmentEndpoint,
        vec: Segment,
        length: Segment,
    },
}
