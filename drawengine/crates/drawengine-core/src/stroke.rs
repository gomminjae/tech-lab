use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::brush::BrushConfig;
use crate::geometry::{catmull_rom_to_bezier, BezierSegment};
use crate::point::{BoundingBox, Color, Point, StrokePoint};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stroke {
    pub id: Uuid,
    pub points: Vec<StrokePoint>,
    pub segments: Vec<SerializableBezierSegment>,
    pub color: Color,
    pub brush: BrushConfig,
    pub bounding_box: BoundingBox,
    pub is_eraser: bool,
}

/// Serializable version of BezierSegment (serde-friendly).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SerializableBezierSegment {
    pub p0: Point,
    pub p1: Point,
    pub p2: Point,
    pub p3: Point,
    pub start_width: f64,
    pub end_width: f64,
}

impl SerializableBezierSegment {
    pub fn to_bezier(&self) -> BezierSegment {
        BezierSegment {
            p0: self.p0,
            p1: self.p1,
            p2: self.p2,
            p3: self.p3,
            start_width: self.start_width,
            end_width: self.end_width,
        }
    }
}

impl From<BezierSegment> for SerializableBezierSegment {
    fn from(b: BezierSegment) -> Self {
        Self {
            p0: b.p0,
            p1: b.p1,
            p2: b.p2,
            p3: b.p3,
            start_width: b.start_width,
            end_width: b.end_width,
        }
    }
}

impl Stroke {
    pub fn new(brush: BrushConfig) -> Self {
        let color = brush.color;
        let is_eraser = brush.brush_type == crate::brush::BrushType::Eraser;
        Self {
            id: Uuid::new_v4(),
            points: Vec::new(),
            segments: Vec::new(),
            color,
            brush,
            bounding_box: BoundingBox::empty(),
            is_eraser,
        }
    }

    pub fn with_id(mut self, id: Uuid) -> Self {
        self.id = id;
        self
    }

    fn recompute_bounding_box(&mut self) {
        let mut bb = BoundingBox::empty();
        for seg in &self.segments {
            for t_step in 0..=10 {
                let t = t_step as f64 / 10.0;
                let bezier = seg.to_bezier();
                let p = bezier.evaluate(t);
                let w = bezier.width_at(t);
                bb.expand_to_include(&p);
                bb.expand_by(w * 0.5);
            }
        }
        if bb.is_valid() {
            self.bounding_box = bb;
        }
    }
}

/// Incrementally builds a Stroke from streaming input points.
pub struct StrokeBuilder {
    stroke: Stroke,
    widths: Vec<f64>,
    last_velocity: f64,
}

impl StrokeBuilder {
    pub fn new(brush: BrushConfig) -> Self {
        Self {
            stroke: Stroke::new(brush),
            widths: Vec::new(),
            last_velocity: 0.0,
        }
    }

