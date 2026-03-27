
// TUI application state and main loop.
// Stub: will be implemented in Phase 5.

/// Application state for the TUI dashboard.
pub struct AppState {
    /// Whether the application should quit.
    pub should_quit: bool,
    /// Currently selected panel index.
    pub active_panel: usize,
    /// Number of panels.
    panel_count: usize,
    /// Current scroll offset.
    pub scroll_offset: usize,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            should_quit: false,
            active_panel: 0,
            panel_count: 4,
            scroll_offset: 0,
        }
    }

    /// Switch to the next panel.
    pub fn next_panel(&mut self) {
        self.active_panel = (self.active_panel + 1) % self.panel_count;
    }

    /// Scroll up by one line.
    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    /// Scroll down by one line.
    pub fn scroll_down(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_add(1);
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
