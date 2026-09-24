//! Viewport: world (y-up) ↔ screen (y-down) mapping for the Canvas.

use constraint_runtime::Point2;
use eframe::egui::{Pos2, Rect};

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

    pub fn to_screen(&self, p: Point2) -> Pos2 {
        Pos2::new(
            (self.origin_screen.x as f64 + self.scale * p.x) as f32,
            (self.origin_screen.y as f64 - self.scale * p.y) as f32,
        )
    }

    pub fn to_world(&self, s: Pos2) -> Point2 {
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

#[cfg(test)]
#[path = "tests_viewport.rs"]
mod tests_viewport;
