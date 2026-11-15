//! Viewport: world (y-up) ↔ screen (y-down) mapping for the Canvas.

use constraint_runtime::{DocumentSettings, Point2};
use eframe::egui::{Pos2, Rect, Vec2 as ScreenVec};

/// Uniform scale + origin mapping world coordinates to Canvas pixels (y-flip).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Viewport {
    /// Screen units per world unit (uniform).
    pub scale: f64,
    /// Screen position of world origin `(0, 0)`.
    pub origin_screen: Pos2,
}

impl Viewport {
    pub fn new(scale: f64, origin_screen: Pos2) -> Self {
        assert!(scale > 0.0 && scale.is_finite());
        Self {
            scale,
            origin_screen,
        }
    }

    /// Fit `world` AABB into `canvas` with uniform scale and margin (logical points).
    pub fn fit(canvas: Rect, world_min: Point2, world_max: Point2, margin: f32) -> Self {
        let cw = (canvas.width() - 2.0 * margin).max(1.0) as f64;
        let ch = (canvas.height() - 2.0 * margin).max(1.0) as f64;
        let ww = (world_max.x - world_min.x).abs().max(1e-9);
        let wh = (world_max.y - world_min.y).abs().max(1e-9);
        let scale = (cw / ww).min(ch / wh);

        // Place world AABB centered in the padded canvas; y-up → screen y decreases with world y.
        let world_cx = 0.5 * (world_min.x + world_max.x);
        let world_cy = 0.5 * (world_min.y + world_max.y);
        let screen_cx = canvas.center().x as f64;
        let screen_cy = canvas.center().y as f64;
        // screen = origin + (scale * wx, -scale * wy)
        // screen_cx = origin_x + scale * world_cx
        // screen_cy = origin_y - scale * world_cy
        let origin_x = screen_cx - scale * world_cx;
        let origin_y = screen_cy + scale * world_cy;
        Self::new(scale, Pos2::new(origin_x as f32, origin_y as f32))
    }

    pub fn to_screen(self, p: Point2) -> Pos2 {
        Pos2::new(
            (self.origin_screen.x as f64 + self.scale * p.x) as f32,
            (self.origin_screen.y as f64 - self.scale * p.y) as f32,
        )
    }

    pub fn to_world(self, s: Pos2) -> Point2 {
        Point2::new(
            (s.x as f64 - self.origin_screen.x as f64) / self.scale,
            (self.origin_screen.y as f64 - s.y as f64) / self.scale,
        )
    }

    /// Convert a length in screen pixels to world units.
    pub fn px_to_world(&self, px: f64) -> f64 {
        px / self.scale
    }
}

/// Logical screen units per inch assumed for the print-size zoom (the
/// browser does not report the physical DPI).
pub const SCREEN_UNITS_PER_INCH: f64 = 96.0;
/// Zoom bounds as fractions of the print-size scale: 1 % … 100 000 %.
pub const ZOOM_MIN: f64 = 0.01;
pub const ZOOM_MAX: f64 = 1000.0;

/// Screen units per world unit at physical print size (system-architecture:
/// Print-size zoom and zoom display).
pub fn print_scale(settings: &DocumentSettings) -> f64 {
    settings.scale * SCREEN_UNITS_PER_INCH / 25.4
}

/// Smallest and largest allowed scale for `settings` (system-architecture:
/// Zoom is bounded).
pub fn zoom_bounds(settings: &DocumentSettings) -> (f64, f64) {
    let s = print_scale(settings);
    (ZOOM_MIN * s, ZOOM_MAX * s)
}

/// The view as UI session state: the world location at the Canvas center and
/// the scale (screen units per world unit). Toolkit types appear only at the
/// edges (`Rect`, `Pos2`, `ScreenVec`), so the math is testable headless.
///
/// Storing the center (not the screen origin) makes a Canvas resize keep
/// center and scale for free (system-architecture: Resizing the Canvas keeps
/// center and scale).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewState {
    pub center: Point2,
    pub scale: f64,
}

impl ViewState {
    /// The world ↔ screen mapping for `canvas`.
    pub fn viewport(&self, canvas: Rect) -> Viewport {
        let c = canvas.center();
        let origin_x = c.x as f64 - self.scale * self.center.x;
        let origin_y = c.y as f64 + self.scale * self.center.y;
        Viewport::new(self.scale, Pos2::new(origin_x as f32, origin_y as f32))
    }

    /// Multiply the scale by `factor` (clamped to `bounds`), keeping the world
    /// location under `anchor` at the same screen position.
    pub fn zoom(&mut self, canvas: Rect, anchor: Pos2, factor: f64, bounds: (f64, f64)) {
        if !(factor.is_finite() && factor > 0.0) {
            return;
        }
        let w = self.viewport(canvas).to_world(anchor);
        let s = self.scale;
        let s2 = (s * factor).clamp(bounds.0, bounds.1);
        // center′ = w + (center − w)·(s / s′): w stays under the anchor.
        self.center = w + (self.center - w) * (s / s2);
        self.scale = s2;
    }

    /// Move every screen image by `delta` (screen units, y down).
    pub fn pan(&mut self, delta: ScreenVec) {
        self.center = Point2::new(
            self.center.x - delta.x as f64 / self.scale,
            self.center.y + delta.y as f64 / self.scale,
        );
    }

    /// Clamp the scale into `bounds` around the Canvas center (bounds move
    /// with the document scale).
    pub fn clamp(&mut self, bounds: (f64, f64)) {
        self.scale = self.scale.clamp(bounds.0, bounds.1);
    }

    /// The view that fits the world box `min`..`max` into `canvas` with
    /// `margin` screen units per side, scale clamped to `bounds`.
    pub fn fit(canvas: Rect, min: Point2, max: Point2, margin: f32, bounds: (f64, f64)) -> Self {
        let vp = Viewport::fit(canvas, min, max, margin);
        Self {
            center: Point2::new(0.5 * (min.x + max.x), 0.5 * (min.y + max.y)),
            scale: vp.scale.clamp(bounds.0, bounds.1),
        }
    }

    /// Zoom as a percentage of print size.
    pub fn percent(&self, print_scale: f64) -> f64 {
        100.0 * self.scale / print_scale
    }
}

/// Margin around a drawing's box, as a fraction of its size per side.
const BOX_MARGIN: f64 = 0.1;
/// Smallest box size per axis in world units (one point, or a line).
const BOX_MIN_SIZE: f64 = 1.0;

/// World AABB `(min, max)` showing all `points` with a margin; `None` without points.
pub fn world_box(points: impl IntoIterator<Item = Point2>) -> Option<(Point2, Point2)> {
    let mut points = points.into_iter();
    let first = points.next()?;
    let (mut min, mut max) = (first, first);
    for p in points {
        min = Point2::new(min.x.min(p.x), min.y.min(p.y));
        max = Point2::new(max.x.max(p.x), max.y.max(p.y));
    }
    let grow = |lo: f64, hi: f64| {
        let size = (hi - lo).max(BOX_MIN_SIZE);
        let (mid, half) = (0.5 * (lo + hi), 0.5 * size * (1.0 + 2.0 * BOX_MARGIN));
        (mid - half, mid + half)
    };
    let (x0, x1) = grow(min.x, max.x);
    let (y0, y1) = grow(min.y, max.y);
    Some((Point2::new(x0, y0), Point2::new(x1, y1)))
}

#[cfg(test)]
#[path = "tests_viewport.rs"]
mod tests_viewport;
