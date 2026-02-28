use crate::point::Point;

/// A cubic Bezier segment with associated start/end widths for variable-width rendering.
#[derive(Debug, Clone, Copy)]
pub struct BezierSegment {
    pub p0: Point,
    pub p1: Point,
    pub p2: Point,
    pub p3: Point,
    pub start_width: f64,
    pub end_width: f64,
}

impl BezierSegment {
    pub fn evaluate(&self, t: f64) -> Point {
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let mt3 = mt2 * mt;
        let t2 = t * t;
        let t3 = t2 * t;
        Point::new(
            mt3 * self.p0.x + 3.0 * mt2 * t * self.p1.x + 3.0 * mt * t2 * self.p2.x + t3 * self.p3.x,
            mt3 * self.p0.y + 3.0 * mt2 * t * self.p1.y + 3.0 * mt * t2 * self.p2.y + t3 * self.p3.y,
        )
    }

    pub fn width_at(&self, t: f64) -> f64 {
        self.start_width + (self.end_width - self.start_width) * t
    }
}

/// Convert four Catmull-Rom control points to a cubic Bezier segment for the middle segment (p1→p2).
/// `alpha` controls the tension (0.5 = centripetal, 0.0 = uniform, 1.0 = chordal).
pub fn catmull_rom_to_bezier(
    p0: Point,
    p1: Point,
    p2: Point,
    p3: Point,
    alpha: f64,
) -> (Point, Point, Point, Point) {
    let d1 = p0.distance_to(&p1).max(1e-6);
    let d2 = p1.distance_to(&p2).max(1e-6);
    let d3 = p2.distance_to(&p3).max(1e-6);

    let d1a = d1.powf(alpha);
    let d2a = d2.powf(alpha);
    let d3a = d3.powf(alpha);
    let d1_2a = d1.powf(2.0 * alpha);
    let d2_2a = d2.powf(2.0 * alpha);
    let d3_2a = d3.powf(2.0 * alpha);

    let b1x = (d1_2a * p2.x - d2_2a * p0.x + (2.0 * d1_2a + 3.0 * d1a * d2a + d2_2a) * p1.x)
        / (3.0 * d1a * (d1a + d2a));
    let b1y = (d1_2a * p2.y - d2_2a * p0.y + (2.0 * d1_2a + 3.0 * d1a * d2a + d2_2a) * p1.y)
        / (3.0 * d1a * (d1a + d2a));

    let b2x = (d3_2a * p1.x - d2_2a * p3.x + (2.0 * d3_2a + 3.0 * d3a * d2a + d2_2a) * p2.x)
        / (3.0 * d3a * (d3a + d2a));
    let b2y = (d3_2a * p1.y - d2_2a * p3.y + (2.0 * d3_2a + 3.0 * d3a * d2a + d2_2a) * p2.y)
        / (3.0 * d3a * (d3a + d2a));

    (p1, Point::new(b1x, b1y), Point::new(b2x, b2y), p2)
}

/// Smooth a sequence of points using a simple moving average filter.
pub fn smooth_points(points: &[Point], factor: f64) -> Vec<Point> {
    if points.len() < 3 || factor < 1e-9 {
        return points.to_vec();
    }

    let mut result = vec![points[0]];
    for i in 1..points.len() - 1 {
        let prev = points[i - 1];
        let curr = points[i];
        let next = points[i + 1];
        let smoothed = Point::new(
            curr.x * (1.0 - factor) + (prev.x + next.x) * 0.5 * factor,
            curr.y * (1.0 - factor) + (prev.y + next.y) * 0.5 * factor,
        );
        result.push(smoothed);
    }
    result.push(*points.last().unwrap());
    result
}

/// Calculate velocity between two points given their timestamps (pixels per second).
pub fn calculate_velocity(p1: Point, t1: f64, p2: Point, t2: f64) -> f64 {
    let dt = (t2 - t1).abs();
    if dt < 1e-9 {
        return 0.0;
    }
    p1.distance_to(&p2) / dt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bezier_evaluate_endpoints() {
        let seg = BezierSegment {
            p0: Point::new(0.0, 0.0),
            p1: Point::new(1.0, 2.0),
            p2: Point::new(3.0, 2.0),
            p3: Point::new(4.0, 0.0),
            start_width: 2.0,
            end_width: 4.0,
        };
        let start = seg.evaluate(0.0);
        let end = seg.evaluate(1.0);
        assert!((start.x - 0.0).abs() < 1e-9);
        assert!((start.y - 0.0).abs() < 1e-9);
        assert!((end.x - 4.0).abs() < 1e-9);
        assert!((end.y - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_catmull_rom_endpoints() {
        let p0 = Point::new(0.0, 0.0);
        let p1 = Point::new(1.0, 1.0);
        let p2 = Point::new(2.0, 0.0);
        let p3 = Point::new(3.0, 1.0);
        let (b0, _b1, _b2, b3) = catmull_rom_to_bezier(p0, p1, p2, p3, 0.5);
        assert!((b0.x - p1.x).abs() < 1e-9);
        assert!((b0.y - p1.y).abs() < 1e-9);
        assert!((b3.x - p2.x).abs() < 1e-9);
        assert!((b3.y - p2.y).abs() < 1e-9);
    }

    #[test]
    fn test_smooth_points() {
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(1.0, 10.0),
            Point::new(2.0, 0.0),
        ];
        let smoothed = smooth_points(&points, 0.5);
        assert_eq!(smoothed.len(), 3);
        // Middle point should be pulled toward the average of neighbors
        assert!(smoothed[1].y < 10.0);
    }

    #[test]
    fn test_calculate_velocity() {
        let v = calculate_velocity(
            Point::new(0.0, 0.0),
            0.0,
            Point::new(3.0, 4.0),
            1.0,
        );
        assert!((v - 5.0).abs() < 1e-9);
    }

    #[test]
    fn test_width_at() {
        let seg = BezierSegment {
            p0: Point::new(0.0, 0.0),
            p1: Point::new(1.0, 0.0),
            p2: Point::new(2.0, 0.0),
            p3: Point::new(3.0, 0.0),
            start_width: 2.0,
            end_width: 6.0,
        };
        assert!((seg.width_at(0.0) - 2.0).abs() < 1e-9);
        assert!((seg.width_at(0.5) - 4.0).abs() < 1e-9);
        assert!((seg.width_at(1.0) - 6.0).abs() < 1e-9);
    }
}
