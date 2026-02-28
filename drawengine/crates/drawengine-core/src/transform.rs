use crate::point::Point;

/// Viewport manages zoom/pan transformations between screen and canvas coordinates.
#[derive(Debug, Clone, Copy)]
pub struct Viewport {
    pub scale: f64,
    pub offset_x: f64,
    pub offset_y: f64,
    pub min_scale: f64,
    pub max_scale: f64,
}

impl Viewport {
    pub fn new() -> Self {
        Self {
            scale: 1.0,
            offset_x: 0.0,
            offset_y: 0.0,
            min_scale: 0.1,
            max_scale: 10.0,
        }
    }

    /// Convert screen coordinates to canvas coordinates.
    pub fn screen_to_canvas(&self, screen: Point) -> Point {
        Point::new(
            (screen.x - self.offset_x) / self.scale,
            (screen.y - self.offset_y) / self.scale,
        )
    }

    /// Convert canvas coordinates to screen coordinates.
    pub fn canvas_to_screen(&self, canvas: Point) -> Point {
        Point::new(
            canvas.x * self.scale + self.offset_x,
            canvas.y * self.scale + self.offset_y,
        )
    }

    /// Zoom toward a focal point (in screen coords).
    pub fn zoom(&mut self, factor: f64, focal_screen: Point) {
        let new_scale = (self.scale * factor).clamp(self.min_scale, self.max_scale);
        let actual_factor = new_scale / self.scale;

        // Adjust offset so the focal point stays fixed
        self.offset_x = focal_screen.x - (focal_screen.x - self.offset_x) * actual_factor;
        self.offset_y = focal_screen.y - (focal_screen.y - self.offset_y) * actual_factor;
        self.scale = new_scale;
    }

    /// Pan by a delta in screen coordinates.
    pub fn pan(&mut self, dx: f64, dy: f64) {
        self.offset_x += dx;
        self.offset_y += dy;
    }

    /// Reset to identity transform.
    pub fn reset(&mut self) {
        self.scale = 1.0;
        self.offset_x = 0.0;
        self.offset_y = 0.0;
    }
}

impl Default for Viewport {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_transform() {
        let vp = Viewport::new();
        let p = Point::new(100.0, 200.0);
        let canvas = vp.screen_to_canvas(p);
        assert!((canvas.x - 100.0).abs() < 1e-9);
        assert!((canvas.y - 200.0).abs() < 1e-9);
    }

    #[test]
    fn test_roundtrip() {
        let mut vp = Viewport::new();
        vp.scale = 2.0;
        vp.offset_x = 50.0;
        vp.offset_y = 30.0;
        let screen = Point::new(150.0, 130.0);
        let canvas = vp.screen_to_canvas(screen);
        let back = vp.canvas_to_screen(canvas);
        assert!((back.x - screen.x).abs() < 1e-9);
        assert!((back.y - screen.y).abs() < 1e-9);
    }

    #[test]
    fn test_zoom_clamp() {
        let mut vp = Viewport::new();
        vp.zoom(100.0, Point::new(0.0, 0.0));
        assert!(vp.scale <= vp.max_scale);
        vp.zoom(0.001, Point::new(0.0, 0.0));
        assert!(vp.scale >= vp.min_scale);
    }

    #[test]
    fn test_pan() {
        let mut vp = Viewport::new();
        vp.pan(10.0, 20.0);
        assert!((vp.offset_x - 10.0).abs() < 1e-9);
        assert!((vp.offset_y - 20.0).abs() < 1e-9);
    }
}
