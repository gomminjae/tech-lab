use crate::geometry::BezierSegment;
use crate::point::{Color, Point};
use crate::stroke::Stroke;

/// Commands consumed by native renderers (Android Canvas / iOS CoreGraphics).
#[derive(Debug, Clone)]
pub enum RenderCommand {
    Clear {
        color: Color,
    },
    SaveState,
    RestoreState,
    SetTransform {
        scale: f64,
        translate_x: f64,
        translate_y: f64,
    },
    DrawVariableWidthPath {
        segments: Vec<PathSegment>,
        color: Color,
        is_eraser: bool,
    },
}

/// A single Bezier path segment with width info for variable-width rendering.
#[derive(Debug, Clone, Copy)]
pub struct PathSegment {
    pub p0: Point,
    pub cp1: Point,
    pub cp2: Point,
    pub p3: Point,
    pub start_width: f64,
    pub end_width: f64,
}

impl From<BezierSegment> for PathSegment {
    fn from(b: BezierSegment) -> Self {
        Self {
            p0: b.p0,
            cp1: b.p1,
            cp2: b.p2,
            p3: b.p3,
            start_width: b.start_width,
            end_width: b.end_width,
        }
    }
}

/// Generate render commands for a full scene redraw.
pub fn generate_full_render_commands(
    strokes: &[Stroke],
    bg_color: Color,
    scale: f64,
    translate_x: f64,
    translate_y: f64,
) -> Vec<RenderCommand> {
    let mut commands = Vec::new();

    commands.push(RenderCommand::Clear { color: bg_color });
    commands.push(RenderCommand::SaveState);
    commands.push(RenderCommand::SetTransform {
        scale,
        translate_x,
        translate_y,
    });

    for stroke in strokes {
        if stroke.segments.is_empty() {
            continue;
        }
        let segments: Vec<PathSegment> = stroke
            .segments
            .iter()
            .map(|s| s.to_bezier().into())
            .collect();
        commands.push(RenderCommand::DrawVariableWidthPath {
            segments,
            color: stroke.color,
            is_eraser: stroke.is_eraser,
        });
    }

    commands.push(RenderCommand::RestoreState);
    commands
}

/// Generate incremental render commands for newly added segments during drawing.
pub fn generate_incremental_commands(
    new_segments: &[BezierSegment],
    color: Color,
    is_eraser: bool,
) -> Vec<RenderCommand> {
    if new_segments.is_empty() {
        return vec![];
    }
    let segments: Vec<PathSegment> = new_segments.iter().copied().map(Into::into).collect();
    vec![RenderCommand::DrawVariableWidthPath {
        segments,
        color,
        is_eraser,
    }]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brush::BrushConfig;
    use crate::point::{Color, StrokePoint};
    use crate::stroke::StrokeBuilder;

    #[test]
    fn test_full_render_commands_empty() {
        let cmds = generate_full_render_commands(&[], Color::white(), 1.0, 0.0, 0.0);
        assert_eq!(cmds.len(), 4); // Clear, SaveState, SetTransform, RestoreState
    }

    #[test]
    fn test_full_render_commands_with_stroke() {
        let brush = BrushConfig::pen(Color::black(), 2.0);
        let mut builder = StrokeBuilder::new(brush);
        for i in 0..5 {
            let t = i as f64;
            builder.add_point(StrokePoint::new(t * 10.0, t * 5.0, 0.5, t * 0.016));
        }
        let stroke = builder.finish();
        let cmds = generate_full_render_commands(&[stroke], Color::white(), 1.0, 0.0, 0.0);
        // Clear + SaveState + SetTransform + DrawPath + RestoreState
        assert!(cmds.len() >= 4);
    }

    #[test]
    fn test_incremental_commands() {
        let seg = BezierSegment {
            p0: Point::new(0.0, 0.0),
            p1: Point::new(1.0, 1.0),
            p2: Point::new(2.0, 1.0),
            p3: Point::new(3.0, 0.0),
            start_width: 2.0,
            end_width: 3.0,
        };
        let cmds = generate_incremental_commands(&[seg], Color::black(), false);
        assert_eq!(cmds.len(), 1);
    }
}
