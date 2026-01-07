//! Compositor state management
//!
//! This module handles the persistent and runtime state of the compositor.

/// Compositor runtime state
pub struct CompositorState {
    /// Current workspace index
    pub current_workspace: usize,

    /// Total number of workspaces
    pub workspace_count: usize,

    /// Compositor is running
    pub running: bool,

    /// Focus follow mouse
    pub focus_follow_mouse: bool,
}

impl Default for CompositorState {
    fn default() -> Self {
        Self {
            current_workspace: 0,
            workspace_count: 4,
            running: true,
            focus_follow_mouse: false,
        }
    }
}

impl CompositorState {
    /// Create a new compositor state with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Switch to next workspace
    pub fn next_workspace(&mut self) {
        self.current_workspace = (self.current_workspace + 1) % self.workspace_count;
    }

    /// Switch to previous workspace
    pub fn previous_workspace(&mut self) {
        self.current_workspace = if self.current_workspace == 0 {
            self.workspace_count - 1
        } else {
            self.current_workspace - 1
        };
    }

    /// Go to specific workspace
    pub fn goto_workspace(&mut self, index: usize) {
        if index < self.workspace_count {
            self.current_workspace = index;
        }
    }
}