    /// Add a point and return new BezierSegments generated (if any).
    pub fn add_point(&mut self, point: StrokePoint) -> Vec<BezierSegment> {
        // Calculate velocity-based width
        let velocity = if let Some(prev) = self.stroke.points.last() {
            prev.speed_to(&point)
        } else {
            0.0
        };
        self.last_velocity = velocity;
        let width = self.stroke.brush.compute_width(point.pressure, velocity);
        self.widths.push(width);
        self.stroke.points.push(point);

        let n = self.stroke.points.len();

        // Need at least 2 points for a segment; use Catmull-Rom with 4 points
        if n < 2 {
            return vec![];
        }

        if n == 2 {
            // Simple linear segment for the first two points
            let p0 = self.stroke.points[0].position;
            let p1 = self.stroke.points[1].position;
            let seg = BezierSegment {
                p0,
                p1: p0.lerp(&p1, 1.0 / 3.0),
                p2: p0.lerp(&p1, 2.0 / 3.0),
                p3: p1,
                start_width: self.widths[0],
                end_width: self.widths[1],
            };
            self.stroke.segments.push(seg.into());
            self.stroke.recompute_bounding_box();
            return vec![seg];
        }

        // For 3+ points, use Catmull-Rom on the latest 4 points (or mirror endpoints)
        let idx = n - 1; // latest point index
        let i2 = idx;
        let i1 = idx - 1;
        let i0 = if idx >= 2 { idx - 2 } else { 0 };
        let i3 = idx; // mirror: next point doesn't exist yet, use current

        let pts = &self.stroke.points;
        let p0 = if i0 == i1 {
            // Mirror: reflect p1 across itself
            pts[i1].position * 2.0 - pts[i2].position
        } else {
            pts[i0].position
        };
        let p3 = pts[i3].position; // For the last segment, p3 == p2 (will be updated)

        // Replace previous last segment with properly computed Catmull-Rom
        if n >= 4 {
            let ip0 = idx - 3;
            let ip1 = idx - 2;
            let ip2 = idx - 1;
            let ip3 = idx;

            let (b0, b1, b2, b3) = catmull_rom_to_bezier(
                pts[ip0].position,
                pts[ip1].position,
                pts[ip2].position,
                pts[ip3].position,
                0.5,
            );
            let seg = BezierSegment {
                p0: b0,
                p1: b1,
                p2: b2,
                p3: b3,
                start_width: self.widths[ip1],
                end_width: self.widths[ip2],
            };
            // Replace the second-to-last segment if it was a placeholder
            if self.stroke.segments.len() >= 2 {
                let replace_idx = self.stroke.segments.len() - 1;
                self.stroke.segments[replace_idx] = seg.into();
            }
        }

        // Add new trailing segment (linear approximation, will be refined on next point)
        let last_seg = BezierSegment {
            p0: pts[i1].position,
            p1: pts[i1].position.lerp(&pts[i2].position, 1.0 / 3.0),
            p2: pts[i1].position.lerp(&pts[i2].position, 2.0 / 3.0),
            p3: pts[i2].position,
            start_width: self.widths[i1],
            end_width: self.widths[i2],
        };
        self.stroke.segments.push(last_seg.into());
        self.stroke.recompute_bounding_box();

        // Return only the newly relevant segments
        let _ = (p0, p3); // suppress unused warnings
        let seg_count = self.stroke.segments.len();
        if seg_count >= 2 {
            vec![
                self.stroke.segments[seg_count - 2].to_bezier(),
                self.stroke.segments[seg_count - 1].to_bezier(),
            ]
        } else {
            vec![self.stroke.segments[seg_count - 1].to_bezier()]
        }
    }

    /// Finalize and return the completed Stroke.
    pub fn finish(mut self) -> Stroke {
        // Refine the last segment with Catmull-Rom if possible
        let n = self.stroke.points.len();
        if n >= 4 {
            let pts = &self.stroke.points;
            let idx = n - 1;
            let (b0, b1, b2, b3) = catmull_rom_to_bezier(
                pts[idx - 3].position,
                pts[idx - 2].position,
                pts[idx - 1].position,
                pts[idx].position,
                0.5,
            );
            let seg = BezierSegment {
                p0: b0,
                p1: b1,
                p2: b2,
                p3: b3,
                start_width: self.widths[idx - 2],
                end_width: self.widths[idx - 1],
            };
            if let Some(last) = self.stroke.segments.last_mut() {
                *last = seg.into();
            }
        }
        self.stroke.recompute_bounding_box();
        self.stroke
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brush::BrushConfig;
    use crate::point::Color;

    #[test]
    fn test_stroke_builder_single_point() {
        let brush = BrushConfig::pen(Color::black(), 2.0);
        let mut builder = StrokeBuilder::new(brush);
        let segs = builder.add_point(StrokePoint::new(10.0, 10.0, 0.5, 0.0));
        assert!(segs.is_empty());
    }

    #[test]
    fn test_stroke_builder_two_points() {
        let brush = BrushConfig::pen(Color::black(), 2.0);
        let mut builder = StrokeBuilder::new(brush);
        builder.add_point(StrokePoint::new(0.0, 0.0, 0.5, 0.0));
        let segs = builder.add_point(StrokePoint::new(10.0, 10.0, 0.5, 0.016));
        assert_eq!(segs.len(), 1);
    }

    #[test]
    fn test_stroke_builder_multiple_points() {
        let brush = BrushConfig::pen(Color::black(), 2.0);
        let mut builder = StrokeBuilder::new(brush);
        for i in 0..10 {
            let t = i as f64;
            builder.add_point(StrokePoint::new(t * 10.0, (t * 0.5).sin() * 20.0, 0.5, t * 0.016));
        }
        let stroke = builder.finish();
        assert!(!stroke.segments.is_empty());
        assert!(stroke.bounding_box.is_valid());
    }

    #[test]
    fn test_stroke_has_id() {
        let stroke = Stroke::new(BrushConfig::default());
        assert!(!stroke.id.is_nil());
    }
}
