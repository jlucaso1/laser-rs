use super::svg_doc::SvgDocument;

const MAX_HISTORY_SIZE: usize = 100;

#[derive(Debug)]
pub struct History {
    /// Past states (for undo)
    undo_stack: Vec<SvgDocument>,
    /// Future states (for redo)
    redo_stack: Vec<SvgDocument>,
}

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}

impl History {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    /// Save the current state before making changes
    pub fn save_state(&mut self, document: &SvgDocument) {
        self.undo_stack.push(document.clone());

        // Clear redo stack when new action is performed
        self.redo_stack.clear();

        // Limit history size
        if self.undo_stack.len() > MAX_HISTORY_SIZE {
            self.undo_stack.remove(0);
        }
    }

    /// Undo: restore previous state
    pub fn undo(&mut self, current: &SvgDocument) -> Option<SvgDocument> {
        if let Some(previous) = self.undo_stack.pop() {
            // Save current state to redo stack
            self.redo_stack.push(current.clone());
            Some(previous)
        } else {
            None
        }
    }

    /// Redo: restore next state
    pub fn redo(&mut self, current: &SvgDocument) -> Option<SvgDocument> {
        if let Some(next) = self.redo_stack.pop() {
            // Save current state to undo stack
            self.undo_stack.push(current.clone());
            Some(next)
        } else {
            None
        }
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Clear all history (e.g., when loading a new file)
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    /// Get undo stack size (for UI display)
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    /// Get redo stack size (for UI display)
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }
}
