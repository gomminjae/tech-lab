use std::sync::RwLock;

use drawengine_core::canvas::DrawEngine;

use crate::types::{
    convert_render_command, DrawEngineError, FfiBrushConfig, FfiEngineState, FfiRenderCommand,
};

/// Thread-safe FFI facade over DrawEngine.
/// Uses RwLock for concurrent read (render thread) / write (input thread) access.
#[derive(uniffi::Object)]
pub struct DrawEngineFFI {
    inner: RwLock<DrawEngine>,
}

#[uniffi::export]
impl DrawEngineFFI {
    #[uniffi::constructor]
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            inner: RwLock::new(DrawEngine::new(width, height)),
        }
    }

    // --- Brush ---

    pub fn set_brush(&self, config: FfiBrushConfig) {
        let mut engine = self.inner.write().unwrap();
        engine.set_brush(config.into());
    }

    // --- Drawing ---

    pub fn begin_stroke(
        &self,
        x: f64,
        y: f64,
        pressure: f64,
        timestamp: f64,
    ) -> Vec<FfiRenderCommand> {
        let mut engine = self.inner.write().unwrap();
        engine
            .begin_stroke(x, y, pressure, timestamp)
            .into_iter()
            .map(convert_render_command)
            .collect()
    }

    pub fn add_point(
        &self,
        x: f64,
        y: f64,
        pressure: f64,
        timestamp: f64,
    ) -> Vec<FfiRenderCommand> {
        let mut engine = self.inner.write().unwrap();
        engine
            .add_point(x, y, pressure, timestamp)
            .into_iter()
            .map(convert_render_command)
            .collect()
    }

    pub fn end_stroke(&self) -> Vec<FfiRenderCommand> {
        let mut engine = self.inner.write().unwrap();
        engine
            .end_stroke()
            .into_iter()
            .map(convert_render_command)
            .collect()
    }

    // --- Undo/Redo ---

    pub fn undo(&self) -> Vec<FfiRenderCommand> {
        let mut engine = self.inner.write().unwrap();
        engine.undo().into_iter().map(convert_render_command).collect()
    }

    pub fn redo(&self) -> Vec<FfiRenderCommand> {
        let mut engine = self.inner.write().unwrap();
        engine.redo().into_iter().map(convert_render_command).collect()
    }

    // --- Viewport ---

    pub fn zoom(&self, factor: f64, focal_x: f64, focal_y: f64) -> Vec<FfiRenderCommand> {
        let mut engine = self.inner.write().unwrap();
        engine
            .zoom(factor, focal_x, focal_y)
            .into_iter()
            .map(convert_render_command)
            .collect()
    }

    pub fn pan(&self, dx: f64, dy: f64) -> Vec<FfiRenderCommand> {
        let mut engine = self.inner.write().unwrap();
        engine
            .pan(dx, dy)
            .into_iter()
            .map(convert_render_command)
            .collect()
    }

    pub fn reset_viewport(&self) -> Vec<FfiRenderCommand> {
        let mut engine = self.inner.write().unwrap();
        engine
            .reset_viewport()
            .into_iter()
            .map(convert_render_command)
            .collect()
    }

    // --- Render ---

    pub fn full_render(&self) -> Vec<FfiRenderCommand> {
        let engine = self.inner.read().unwrap();
        engine
            .full_render()
            .into_iter()
            .map(convert_render_command)
            .collect()
    }

    // --- State ---

    pub fn get_state(&self) -> FfiEngineState {
        let engine = self.inner.read().unwrap();
        let (offset_x, offset_y) = engine.get_offset();
        FfiEngineState {
            stroke_count: engine.stroke_count() as u32,
            can_undo: engine.can_undo(),
            can_redo: engine.can_redo(),
            scale: engine.get_scale(),
            offset_x,
            offset_y,
            active_layer_id: engine.active_layer_id().to_string(),
        }
    }

    // --- Serialization ---

    pub fn save(&self) -> Result<String, DrawEngineError> {
        let engine = self.inner.read().unwrap();
        engine.save().map_err(DrawEngineError::from)
    }

    pub fn load(&self, json: String) -> Result<(), DrawEngineError> {
        let mut engine = self.inner.write().unwrap();
        engine.load(&json).map_err(DrawEngineError::from)
    }
}
