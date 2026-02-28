use uuid::Uuid;

use crate::stroke::Stroke;

#[derive(Debug, Clone)]
pub enum HistoryAction {
    AddStroke {
        layer_index: usize,
        stroke: Stroke,
    },
    RemoveStroke {
        layer_index: usize,
        stroke: Stroke,
    },
}

impl HistoryAction {
    pub fn inverse(&self) -> HistoryAction {
        match self {
            HistoryAction::AddStroke {
                layer_index,
                stroke,
            } => HistoryAction::RemoveStroke {
                layer_index: *layer_index,
                stroke: stroke.clone(),
            },
            HistoryAction::RemoveStroke {
                layer_index,
                stroke,
            } => HistoryAction::AddStroke {
                layer_index: *layer_index,
                stroke: stroke.clone(),
            },
        }
    }

    pub fn stroke_id(&self) -> Uuid {
        match self {
            HistoryAction::AddStroke { stroke, .. } => stroke.id,
            HistoryAction::RemoveStroke { stroke, .. } => stroke.id,
        }
    }
}

pub struct History {
    undo_stack: Vec<HistoryAction>,
    redo_stack: Vec<HistoryAction>,
    max_size: usize,
}

impl History {
    pub fn new(max_size: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_size,
        }
    }

    pub fn push(&mut self, action: HistoryAction) {
        self.redo_stack.clear();
        self.undo_stack.push(action);
        if self.undo_stack.len() > self.max_size {
            self.undo_stack.remove(0);
        }
    }

    pub fn undo(&mut self) -> Option<HistoryAction> {
        if let Some(action) = self.undo_stack.pop() {
            let inverse = action.inverse();
            self.redo_stack.push(action);
            Some(inverse)
        } else {
            None
        }
    }

    pub fn redo(&mut self) -> Option<HistoryAction> {
        if let Some(action) = self.redo_stack.pop() {
            let to_apply = action.clone();
            self.undo_stack.push(action);
            Some(to_apply)
        } else {
            None
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}

impl Default for History {
    fn default() -> Self {
        Self::new(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brush::BrushConfig;
    use crate::stroke::Stroke;

    fn make_stroke() -> Stroke {
        Stroke::new(BrushConfig::default())
    }

    #[test]
    fn test_undo_redo() {
        let mut history = History::new(10);
        let stroke = make_stroke();
        history.push(HistoryAction::AddStroke {
            layer_index: 0,
            stroke: stroke.clone(),
        });
        assert!(history.can_undo());
        assert!(!history.can_redo());

        let inverse = history.undo().unwrap();
        match inverse {
            HistoryAction::RemoveStroke { stroke: s, .. } => {
                assert_eq!(s.id, stroke.id);
            }
            _ => panic!("Expected RemoveStroke"),
        }
        assert!(!history.can_undo());
        assert!(history.can_redo());

        let redo_action = history.redo().unwrap();
        match redo_action {
            HistoryAction::AddStroke { stroke: s, .. } => {
                assert_eq!(s.id, stroke.id);
            }
            _ => panic!("Expected AddStroke"),
        }
    }

    #[test]
    fn test_push_clears_redo() {
        let mut history = History::new(10);
        history.push(HistoryAction::AddStroke {
            layer_index: 0,
            stroke: make_stroke(),
        });
        history.undo();
        assert!(history.can_redo());

        history.push(HistoryAction::AddStroke {
            layer_index: 0,
            stroke: make_stroke(),
        });
        assert!(!history.can_redo());
    }

    #[test]
    fn test_max_size() {
        let mut history = History::new(3);
        for _ in 0..5 {
            history.push(HistoryAction::AddStroke {
                layer_index: 0,
                stroke: make_stroke(),
            });
        }
        let mut count = 0;
        while history.undo().is_some() {
            count += 1;
        }
        assert_eq!(count, 3);
    }
}
