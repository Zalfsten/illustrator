//! A laid-out page as outlines in the label frame (pt, y up, origin at the
//! left end of the baseline).

use constraint_runtime::Point2;
use label_layout::{Contour, Paint, Segment, Shape, TypesetLine};
use typst::layout::{Abs, Frame, FrameItem, Point, Transform};
use typst::text::TextItem;
use typst::visualize::{CurveItem, Geometry, Shape as TypstShape};
use typst_layout::PagedDocument;

use crate::world::MARKER_PT;

/// The line on the first page of `doc`.
pub fn line(doc: &PagedDocument) -> TypesetLine {
    let frame = &doc.pages()[0].frame;
    let mut walk = Walk::default();
    walk.frame(frame, Transform::identity());
    // Typst's y points down from the top of the page; the marker gives the
    // baseline. Flip so that y points up from the baseline.
    let baseline = walk.baseline.unwrap_or(frame.height()).to_pt();
    let to_frame = |p: Point2| Point2::new(p.x, baseline - p.y);
    let shapes: Vec<Shape> = walk
        .shapes
        .into_iter()
        .map(|s| Shape {
            contours: s
                .contours
                .into_iter()
                .map(|Contour(segs)| Contour(segs.into_iter().map(|g| map(g, to_frame)).collect()))
                .collect(),
            paint: s.paint,
        })
        .collect();
    let (mut height, mut depth) = (0.0_f64, 0.0_f64);
    for p in shapes
        .iter()
        .flat_map(|s| &s.contours)
        .flat_map(|c| &c.0)
        .flat_map(Segment::points)
    {
        height = height.max(p.y);
        depth = depth.max(-p.y);
    }
    // Strokes reach half their width beyond their path.
    for s in &shapes {
        if let Paint::Stroke { width } = s.paint {
            for p in s
                .contours
                .iter()
                .flat_map(|c| &c.0)
                .flat_map(Segment::points)
            {
                height = height.max(p.y + 0.5 * width);
                depth = depth.max(-(p.y - 0.5 * width));
            }
        }
    }
    TypesetLine {
        shapes,
        width: frame.width().to_pt(),
        height,
        depth,
    }
}

fn map(s: Segment, f: impl Fn(Point2) -> Point2) -> Segment {
    match s {
        Segment::MoveTo(p) => Segment::MoveTo(f(p)),
        Segment::LineTo(p) => Segment::LineTo(f(p)),
        Segment::QuadTo(c, p) => Segment::QuadTo(f(c), f(p)),
        Segment::CubicTo(c1, c2, p) => Segment::CubicTo(f(c1), f(c2), f(p)),
        Segment::Close => Segment::Close,
    }
}

/// Shapes in page coordinates (pt, y down) and the marker's baseline.
#[derive(Default)]
struct Walk {
    baseline: Option<Abs>,
    shapes: Vec<Shape>,
}

fn pt(p: Point) -> Point2 {
    Point2::new(p.x.to_pt(), p.y.to_pt())
}

impl Walk {
    fn frame(&mut self, frame: &Frame, ts: Transform) {
        for (pos, item) in frame.items() {
            let ts = ts.pre_concat(Transform::translate(pos.x, pos.y));
            match item {
                FrameItem::Group(g) => self.frame(&g.frame, ts.pre_concat(g.transform)),
                FrameItem::Text(t) if is_marker(t) => {
                    self.baseline = Some(Point::zero().transform(ts).y);
                }
                FrameItem::Text(t) => self.text(t, ts),
                FrameItem::Shape(s, _) => self.shape(s, ts),
                _ => {}
            }
        }
    }

    /// Glyph outlines from the font, placed like Typst's own renderers do.
    fn text(&mut self, t: &TextItem, ts: Transform) {
        let size = t.size;
        let scale = size.to_pt() / t.font.units_per_em();
        let face = t.font.ttf();
        let mut contours = Vec::new();
        let mut x = Abs::zero();
        for g in &t.glyphs {
            let origin = Point::new(x + g.x_offset.at(size), -g.y_offset.at(size));
            let mut b = Outline {
                ts,
                origin,
                scale,
                contours: &mut contours,
                current: Vec::new(),
            };
            face.outline_glyph(ttf_parser::GlyphId(g.id), &mut b);
            b.flush();
            x += g.x_advance.at(size);
        }
        self.shapes.push(Shape {
            contours,
            paint: Paint::Fill,
        });
    }

