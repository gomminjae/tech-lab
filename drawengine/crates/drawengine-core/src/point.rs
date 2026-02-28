use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn distance_to(&self, other: &Point) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    pub fn midpoint(&self, other: &Point) -> Point {
        Point {
            x: (self.x + other.x) * 0.5,
            y: (self.y + other.y) * 0.5,
        }
    }

    pub fn lerp(&self, other: &Point, t: f64) -> Point {
        Point {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
        }
    }
}

impl std::ops::Add for Point {
    type Output = Point;
    fn add(self, rhs: Point) -> Point {
        Point::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl std::ops::Sub for Point {
    type Output = Point;
    fn sub(self, rhs: Point) -> Point {
        Point::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl std::ops::Mul<f64> for Point {
    type Output = Point;
    fn mul(self, rhs: f64) -> Point {
        Point::new(self.x * rhs, self.y * rhs)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct StrokePoint {
    pub position: Point,
    pub pressure: f64,
    pub timestamp: f64,
}

impl StrokePoint {
    pub fn new(x: f64, y: f64, pressure: f64, timestamp: f64) -> Self {
        Self {
            position: Point::new(x, y),
            pressure,
            timestamp,
        }
    }

    pub fn speed_to(&self, other: &StrokePoint) -> f64 {
        let dist = self.position.distance_to(&other.position);
        let dt = (other.timestamp - self.timestamp).abs();
        if dt < 1e-9 {
            0.0
        } else {
            dist / dt
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn black() -> Self {
        Self::new(0.0, 0.0, 0.0, 1.0)
    }

    pub fn white() -> Self {
        Self::new(1.0, 1.0, 1.0, 1.0)
    }

    pub fn from_hex(hex: u32) -> Self {
        let r = ((hex >> 16) & 0xFF) as f32 / 255.0;
        let g = ((hex >> 8) & 0xFF) as f32 / 255.0;
        let b = (hex & 0xFF) as f32 / 255.0;
        Self::new(r, g, b, 1.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

impl BoundingBox {
    pub fn empty() -> Self {
        Self {
            min_x: f64::MAX,
            min_y: f64::MAX,
            max_x: f64::MIN,
            max_y: f64::MIN,
        }
    }

    pub fn from_points(points: &[Point]) -> Self {
        let mut bb = Self::empty();
        for p in points {
            bb.expand_to_include(p);
        }
        bb
    }

    pub fn expand_to_include(&mut self, point: &Point) {
        self.min_x = self.min_x.min(point.x);
        self.min_y = self.min_y.min(point.y);
        self.max_x = self.max_x.max(point.x);
        self.max_y = self.max_y.max(point.y);
    }

    pub fn expand_by(&mut self, margin: f64) {
        self.min_x -= margin;
        self.min_y -= margin;
        self.max_x += margin;
        self.max_y += margin;
    }

    pub fn union(&self, other: &BoundingBox) -> BoundingBox {
        BoundingBox {
            min_x: self.min_x.min(other.min_x),
            min_y: self.min_y.min(other.min_y),
            max_x: self.max_x.max(other.max_x),
            max_y: self.max_y.max(other.max_y),
        }
    }

    pub fn intersects(&self, other: &BoundingBox) -> bool {
        self.min_x <= other.max_x
            && self.max_x >= other.min_x
            && self.min_y <= other.max_y
            && self.max_y >= other.min_y
    }

    pub fn contains_point(&self, point: &Point) -> bool {
        point.x >= self.min_x
            && point.x <= self.max_x
            && point.y >= self.min_y
            && point.y <= self.max_y
    }

    pub fn width(&self) -> f64 {
        self.max_x - self.min_x
    }

    pub fn height(&self) -> f64 {
        self.max_y - self.min_y
    }

    pub fn is_valid(&self) -> bool {
        self.min_x <= self.max_x && self.min_y <= self.max_y
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_distance() {
        let a = Point::new(0.0, 0.0);
        let b = Point::new(3.0, 4.0);
        assert!((a.distance_to(&b) - 5.0).abs() < 1e-9);
    }

    #[test]
    fn test_point_lerp() {
        let a = Point::new(0.0, 0.0);
        let b = Point::new(10.0, 10.0);
        let mid = a.lerp(&b, 0.5);
        assert!((mid.x - 5.0).abs() < 1e-9);
        assert!((mid.y - 5.0).abs() < 1e-9);
    }

    #[test]
    fn test_bounding_box() {
        let points = vec![
            Point::new(1.0, 2.0),
            Point::new(5.0, 8.0),
            Point::new(3.0, 4.0),
        ];
        let bb = BoundingBox::from_points(&points);
        assert!((bb.min_x - 1.0).abs() < 1e-9);
        assert!((bb.min_y - 2.0).abs() < 1e-9);
        assert!((bb.max_x - 5.0).abs() < 1e-9);
        assert!((bb.max_y - 8.0).abs() < 1e-9);
    }

    #[test]
    fn test_color_from_hex() {
        let c = Color::from_hex(0xFF0000);
        assert!((c.r - 1.0).abs() < 0.01);
        assert!((c.g - 0.0).abs() < 0.01);
        assert!((c.b - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_stroke_point_speed() {
        let a = StrokePoint::new(0.0, 0.0, 1.0, 0.0);
        let b = StrokePoint::new(3.0, 4.0, 1.0, 1.0);
        assert!((a.speed_to(&b) - 5.0).abs() < 1e-9);
    }
}
