//! Outlines as triangles (pt, label frame) for display.

use lyon_tessellation::geom::point;
use lyon_tessellation::path::Path;
use lyon_tessellation::{
    BuffersBuilder, FillOptions, FillRule, FillTessellator, FillVertex, StrokeOptions,
    StrokeTessellator, StrokeVertex, VertexBuffers,
};

use crate::typeset::{Contour, Paint, Segment, Shape};

/// Tolerance of curve flattening in pt (well below a pixel at any sane zoom).
const TOLERANCE_PT: f32 = 0.01;

/// Indexed triangles; vertices in pt.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Triangles {
    pub vertices: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
}

/// Fill [`Paint::Fill`] shapes (non-zero rule) and stroke [`Paint::Stroke`] ones.
pub fn tessellate(shapes: &[Shape]) -> Triangles {
    let mut out: VertexBuffers<[f32; 2], u32> = VertexBuffers::new();
    for shape in shapes {
        let path = path(&shape.contours);
        match shape.paint {
            Paint::Fill => {
                FillTessellator::new()
                    .tessellate_path(
                        &path,
                        &FillOptions::tolerance(TOLERANCE_PT).with_fill_rule(FillRule::NonZero),
                        &mut BuffersBuilder::new(&mut out, |v: FillVertex| v.position().to_array()),
                    )
                    .expect("fill tessellation of finite outlines");
            }
            Paint::Stroke { width } => {
                StrokeTessellator::new()
                    .tessellate_path(
                        &path,
                        &StrokeOptions::tolerance(TOLERANCE_PT).with_line_width(width as f32),
                        &mut BuffersBuilder::new(&mut out, |v: StrokeVertex| {
                            v.position().to_array()
                        }),
                    )
                    .expect("stroke tessellation of finite outlines");
            }
        }
    }
    Triangles {
        vertices: out.vertices,
        indices: out.indices,
    }
}

fn path(contours: &[Contour]) -> Path {
    let p = |q: constraint_runtime::Point2| point(q.x as f32, q.y as f32);
    let mut b = Path::builder();
    for Contour(segments) in contours {
        let mut open = false;
        for s in segments {
            match *s {
                Segment::MoveTo(q) => {
                    if open {
                        b.end(false);
                    }
                    b.begin(p(q));
                    open = true;
                }
                Segment::LineTo(q) => {
                    b.line_to(p(q));
                }
                Segment::QuadTo(c, q) => {
                    b.quadratic_bezier_to(p(c), p(q));
                }
                Segment::CubicTo(c1, c2, q) => {
                    b.cubic_bezier_to(p(c1), p(c2), p(q));
                }
                Segment::Close => {
                    if open {
                        b.end(true);
                        open = false;
                    }
                }
            }
        }
        if open {
            b.end(false);
        }
    }
    b.build()
}

#[cfg(test)]
#[path = "tests_tessellate.rs"]
mod tests_tessellate;
