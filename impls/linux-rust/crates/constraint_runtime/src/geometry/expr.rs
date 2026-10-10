//! Geometry-kind formula keying and Expr evaluate / Jacobian dispatch.
//!
//! Opaque [`PointExpr`](crate::expr::PointExpr) / [`VecExpr`](crate::expr::VecExpr) /
//! [`LengthExpr`](crate::expr::LengthExpr) /
//! [`HeadingAngleExpr`](crate::expr::HeadingAngleExpr) /
//! [`TurnAngleExpr`](crate::expr::TurnAngleExpr) types live in [`crate::expr`]. Shared
//! formula *tags* live in [`super::capabilities`]. This module keys them by
//! geometry kind ([`register_expr_kinds!`]) so constraints never match on
//! formula kinds. Local→global index lifts use [`crate::mapper::Mapper`].
//!
//! **Adding a kind:** implement `eval_*` + `accumulate_*_jacobian` per formula
//! family (connectivity defaults to all [`SLOTS`](super::chart::ChartSlot);
//! override only when sparse, e.g. `endpoint_point_param_indices`), then add one
//! block to [`register_expr_kinds!`] (keyed enums, Expr dispatch,
//! [`CurveFormulaKey`](super::capabilities::CurveFormulaKey) /
//! [`EndpointFormulaKey`](super::capabilities::EndpointFormulaKey)).

