/// FFI-safe types that map to UniFFI Records and Enums.
/// These are separate from core types to keep FFI concerns isolated.

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum DrawEngineError {
    #[error("{message}")]
    SerializationError { message: String },
}

impl From<String> for DrawEngineError {
    fn from(s: String) -> Self {
        DrawEngineError::SerializationError { message: s }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiPoint {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiStrokeInput {
    pub x: f64,
    pub y: f64,
    pub pressure: f64,
    pub timestamp: f64,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiBrushConfig {
    pub brush_type: FfiBrushType,
    pub color: FfiColor,
    pub base_width: f64,
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum FfiBrushType {
    Pen,
    Highlighter,
    Eraser,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiPathSegment {
    pub p0_x: f64,
    pub p0_y: f64,
    pub cp1_x: f64,
    pub cp1_y: f64,
    pub cp2_x: f64,
    pub cp2_y: f64,
    pub p3_x: f64,
    pub p3_y: f64,
    pub start_width: f64,
    pub end_width: f64,
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum FfiRenderCommand {
    Clear {
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    },
    SaveState,
    RestoreState,
    SetTransform {
        scale: f64,
        translate_x: f64,
        translate_y: f64,
    },
    DrawVariableWidthPath {
        segments: Vec<FfiPathSegment>,
        r: f32,
        g: f32,
        b: f32,
        a: f32,
        is_eraser: bool,
    },
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiEngineState {
    pub stroke_count: u32,
    pub can_undo: bool,
    pub can_redo: bool,
    pub scale: f64,
    pub offset_x: f64,
    pub offset_y: f64,
    pub active_layer_id: String,
}

// --- Conversion helpers ---

use drawengine_core::brush::{BrushConfig, BrushType};
use drawengine_core::point::Color;
use drawengine_core::render::{PathSegment, RenderCommand};

impl From<FfiColor> for Color {
    fn from(c: FfiColor) -> Self {
        Color::new(c.r, c.g, c.b, c.a)
    }
}

impl From<Color> for FfiColor {
    fn from(c: Color) -> Self {
        FfiColor {
            r: c.r,
            g: c.g,
            b: c.b,
            a: c.a,
        }
    }
}

impl From<FfiBrushType> for BrushType {
    fn from(bt: FfiBrushType) -> Self {
        match bt {
            FfiBrushType::Pen => BrushType::Pen,
            FfiBrushType::Highlighter => BrushType::Highlighter,
            FfiBrushType::Eraser => BrushType::Eraser,
        }
    }
}

impl From<FfiBrushConfig> for BrushConfig {
    fn from(cfg: FfiBrushConfig) -> Self {
        let color: Color = cfg.color.into();
        match cfg.brush_type {
            FfiBrushType::Pen => BrushConfig::pen(color, cfg.base_width),
            FfiBrushType::Highlighter => BrushConfig::highlighter(color, cfg.base_width),
            FfiBrushType::Eraser => BrushConfig::eraser(cfg.base_width),
        }
    }
}

impl From<PathSegment> for FfiPathSegment {
    fn from(s: PathSegment) -> Self {
        FfiPathSegment {
            p0_x: s.p0.x,
            p0_y: s.p0.y,
            cp1_x: s.cp1.x,
            cp1_y: s.cp1.y,
            cp2_x: s.cp2.x,
            cp2_y: s.cp2.y,
            p3_x: s.p3.x,
            p3_y: s.p3.y,
            start_width: s.start_width,
            end_width: s.end_width,
        }
    }
}

pub fn convert_render_command(cmd: RenderCommand) -> FfiRenderCommand {
    match cmd {
        RenderCommand::Clear { color } => FfiRenderCommand::Clear {
            r: color.r,
            g: color.g,
            b: color.b,
            a: color.a,
        },
        RenderCommand::SaveState => FfiRenderCommand::SaveState,
        RenderCommand::RestoreState => FfiRenderCommand::RestoreState,
        RenderCommand::SetTransform {
            scale,
            translate_x,
            translate_y,
        } => FfiRenderCommand::SetTransform {
            scale,
            translate_x,
            translate_y,
        },
        RenderCommand::DrawVariableWidthPath {
            segments,
            color,
            is_eraser,
        } => FfiRenderCommand::DrawVariableWidthPath {
            segments: segments.into_iter().map(Into::into).collect(),
            r: color.r,
            g: color.g,
            b: color.b,
            a: color.a,
            is_eraser,
        },
    }
}