    /// Fraction bars, radical overlines and similar: filled or stroked.
    fn shape(&mut self, s: &TypstShape, ts: Transform) {
        let p = |q: Point| pt(q.transform(ts));
        let contours = match &s.geometry {
            Geometry::Line(to) => vec![Contour(vec![
                Segment::MoveTo(p(Point::zero())),
                Segment::LineTo(p(*to)),
            ])],
            Geometry::Rect(size) => {
                let c = [
                    Point::zero(),
                    Point::new(size.x, Abs::zero()),
                    Point::new(size.x, size.y),
                    Point::new(Abs::zero(), size.y),
                ];
                vec![Contour(vec![
                    Segment::MoveTo(p(c[0])),
                    Segment::LineTo(p(c[1])),
                    Segment::LineTo(p(c[2])),
                    Segment::LineTo(p(c[3])),
                    Segment::Close,
                ])]
            }
            Geometry::Curve(curve) => {
                let mut out: Vec<Contour> = Vec::new();
                for item in curve.0.iter() {
                    match item {
                        CurveItem::Move(q) => out.push(Contour(vec![Segment::MoveTo(p(*q))])),
                        CurveItem::Line(q) => push(&mut out, Segment::LineTo(p(*q))),
                        CurveItem::Cubic(c1, c2, q) => {
                            push(&mut out, Segment::CubicTo(p(*c1), p(*c2), p(*q)))
                        }
                        CurveItem::Close => push(&mut out, Segment::Close),
                    }
                }
                out
            }
        };
        if s.fill.is_some() {
            self.shapes.push(Shape {
                contours: contours.clone(),
                paint: Paint::Fill,
            });
        }
        if let Some(stroke) = &s.stroke {
            self.shapes.push(Shape {
                contours,
                paint: Paint::Stroke {
                    width: stroke.thickness.to_pt(),
                },
            });
        }
    }
}

fn push(out: &mut Vec<Contour>, s: Segment) {
    match out.last_mut() {
        Some(c) => c.0.push(s),
        None => out.push(Contour(vec![s])),
    }
}

fn is_marker(t: &TextItem) -> bool {
    (t.size.to_pt() - MARKER_PT).abs() < 1e-9
}

/// ttf-parser outline (font units, y up) into page coordinates.
struct Outline<'a> {
    ts: Transform,
    origin: Point,
    scale: f64,
    contours: &'a mut Vec<Contour>,
    current: Vec<Segment>,
}

impl Outline<'_> {
    fn p(&self, x: f32, y: f32) -> Point2 {
        let local = Point::new(
            self.origin.x + Abs::pt(f64::from(x) * self.scale),
            self.origin.y - Abs::pt(f64::from(y) * self.scale),
        );
        pt(local.transform(self.ts))
    }

    fn flush(&mut self) {
        if !self.current.is_empty() {
            self.contours
                .push(Contour(std::mem::take(&mut self.current)));
        }
    }
}

impl ttf_parser::OutlineBuilder for Outline<'_> {
    fn move_to(&mut self, x: f32, y: f32) {
        self.flush();
        let p = self.p(x, y);
        self.current.push(Segment::MoveTo(p));
    }
    fn line_to(&mut self, x: f32, y: f32) {
        let p = self.p(x, y);
        self.current.push(Segment::LineTo(p));
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let (c, p) = (self.p(x1, y1), self.p(x, y));
        self.current.push(Segment::QuadTo(c, p));
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let (c1, c2, p) = (self.p(x1, y1), self.p(x2, y2), self.p(x, y));
        self.current.push(Segment::CubicTo(c1, c2, p));
    }
    fn close(&mut self) {
        self.current.push(Segment::Close);
        self.flush();
    }
}
