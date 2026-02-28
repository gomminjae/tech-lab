use crate::point::{BoundingBox, Point};
use crate::stroke::Stroke;

/// Stroke-level eraser: removes entire strokes that intersect with the eraser path.
pub fn find_strokes_to_erase(
    strokes: &[Stroke],
    eraser_point: Point,
    eraser_radius: f64,
) -> Vec<uuid::Uuid> {
    let eraser_bb = BoundingBox {
        min_x: eraser_point.x - eraser_radius,
        min_y: eraser_point.y - eraser_radius,
        max_x: eraser_point.x + eraser_radius,
        max_y: eraser_point.y + eraser_radius,
    };

    let mut to_erase = Vec::new();

    for stroke in strokes {
        if stroke.is_eraser {
            continue;
        }
        if !stroke.bounding_box.is_valid() {
            continue;
        }
        if !stroke.bounding_box.intersects(&eraser_bb) {
            continue;
        }

        // Check each segment's sample points against eraser circle
        for seg in &stroke.segments {
            let bezier = seg.to_bezier();
            let mut hit = false;
            for step in 0..=20 {
                let t = step as f64 / 20.0;
                let p = bezier.evaluate(t);
                if p.distance_to(&eraser_point) <= eraser_radius + bezier.width_at(t) * 0.5 {
                    hit = true;
                    break;
                }
            }
            if hit {
                to_erase.push(stroke.id);
                break;
            }
        }
    }

    to_erase
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brush::BrushConfig;
    use crate::point::{Color, StrokePoint};
    use crate::stroke::StrokeBuilder;

    fn make_test_stroke() -> Stroke {
        let brush = BrushConfig::pen(Color::black(), 2.0);
        let mut builder = StrokeBuilder::new(brush);
        for i in 0..5 {
            let t = i as f64;
            builder.add_point(StrokePoint::new(t * 10.0, 0.0, 0.5, t * 0.016));
        }
        builder.finish()
    }

    #[test]
    fn test_erase_hit() {
        let stroke = make_test_stroke();
        let ids = find_strokes_to_erase(&[stroke], Point::new(20.0, 0.0), 5.0);
        assert_eq!(ids.len(), 1);
    }

    #[test]
    fn test_erase_miss() {
        let stroke = make_test_stroke();
        let ids = find_strokes_to_erase(&[stroke], Point::new(200.0, 200.0), 5.0);
        assert!(ids.is_empty());
    }
}
