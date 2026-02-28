use uuid::Uuid;

use crate::brush::{BrushConfig, BrushType};
use crate::eraser::find_strokes_to_erase;
use crate::geometry::BezierSegment;
use crate::history::{History, HistoryAction};
use crate::layer::LayerManager;
use crate::point::{Color, Point, StrokePoint};
use crate::render::{
    generate_full_render_commands, generate_incremental_commands, RenderCommand,
};
use crate::serialization::DocumentData;
use crate::stroke::{Stroke, StrokeBuilder};
use crate::transform::Viewport;

pub struct DrawEngine {
    pub layer_manager: LayerManager,
    pub viewport: Viewport,
    pub history: History,
    pub background_color: Color,
    pub canvas_width: f64,
    pub canvas_height: f64,

    current_brush: BrushConfig,
    active_builder: Option<StrokeBuilder>,
}

impl DrawEngine {
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            layer_manager: LayerManager::new(),
            viewport: Viewport::new(),
            history: History::default(),
            background_color: Color::white(),
            canvas_width: width,
            canvas_height: height,
            current_brush: BrushConfig::default(),
            active_builder: None,
        }
    }

    // --- Brush ---

    pub fn set_brush(&mut self, brush: BrushConfig) {
        self.current_brush = brush;
    }

    pub fn current_brush(&self) -> &BrushConfig {
        &self.current_brush
    }

    // --- Drawing ---

    /// Begin a new stroke at the given screen-space point.
    pub fn begin_stroke(&mut self, screen_x: f64, screen_y: f64, pressure: f64, timestamp: f64) -> Vec<RenderCommand> {
        let canvas_point = self.viewport.screen_to_canvas(Point::new(screen_x, screen_y));
        let point = StrokePoint::new(canvas_point.x, canvas_point.y, pressure, timestamp);

        let mut builder = StrokeBuilder::new(self.current_brush.clone());
        let _segments = builder.add_point(point);
        self.active_builder = Some(builder);

        // No segments yet on first point
        vec![]
    }

    /// Add a point to the current stroke. Returns incremental render commands.
    pub fn add_point(&mut self, screen_x: f64, screen_y: f64, pressure: f64, timestamp: f64) -> Vec<RenderCommand> {
        let canvas_point = self.viewport.screen_to_canvas(Point::new(screen_x, screen_y));
        let point = StrokePoint::new(canvas_point.x, canvas_point.y, pressure, timestamp);

        if let Some(builder) = &mut self.active_builder {
            let new_segments: Vec<BezierSegment> = builder.add_point(point);
            if self.current_brush.brush_type == BrushType::Eraser {
                // For eraser, check intersections but don't render the eraser stroke
                return vec![];
            }
            generate_incremental_commands(
                &new_segments,
                self.current_brush.color,
                false,
            )
        } else {
            vec![]
        }
    }

    /// End the current stroke. Returns full render commands for a clean redraw.
    pub fn end_stroke(&mut self) -> Vec<RenderCommand> {
        if let Some(builder) = self.active_builder.take() {
            let stroke = builder.finish();

            if self.current_brush.brush_type == BrushType::Eraser {
                // Erase strokes that intersect with the eraser path
                let layer = self.layer_manager.active_layer();
                let mut erased_ids = Vec::new();
                for sp in &stroke.points {
                    let width = self.current_brush.compute_width(sp.pressure, 0.0);
                    let ids = find_strokes_to_erase(
                        &layer.strokes,
                        sp.position,
                        width * 0.5,
                    );
                    for id in ids {
                        if !erased_ids.contains(&id) {
                            erased_ids.push(id);
                        }
                    }
                }

                let layer_idx = self.layer_manager.active_layer_index;
                for id in erased_ids {
                    if let Some(removed) = self.layer_manager.active_layer_mut().remove_stroke(id) {
                        self.history.push(HistoryAction::RemoveStroke {
                            layer_index: layer_idx,
                            stroke: removed,
                        });
                    }
                }
            } else if !stroke.segments.is_empty() {
                let layer_idx = self.layer_manager.active_layer_index;
                self.history.push(HistoryAction::AddStroke {
                    layer_index: layer_idx,
                    stroke: stroke.clone(),
                });
                self.layer_manager.active_layer_mut().add_stroke(stroke);
            }
        }

        self.full_render()
    }

    // --- Undo/Redo ---

    pub fn undo(&mut self) -> Vec<RenderCommand> {
        if let Some(action) = self.history.undo() {
            self.apply_history_action(&action);
        }
        self.full_render()
    }

    pub fn redo(&mut self) -> Vec<RenderCommand> {
        if let Some(action) = self.history.redo() {
            self.apply_history_action(&action);
        }
        self.full_render()
    }

    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    fn apply_history_action(&mut self, action: &HistoryAction) {
        match action {
            HistoryAction::AddStroke {
                layer_index,
                stroke,
            } => {
                if let Some(layer) = self.layer_manager.layers.get_mut(*layer_index) {
                    layer.add_stroke(stroke.clone());
                }
            }
            HistoryAction::RemoveStroke {
                layer_index,
                stroke,
            } => {
                if let Some(layer) = self.layer_manager.layers.get_mut(*layer_index) {
                    layer.remove_stroke(stroke.id);
                }
            }
        }
    }

    // --- Viewport ---

    pub fn zoom(&mut self, factor: f64, focal_x: f64, focal_y: f64) -> Vec<RenderCommand> {
        self.viewport.zoom(factor, Point::new(focal_x, focal_y));
        self.full_render()
    }

    pub fn pan(&mut self, dx: f64, dy: f64) -> Vec<RenderCommand> {
        self.viewport.pan(dx, dy);
        self.full_render()
    }

    pub fn reset_viewport(&mut self) -> Vec<RenderCommand> {
        self.viewport.reset();
        self.full_render()
    }

    pub fn get_scale(&self) -> f64 {
        self.viewport.scale
    }

    pub fn get_offset(&self) -> (f64, f64) {
        (self.viewport.offset_x, self.viewport.offset_y)
    }

    // --- Render ---

    pub fn full_render(&self) -> Vec<RenderCommand> {
        let strokes: Vec<&crate::stroke::Stroke> = self.layer_manager.all_visible_strokes();
        let owned: Vec<Stroke> = strokes.into_iter().cloned().collect();
        generate_full_render_commands(
            &owned,
            self.background_color,
            self.viewport.scale,
            self.viewport.offset_x,
            self.viewport.offset_y,
        )
    }

    // --- Serialization ---

    pub fn save(&self) -> Result<String, String> {
        let data = DocumentData {
            version: 1,
            width: self.canvas_width,
            height: self.canvas_height,
            background_color: self.background_color,
            layers: self.layer_manager.layers.clone(),
        };
        data.save_to_json().map_err(|e| e.to_string())
    }

    pub fn load(&mut self, json: &str) -> Result<(), String> {
        let data = DocumentData::load_from_json(json).map_err(|e| e.to_string())?;
        self.canvas_width = data.width;
        self.canvas_height = data.height;
        self.background_color = data.background_color;
        self.layer_manager = LayerManager::new();
        self.layer_manager.layers = data.layers;
        if self.layer_manager.layers.is_empty() {
            self.layer_manager = LayerManager::new();
        }
        self.history.clear();
        Ok(())
    }

    // --- Info ---

    pub fn stroke_count(&self) -> usize {
        self.layer_manager
            .layers
            .iter()
            .map(|l| l.strokes.len())
            .sum()
    }

    pub fn active_layer_id(&self) -> Uuid {
        self.layer_manager.active_layer_id()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brush::BrushConfig;
    use crate::point::Color;

    #[test]
    fn test_engine_new() {
        let engine = DrawEngine::new(1920.0, 1080.0);
        assert_eq!(engine.stroke_count(), 0);
        assert!(!engine.can_undo());
    }

    #[test]
    fn test_draw_stroke() {
        let mut engine = DrawEngine::new(1920.0, 1080.0);
        engine.set_brush(BrushConfig::pen(Color::black(), 3.0));

        engine.begin_stroke(100.0, 100.0, 0.5, 0.0);
        engine.add_point(110.0, 105.0, 0.6, 0.016);
        engine.add_point(120.0, 110.0, 0.7, 0.032);
        engine.add_point(130.0, 108.0, 0.6, 0.048);
        let cmds = engine.end_stroke();

        assert_eq!(engine.stroke_count(), 1);
        assert!(engine.can_undo());
        assert!(!cmds.is_empty());
    }

    #[test]
    fn test_undo_redo() {
        let mut engine = DrawEngine::new(1920.0, 1080.0);

        engine.begin_stroke(10.0, 10.0, 0.5, 0.0);
        engine.add_point(20.0, 20.0, 0.5, 0.016);
        engine.end_stroke();
        assert_eq!(engine.stroke_count(), 1);

        engine.undo();
        assert_eq!(engine.stroke_count(), 0);

        engine.redo();
        assert_eq!(engine.stroke_count(), 1);
    }

    #[test]
    fn test_zoom_pan() {
        let mut engine = DrawEngine::new(1920.0, 1080.0);
        engine.zoom(2.0, 960.0, 540.0);
        assert!((engine.get_scale() - 2.0).abs() < 1e-9);

        engine.pan(50.0, 30.0);
        let (ox, oy) = engine.get_offset();
        assert!(ox.abs() > 0.0 || oy.abs() > 0.0);
    }

    #[test]
    fn test_save_load() {
        let mut engine = DrawEngine::new(1920.0, 1080.0);
        engine.begin_stroke(10.0, 10.0, 0.5, 0.0);
        engine.add_point(20.0, 20.0, 0.5, 0.016);
        engine.add_point(30.0, 30.0, 0.5, 0.032);
        engine.end_stroke();

        let json = engine.save().unwrap();

        let mut engine2 = DrawEngine::new(800.0, 600.0);
        engine2.load(&json).unwrap();
        assert_eq!(engine2.stroke_count(), 1);
        assert!((engine2.canvas_width - 1920.0).abs() < 1e-9);
    }

    #[test]
    fn test_eraser() {
        let mut engine = DrawEngine::new(1920.0, 1080.0);

        // Draw a stroke
        engine.set_brush(BrushConfig::pen(Color::black(), 3.0));
        engine.begin_stroke(10.0, 10.0, 0.5, 0.0);
        engine.add_point(20.0, 10.0, 0.5, 0.016);
        engine.add_point(30.0, 10.0, 0.5, 0.032);
        engine.end_stroke();
        assert_eq!(engine.stroke_count(), 1);

        // Erase it
        engine.set_brush(BrushConfig::eraser(20.0));
        engine.begin_stroke(15.0, 10.0, 0.5, 0.1);
        engine.add_point(25.0, 10.0, 0.5, 0.116);
        engine.end_stroke();
        assert_eq!(engine.stroke_count(), 0);

        // Undo should bring it back
        engine.undo();
        assert_eq!(engine.stroke_count(), 1);
    }
}
