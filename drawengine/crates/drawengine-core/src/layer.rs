use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::point::BoundingBox;
use crate::stroke::Stroke;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    pub id: Uuid,
    pub name: String,
    pub visible: bool,
    pub opacity: f32,
    pub strokes: Vec<Stroke>,
}

impl Layer {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            visible: true,
            opacity: 1.0,
            strokes: Vec::new(),
        }
    }

    pub fn add_stroke(&mut self, stroke: Stroke) {
        self.strokes.push(stroke);
    }

    pub fn remove_stroke(&mut self, stroke_id: Uuid) -> Option<Stroke> {
        if let Some(idx) = self.strokes.iter().position(|s| s.id == stroke_id) {
            Some(self.strokes.remove(idx))
        } else {
            None
        }
    }

    pub fn bounding_box(&self) -> BoundingBox {
        let mut bb = BoundingBox::empty();
        for stroke in &self.strokes {
            if stroke.bounding_box.is_valid() {
                bb = bb.union(&stroke.bounding_box);
            }
        }
        bb
    }
}

pub struct LayerManager {
    pub layers: Vec<Layer>,
    pub active_layer_index: usize,
}

impl LayerManager {
    pub fn new() -> Self {
        let default_layer = Layer::new("Layer 1");
        Self {
            layers: vec![default_layer],
            active_layer_index: 0,
        }
    }

    pub fn active_layer(&self) -> &Layer {
        &self.layers[self.active_layer_index]
    }

    pub fn active_layer_mut(&mut self) -> &mut Layer {
        &mut self.layers[self.active_layer_index]
    }

    pub fn active_layer_id(&self) -> Uuid {
        self.layers[self.active_layer_index].id
    }

    pub fn all_visible_strokes(&self) -> Vec<&Stroke> {
        self.layers
            .iter()
            .filter(|l| l.visible)
            .flat_map(|l| l.strokes.iter())
            .collect()
    }

    pub fn find_stroke_layer(&self, stroke_id: Uuid) -> Option<usize> {
        self.layers
            .iter()
            .position(|l| l.strokes.iter().any(|s| s.id == stroke_id))
    }
}

impl Default for LayerManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brush::BrushConfig;
    use crate::stroke::Stroke;

    #[test]
    fn test_layer_add_remove_stroke() {
        let mut layer = Layer::new("Test");
        let stroke = Stroke::new(BrushConfig::default());
        let id = stroke.id;
        layer.add_stroke(stroke);
        assert_eq!(layer.strokes.len(), 1);
        let removed = layer.remove_stroke(id);
        assert!(removed.is_some());
        assert_eq!(layer.strokes.len(), 0);
    }

    #[test]
    fn test_layer_manager_default() {
        let mgr = LayerManager::new();
        assert_eq!(mgr.layers.len(), 1);
        assert_eq!(mgr.active_layer_index, 0);
    }

    #[test]
    fn test_all_visible_strokes() {
        let mut mgr = LayerManager::new();
        let stroke = Stroke::new(BrushConfig::default());
        mgr.active_layer_mut().add_stroke(stroke);
        let visible = mgr.all_visible_strokes();
        assert_eq!(visible.len(), 1);
    }
}
