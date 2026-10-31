use super::*;
use crate::constraint_kinds::VectorsParallel;
use crate::document::Document;
use crate::geom::Point2;
use crate::geometry::{GeometryRole, LineExprs};
use crate::geometry_kinds::Segment;
use crate::names::ObjectId;

fn set(width: Option<f64>, color: Option<Rgb>, dash: Option<Dash>, arrow: Option<Arrow>) -> StylePatch {
    StylePatch {
        width: width.map(Some),
        color: color.map(Some),
        dash: dash.map(Some),
        arrow: arrow.map(Some),
    }
}

fn segment(doc: &mut Document) -> GeometryId {
    doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)))
}

fn axis_ref(overrides: StyleProps) -> StyleRef {
    StyleRef {
        style: "axis".to_owned(),
        overrides,
    }
}

const RED: Rgb = Rgb(0xc0, 0, 0);

#[test]
fn new_table_holds_only_an_empty_default() {
    let styles = Styles::default();
    let all: Vec<_> = styles.iter().collect();
    assert_eq!(all, vec![(DEFAULT_STYLE, &StyleProps::default())]);
}

#[test]
fn built_in_look() {
    let mut doc = Document::new();
    let x = segment(&mut doc);
    assert_eq!(doc.geometry_style(x), Some(&StyleRef::default()));
    assert_eq!(
        doc.resolved_style(x),
        Some(Style {
            width: 0.4,
            color: Rgb(0, 0, 0),
            dash: Dash::Solid,
            arrow: Arrow::None,
        })
    );
}

#[test]
fn override_wins() {
    let mut styles = Styles::default();
    styles.define("axis", &set(Some(0.8), None, None, Some(Arrow::End))).unwrap();
    let r = axis_ref(StyleProps {
        color: Some(RED),
        ..StyleProps::default()
    });
    assert_eq!(
        styles.resolve(&r),
        Style {
            width: 0.8,
            color: RED,
            dash: Dash::Solid,
            arrow: Arrow::End,
        }
    );
}

#[test]
fn default_fills_the_gaps() {
    let mut styles = Styles::default();
    styles.define(DEFAULT_STYLE, &set(Some(0.6), None, None, None)).unwrap();
    styles.define("axis", &set(None, None, None, Some(Arrow::End))).unwrap();
    let s = styles.resolve(&axis_ref(StyleProps::default()));
    assert_eq!(s.width, 0.6);
    assert_eq!(s.arrow, Arrow::End);
}

#[test]
fn define_merges_and_null_removes() {
    let mut styles = Styles::default();
    styles.define("vector", &set(Some(0.8), None, None, Some(Arrow::End))).unwrap();
    styles.define("vector", &set(Some(1.2), None, None, None)).unwrap();
    assert_eq!(
        styles.get("vector"),
        Some(&StyleProps {
            width: Some(1.2),
            arrow: Some(Arrow::End),
            ..StyleProps::default()
        })
    );
    let remove = StylePatch {
        width: Some(None),
        ..StylePatch::default()
    };
    styles.define("vector", &remove).unwrap();
    assert_eq!(styles.get("vector").unwrap().width, None);
}

#[test]
fn define_rejects_bad_names_and_widths() {
    let mut styles = Styles::default();
    assert_eq!(styles.define("2nd", &set(Some(1.0), None, None, None)), Err(StyleError::InvalidName));
    for w in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert_eq!(styles.define("axis", &set(Some(w), None, None, None)), Err(StyleError::InvalidWidth));
    }
    assert!(!styles.contains("axis"));
}

#[test]
fn changing_a_style_changes_its_users() {
    let mut doc = Document::new();
    doc.define_style("axis", &set(Some(0.8), None, None, None)).unwrap();
    let x = segment(&mut doc);
    let y = segment(&mut doc);
    doc.set_geometry_style(x, axis_ref(StyleProps::default())).unwrap();
    let thick = StyleProps {
        width: Some(1.0),
        ..StyleProps::default()
    };
    doc.set_geometry_style(y, axis_ref(thick)).unwrap();
    doc.define_style("axis", &set(Some(1.2), None, None, None)).unwrap();
    assert_eq!(doc.resolved_style(x).unwrap().width, 1.2);
    assert_eq!(doc.resolved_style(y).unwrap().width, 1.0);
}

#[test]
fn style_in_use() {
    let mut doc = Document::new();
    doc.define_style("axis", &set(Some(0.8), None, None, None)).unwrap();
    let x = segment(&mut doc);
    doc.set_geometry_style(x, axis_ref(StyleProps::default())).unwrap();
    let before = doc.styles().clone();
    assert_eq!(doc.delete_style("axis"), Err(StyleError::InUse(vec![x])));
    assert_eq!(doc.styles(), &before);
}

#[test]
fn default_stays() {
    let mut doc = Document::new();
    assert_eq!(doc.delete_style(DEFAULT_STYLE), Err(StyleError::InUse(vec![])));
    assert!(doc.styles().contains(DEFAULT_STYLE));
}

#[test]
fn deleting_unused_and_unknown_styles() {
    let mut doc = Document::new();
    doc.define_style("hidden", &set(None, None, Some(Dash::Dashed), None)).unwrap();
    doc.delete_style("hidden").unwrap();
    assert!(!doc.styles().contains("hidden"));
    assert_eq!(doc.delete_style("hidden"), Err(StyleError::Unknown("hidden".to_owned())));
}

#[test]
fn geometry_cannot_refer_to_an_unknown_style() {
    let mut doc = Document::new();
    let x = segment(&mut doc);
    assert_eq!(
        doc.set_geometry_style(x, axis_ref(StyleProps::default())),
        Err(StyleError::Unknown("axis".to_owned()))
    );
    assert_eq!(doc.geometry_style(x), Some(&StyleRef::default()));
}

#[test]
fn style_and_object_may_share_a_name() {
    let mut doc = Document::new();
    let x = segment(&mut doc);
    doc.rename(ObjectId::Geometry(x), "axis").unwrap();
    doc.define_style("axis", &set(Some(0.8), None, None, None)).unwrap();
    assert_eq!(doc.lookup("axis"), Some(ObjectId::Geometry(x)));
    assert!(doc.styles().contains("axis"));
}

#[test]
fn restyling_does_not_move_anything() {
    let mut doc = Document::new();
    let a = segment(&mut doc);
    let b = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(0.0, 1.0), Point2::new(2.0, 1.0)));
    doc.add_constraint(VectorsParallel {
        a: Segment::direction_expr(a),
        b: Segment::direction_expr(b),
    });
    let p = doc.clone_vector();
    let look = StyleProps {
        width: Some(2.0),
        arrow: Some(Arrow::End),
        ..StyleProps::default()
    };
    doc.set_geometry_style(
        b,
        StyleRef {
            style: DEFAULT_STYLE.to_owned(),
            overrides: look,
        },
    )
    .unwrap();
    assert_eq!(doc.vector(), &p);
}

#[test]
fn cloned_document_carries_the_styles() {
    let mut doc = Document::new();
    doc.define_style("axis", &set(Some(0.8), Some(RED), None, None)).unwrap();
    let x = segment(&mut doc);
    doc.set_geometry_style(x, axis_ref(StyleProps::default())).unwrap();
    let copy = doc.clone();
    doc.define_style("axis", &set(Some(2.0), None, None, None)).unwrap();
    assert_eq!(copy.styles().get("axis").unwrap().width, Some(0.8));
    assert_eq!(copy.geometry_style(x).unwrap().style, "axis");
}
