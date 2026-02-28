use serde::{Deserialize, Serialize};

use crate::layer::Layer;
use crate::point::Color;

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentData {
    pub version: u32,
    pub width: f64,
    pub height: f64,
    pub background_color: Color,
    pub layers: Vec<Layer>,
}

impl DocumentData {
    pub fn save_to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn load_from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brush::BrushConfig;
    use crate::layer::Layer;
    use crate::point::{Color, StrokePoint};
    use crate::stroke::StrokeBuilder;

    #[test]
    fn test_roundtrip_serialization() {
        let mut layer = Layer::new("Test Layer");
        let brush = BrushConfig::pen(Color::black(), 2.0);
        let mut builder = StrokeBuilder::new(brush);
        for i in 0..5 {
            let t = i as f64;
            builder.add_point(StrokePoint::new(t * 10.0, t * 5.0, 0.5, t * 0.016));
        }
        layer.add_stroke(builder.finish());

        let doc = DocumentData {
            version: 1,
            width: 1920.0,
            height: 1080.0,
            background_color: Color::white(),
            layers: vec![layer],
        };

        let json = doc.save_to_json().unwrap();
        let loaded = DocumentData::load_from_json(&json).unwrap();
        assert_eq!(loaded.version, 1);
        assert_eq!(loaded.layers.len(), 1);
        assert_eq!(loaded.layers[0].strokes.len(), 1);
    }
}
