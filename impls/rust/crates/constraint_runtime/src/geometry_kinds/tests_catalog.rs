use super::*;

#[test]
fn capability_sets_match_kinds() {
    assert_eq!(
        GeometryKind::Segment.caps(),
        Caps::CURVE.with_endpoints().with_line()
    );
    assert_eq!(
        GeometryKind::Arc.caps(),
        Caps::CURVE.with_endpoints().with_circular()
    );
    assert_eq!(GeometryKind::Label.caps(), Caps::NONE.with_anchor().with_line());
}

/// Every geometry kind (extend with the catalog).
const ALL_KINDS: [GeometryKind; 3] = [GeometryKind::Segment, GeometryKind::Arc, GeometryKind::Label];

#[test]
fn name_prefixes_are_valid_stems() {
    for kind in ALL_KINDS {
        let p = kind.name_prefix();
        assert!(crate::names::is_valid_name(p), "{kind:?}: {p:?}");
        assert!(
            !p.ends_with(|c: char| c.is_ascii_digit()),
            "{kind:?}: {p:?}"
        );
    }
}

/// A curve kind that keeps the panicking `shape` / `curvature` defaults
/// compiles, but panics in the first Mesh or export; catch it here.
#[test]
fn curve_kinds_report_shape_and_curvature() {
    for kind in ALL_KINDS.into_iter().filter(|k| k.caps().curve) {
        with_kind!(kind, K => {
            let params: Vec<f64> = (0..K::PARAMETER_COUNT).map(|i| 1.0 + i as f64).collect();
            let c = K::unpack(&params);
            let stored = K::default_attrs();
            let a = K::attrs_of(&stored);
            assert!(!K::shape(c, a).is_empty(), "{kind:?} has no shape");
            let k0 = K::curvature(0.0, c, a);
            for t in [0.0, 0.3, 1.0] {
                let k = K::curvature(t, c, a);
                assert!(k >= 0.0, "{kind:?}: curvature {k} at t = {t}");
                if K::UNIFORM_CURVE {
                    assert_eq!(k, k0, "{kind:?} is uniform but its curvature varies");
                }
            }
        });
    }
}

#[test]
fn instances_project_their_stored_attributes() {
    use crate::document::Document;
    use crate::geom::{HeadingAngle, Length, Point2, TurnAngle};
    use crate::geometry::GeometryRole;
    use crate::geometry_kinds::{Arc, Label, LabelAttrs, Segment};

    let mut doc = Document::new();
    let ids = [
        doc.add_geometry::<Segment>(
            GeometryRole::Result,
            (Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)),
        ),
        doc.add_geometry::<Arc>(
            GeometryRole::Result,
            (
                Point2::new(0.0, 0.0),
                Length::new(1.0),
                HeadingAngle::new(0.0),
                TurnAngle::new(1.0),
            ),
        ),
        doc.add_geometry::<Label>(
            GeometryRole::Result,
            (Point2::new(0.0, 0.0), HeadingAngle::new(0.0)),
        ),
    ];
    for id in ids {
        let g = doc.geometry(id).unwrap();
        with_kind!(g.kind(), K => {
            let default = K::default_attrs();
            assert_eq!(g.attrs_of::<K>(), K::attrs_of(&default), "{:?}", g.kind());
        });
    }

    let label = ids[2];
    let attrs = LabelAttrs {
        source: "\\alpha".into(),
        ..LabelAttrs::default()
    };
    assert!(doc.set_label_attrs(label, attrs.clone()));
    assert_eq!(doc.geometry(label).unwrap().attrs_of::<Label>(), &attrs);
}
