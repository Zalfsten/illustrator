//! Test typesetters: every line a filled 5·n × 7 pt rectangle (n characters);
//! lines containing `\foo` fail as unsupported. [`Manual`] answers only when
//! told to, like a loading worker.

use std::cell::RefCell;
use std::rc::Rc;

use constraint_runtime::{Markup, Point2};
use label_layout::{
    Contour, LineRequest, LineResult, LineTypesetter, Paint, Segment, Shape, TypesetError, TypesetLine,
    Typesetter, TypesetterState,
};

pub struct Boxes;

impl LineTypesetter for Boxes {
    fn typeset_line(&self, line: &str, _: Markup, _: f64, _: &str) -> LineResult {
        if line.contains(r"\foo") {
            return Err(TypesetError::Unsupported {
                construct: r"\foo".into(),
            });
        }
        let w = 5.0 * line.chars().count() as f64;
        let p = Point2::new;
        Ok(TypesetLine {
            shapes: vec![Shape {
                contours: vec![Contour(vec![
                    Segment::MoveTo(p(0.0, 0.0)),
                    Segment::LineTo(p(w, 0.0)),
                    Segment::LineTo(p(w, 7.0)),
                    Segment::LineTo(p(0.0, 7.0)),
                    Segment::Close,
                ])],
                paint: Paint::Fill,
            }],
            width: w,
            height: 7.0,
            depth: 0.0,
        })
    }
}

#[derive(Default)]
struct State {
    loading: bool,
    waiting: Vec<LineRequest>,
    done: Vec<(LineRequest, LineResult)>,
}

/// A loading typesetter whose requests wait for [`Manual::complete`].
#[derive(Clone, Default)]
pub struct Manual(Rc<RefCell<State>>);

impl Manual {
    pub fn loading() -> Self {
        let m = Self::default();
        m.0.borrow_mut().loading = true;
        m
    }

    /// Become ready and answer every waiting request.
    pub fn complete(&self) {
        let mut s = self.0.borrow_mut();
        s.loading = false;
        let waiting = std::mem::take(&mut s.waiting);
        for req in waiting {
            let r = Boxes.typeset_line(&req.line, req.markup, req.size_pt(), &req.preamble);
            s.done.push((req, r));
        }
    }
}

impl Typesetter for Manual {
    fn request(&self, req: LineRequest) {
        self.0.borrow_mut().waiting.push(req);
    }

    fn take_completed(&self) -> Vec<(LineRequest, LineResult)> {
        std::mem::take(&mut self.0.borrow_mut().done)
    }

    fn state(&self) -> TypesetterState {
        if self.0.borrow().loading {
            TypesetterState::Loading
        } else {
            TypesetterState::Ready
        }
    }
}