/// Dispatch for a single keyed catalog: value eval + Jacobian accumulate.
///
/// - `jac_static`: Jacobian ignores chart/Vector (affine / constant derivatives).
/// - `jac_chart`: Jacobian needs an unpacked chart.
/// Connectivity defaults to all chart slots.
macro_rules! impl_keyed_expr {
    (
        $Expr:ident, $Keyed:ident => $Value:ty;
        eval: $eval:ident;
        jac_static: $accumulate:ident;
        $($var:ident => $kind:ty),+ $(,)?
    ) => {
        impl $crate::expr::$Expr {
            pub fn evaluate<S: $crate::store::GeometryStore + ?Sized>(
                &self,
                store: &S,
                p: &$crate::vector::Vector,
            ) -> $Value {
                let g = store.expect_geometry(self.geometry);
                match &self.formula {
                    $(
                        $Keyed::$var(f) => {
                            let chart = $crate::mapper::Mapper::unpack_chart(
                                <$kind as $crate::geometry_kinds::InGeometryCatalog>::as_catalog_ref(g),
                                p,
                            );
                            <$kind>::$eval(*f, chart)
                        }
                    )+
                }
            }

            pub fn param_indices<S: $crate::store::GeometryStore + ?Sized>(
                &self,
                store: &S,
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

            /// Formula-local DOF count (no allocation).
            pub fn param_count(&self) -> usize {
                match &self.formula {
                    $($Keyed::$var(_f) => {
                        $crate::geometry::ChartSlot::union_dof(<$kind>::SLOTS)
                    })+
                }
            }

            /// Write the formula Jacobian into `j` (formula-local columns), scaled by `scale`.
            pub fn accumulate_jacobian<S: $crate::store::GeometryStore + ?Sized, J: $crate::matrix::MatrixAccess>(
                &self,
                _store: &S,
                _p: &$crate::vector::Vector,
                j: &mut J,
                scale: f64,
            ) {
                match &self.formula {
                    $($Keyed::$var(f) => <$kind>::$accumulate(*f, j, scale),)+
                }
            }
        }
    };

    (
        $Expr:ident, $Keyed:ident => $Value:ty;
        eval: $eval:ident;
        jac_chart: $accumulate:ident;
        $($var:ident => $kind:ty),+ $(,)?
    ) => {
        impl $crate::expr::$Expr {
            pub fn evaluate<S: $crate::store::GeometryStore + ?Sized>(
                &self,
                store: &S,
                p: &$crate::vector::Vector,
            ) -> $Value {
                let g = store.expect_geometry(self.geometry);
                match &self.formula {
                    $(
                        $Keyed::$var(f) => {
                            let chart = $crate::mapper::Mapper::unpack_chart(
                                <$kind as $crate::geometry_kinds::InGeometryCatalog>::as_catalog_ref(g),
                                p,
                            );
                            <$kind>::$eval(*f, chart)
                        }
                    )+
                }
            }

            pub fn param_indices<S: $crate::store::GeometryStore + ?Sized>(
                &self,
                store: &S,
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

            /// Formula-local DOF count (no allocation).
            pub fn param_count(&self) -> usize {
                match &self.formula {
                    $($Keyed::$var(_f) => {
                        $crate::geometry::ChartSlot::union_dof(<$kind>::SLOTS)
                    })+
                }
            }

            /// Write the formula Jacobian into `j` (formula-local columns), scaled by `scale`.
            pub fn accumulate_jacobian<S: $crate::store::GeometryStore + ?Sized, J: $crate::matrix::MatrixAccess>(
                &self,
                store: &S,
                p: &$crate::vector::Vector,
                j: &mut J,
                scale: f64,
            ) {
                let g = store.expect_geometry(self.geometry);
                match &self.formula {
                    $(
                        $Keyed::$var(f) => {
                            let chart = $crate::mapper::Mapper::unpack_chart(
                                <$kind as $crate::geometry_kinds::InGeometryCatalog>::as_catalog_ref(g),
                                p,
                            );
                            <$kind>::$accumulate(*f, chart, j, scale)
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
        impl $crate::expr::PointExpr {
            pub fn evaluate<S: $crate::store::GeometryStore + ?Sized>(
                &self,
                store: &S,
                p: &$crate::vector::Vector,
            ) -> $crate::geom::Point2 {
                let g = store.expect_geometry(self.geometry);
                match &self.formula {
                    $(
                        PointFormulaKeyed::$curve_var(f) => {
                            let chart = $crate::mapper::Mapper::unpack_chart(
                                <$kind as $crate::geometry_kinds::InGeometryCatalog>::as_catalog_ref(g),
                                p,
                            );
                            <$kind>::eval_curve_point(*f, chart)
                        }
                        $(
                            PointFormulaKeyed::$endpoint_var(f) => {
                                let chart = $crate::mapper::Mapper::unpack_chart(
                                <$kind as $crate::geometry_kinds::InGeometryCatalog>::as_catalog_ref(g),
                                p,
                            );
                                <$kind>::eval_endpoint_point(*f, chart)
                            }
                        )?
                    )+
                }
            }

            pub fn param_indices<S: $crate::store::GeometryStore + ?Sized>(
                &self,
                store: &S,
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

            /// Formula-local DOF count (no allocation).
            pub fn param_count(&self) -> usize {
                match &self.formula {
                    $(
                        PointFormulaKeyed::$curve_var(_f) => {
                            $crate::geometry::ChartSlot::union_dof(<$kind>::SLOTS)
                        }
                        $(
                            PointFormulaKeyed::$endpoint_var(f) => {
                                <$kind>::endpoint_point_param_count(*f)
                            }
                        )?
                    )+
                }
            }

            /// Write the \(2 \times n\) point Jacobian into `j` (formula-local columns).
            pub fn accumulate_jacobian<S: $crate::store::GeometryStore + ?Sized, J: $crate::matrix::MatrixAccess>(
                &self,
                store: &S,
                p: &$crate::vector::Vector,
                j: &mut J,
                scale: f64,
            ) {
                let g = store.expect_geometry(self.geometry);
                match &self.formula {
                    $(
                        PointFormulaKeyed::$curve_var(f) => {
                            let chart = $crate::mapper::Mapper::unpack_chart(
                                <$kind as $crate::geometry_kinds::InGeometryCatalog>::as_catalog_ref(g),
                                p,
                            );
                            <$kind>::accumulate_curve_point_jacobian(*f, chart, j, scale)
                        }
                        $(
                            PointFormulaKeyed::$endpoint_var(f) => {
                                let chart = $crate::mapper::Mapper::unpack_chart(
                                    <$kind as $crate::geometry_kinds::InGeometryCatalog>::as_catalog_ref(g),
                                    p,
                                );
                                <$kind>::accumulate_endpoint_point_jacobian(*f, chart, j, scale)
                            }
                        )?
                    )+
                }
            }
        }

        impl_keyed_expr! {
            VecExpr, VecFormulaKeyed => $crate::geom::Vec2;
            eval: eval_vec;
            jac_chart: accumulate_vec_jacobian;
            $($vec_var => $kind),+
        }

        impl_keyed_expr! {
            LengthExpr, LengthFormulaKeyed => $crate::geom::Length;
            eval: eval_length;
            jac_chart: accumulate_length_jacobian;
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
    crate::geometry_kinds::Arc {
        curve: ArcCurve,
        endpoint: ArcEndpoint,
        vec: Arc,
        length: Arc,
    },
}

/// Kind-keyed heading formulas.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum HeadingAngleFormulaKeyed {
    Arc(crate::geometry::capabilities::HeadingAngleFormula),
}

/// Kind-keyed turn formulas.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum TurnAngleFormulaKeyed {
    Arc(crate::geometry::capabilities::TurnAngleFormula),
}

impl crate::expr::HeadingAngleExpr {
    pub fn evaluate<S: crate::store::GeometryStore + ?Sized>(
        &self,
        store: &S,
        p: &crate::vector::Vector,
    ) -> crate::geom::HeadingAngle {
        let g = store.expect_geometry(self.geometry);
        match &self.formula {
            HeadingAngleFormulaKeyed::Arc(f) => {
                let chart = crate::mapper::Mapper::unpack_chart(
                    <crate::geometry_kinds::Arc as crate::geometry_kinds::InGeometryCatalog>::as_catalog_ref(
                        g,
                    ),
                    p,
                );
                crate::geometry_kinds::Arc::eval_heading(*f, chart)
            }
        }
    }

    pub fn param_indices<S: crate::store::GeometryStore + ?Sized>(
        &self,
        store: &S,
    ) -> Vec<usize> {
        let local = match &self.formula {
            HeadingAngleFormulaKeyed::Arc(f) => {
                crate::geometry_kinds::Arc::heading_param_indices(*f)
            }
        };
        crate::mapper::Mapper::globalize_indices(store.expect_geometry(self.geometry), local)
    }

    pub fn param_count(&self) -> usize {
        match &self.formula {
            HeadingAngleFormulaKeyed::Arc(f) => {
                crate::geometry_kinds::Arc::heading_param_count(*f)
            }
        }
    }

    pub fn accumulate_jacobian<
        S: crate::store::GeometryStore + ?Sized,
        J: crate::matrix::MatrixAccess,
    >(
        &self,
        _store: &S,
        _p: &crate::vector::Vector,
        j: &mut J,
        scale: f64,
    ) {
        match &self.formula {
            HeadingAngleFormulaKeyed::Arc(f) => {
                crate::geometry_kinds::Arc::accumulate_heading_jacobian(*f, j, scale)
            }
        }
    }
}

impl crate::expr::TurnAngleExpr {
    pub fn evaluate<S: crate::store::GeometryStore + ?Sized>(
        &self,
        store: &S,
        p: &crate::vector::Vector,
    ) -> crate::geom::TurnAngle {
        let g = store.expect_geometry(self.geometry);
        match &self.formula {
            TurnAngleFormulaKeyed::Arc(f) => {
                let chart = crate::mapper::Mapper::unpack_chart(
                    <crate::geometry_kinds::Arc as crate::geometry_kinds::InGeometryCatalog>::as_catalog_ref(
                        g,
                    ),
                    p,
                );
                crate::geometry_kinds::Arc::eval_turn(*f, chart)
            }
        }
    }

    pub fn param_indices<S: crate::store::GeometryStore + ?Sized>(
        &self,
        store: &S,
    ) -> Vec<usize> {
        let local = match &self.formula {
            TurnAngleFormulaKeyed::Arc(f) => crate::geometry_kinds::Arc::turn_param_indices(*f),
        };
        crate::mapper::Mapper::globalize_indices(store.expect_geometry(self.geometry), local)
    }

    pub fn param_count(&self) -> usize {
        match &self.formula {
            TurnAngleFormulaKeyed::Arc(f) => crate::geometry_kinds::Arc::turn_param_count(*f),
        }
    }

    pub fn accumulate_jacobian<
        S: crate::store::GeometryStore + ?Sized,
        J: crate::matrix::MatrixAccess,
    >(
        &self,
        _store: &S,
        _p: &crate::vector::Vector,
        j: &mut J,
        scale: f64,
    ) {
        match &self.formula {
            TurnAngleFormulaKeyed::Arc(f) => {
                crate::geometry_kinds::Arc::accumulate_turn_jacobian(*f, j, scale)
            }
        }
    }
}

impl crate::geometry_kinds::Arc {
    /// Heading at the arc start (chart atom).
    pub fn start_heading_expr(id: crate::geometry::GeometryId) -> crate::expr::HeadingAngleExpr {
        crate::expr::HeadingAngleExpr::new(
            id,
            HeadingAngleFormulaKeyed::Arc(
                crate::geometry::capabilities::HeadingAngleFormula::StartHeading,
            ),
        )
    }

    /// Signed sweep turn (chart atom).
    pub fn sweep_expr(id: crate::geometry::GeometryId) -> crate::expr::TurnAngleExpr {
        crate::expr::TurnAngleExpr::new(
            id,
            TurnAngleFormulaKeyed::Arc(crate::geometry::capabilities::TurnAngleFormula::Sweep),
        )
    }
}
