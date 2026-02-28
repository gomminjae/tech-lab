use serde::{Deserialize, Serialize};

use crate::point::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BrushType {
    Pen,
    Highlighter,
    Eraser,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BrushConfig {
    pub brush_type: BrushType,
    pub color: Color,
    pub base_width: f64,
    pub min_width_factor: f64,
    pub max_width_factor: f64,
    pub pressure_sensitivity: f64,
    pub velocity_sensitivity: f64,
    pub smoothing: f64,
}

impl BrushConfig {
    pub fn pen(color: Color, width: f64) -> Self {
        Self {
            brush_type: BrushType::Pen,
            color,
            base_width: width,
            min_width_factor: 0.3,
            max_width_factor: 1.5,
            pressure_sensitivity: 0.8,
            velocity_sensitivity: 0.3,
            smoothing: 0.5,
        }
    }

    pub fn highlighter(color: Color, width: f64) -> Self {
        let mut highlight_color = color;
        highlight_color.a = 0.3;
        Self {
            brush_type: BrushType::Highlighter,
            color: highlight_color,
            base_width: width,
            min_width_factor: 0.9,
            max_width_factor: 1.1,
            pressure_sensitivity: 0.1,
            velocity_sensitivity: 0.05,
            smoothing: 0.2,
        }
    }

    pub fn eraser(width: f64) -> Self {
        Self {
            brush_type: BrushType::Eraser,
            color: Color::white(),
            base_width: width,
            min_width_factor: 0.8,
            max_width_factor: 1.2,
            pressure_sensitivity: 0.0,
            velocity_sensitivity: 0.0,
            smoothing: 0.3,
        }
    }

    pub fn compute_width(&self, pressure: f64, velocity: f64) -> f64 {
        let pressure_factor = 1.0 + (pressure - 0.5) * self.pressure_sensitivity;
        let velocity_factor = 1.0 - (velocity.min(1000.0) / 1000.0) * self.velocity_sensitivity;
        let factor = (pressure_factor * velocity_factor)
            .clamp(self.min_width_factor, self.max_width_factor);
        self.base_width * factor
    }
}

impl Default for BrushConfig {
    fn default() -> Self {
        Self::pen(Color::black(), 2.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pen_default() {
        let brush = BrushConfig::default();
        assert_eq!(brush.brush_type, BrushType::Pen);
        assert!((brush.base_width - 2.0).abs() < 1e-9);
    }

    #[test]
    fn test_compute_width_pressure() {
        let brush = BrushConfig::pen(Color::black(), 4.0);
        let w_low = brush.compute_width(0.1, 0.0);
        let w_high = brush.compute_width(0.9, 0.0);
        assert!(w_high > w_low);
    }

    #[test]
    fn test_compute_width_velocity() {
        let brush = BrushConfig::pen(Color::black(), 4.0);
        let w_slow = brush.compute_width(0.5, 0.0);
        let w_fast = brush.compute_width(0.5, 800.0);
        assert!(w_slow > w_fast);
    }

    #[test]
    fn test_width_clamped() {
        let brush = BrushConfig::pen(Color::black(), 4.0);
        let w = brush.compute_width(0.0, 2000.0);
        assert!(w >= brush.base_width * brush.min_width_factor);
        let w = brush.compute_width(1.0, 0.0);
        assert!(w <= brush.base_width * brush.max_width_factor);
    }
}
