
//! Keyboard event handling for the TUI dashboard.
//!
//! Maps key events to application actions.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::tui::app::AppState;

/// An action the TUI can perform in response to a key event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Quit,
    NextPanel,
    ScrollUp,
    ScrollDown,
    Refresh,
    None,
}

/// Map a key event to an action.
pub fn map_key(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('q') => Action::Quit,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::Quit,
        KeyCode::Tab => Action::NextPanel,
        KeyCode::Char('k') | KeyCode::Up => Action::ScrollUp,
        KeyCode::Char('j') | KeyCode::Down => Action::ScrollDown,
        KeyCode::Char('r') => Action::Refresh,
        _ => Action::None,
    }
}

/// Apply an action to the application state.
pub fn apply_action(state: &mut AppState, action: Action) {
    match action {
        Action::Quit => {
            state.should_quit = true;
        }
        Action::NextPanel => {
            state.next_panel();
        }
        Action::ScrollUp => {
            state.scroll_up();
        }
        Action::ScrollDown => {
            state.scroll_down();
        }
        Action::Refresh => {
            // Caller is responsible for refreshing data from cache
        }
        Action::None => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    fn make_key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    #[test]
    fn test_quit() {
        assert_eq!(map_key(make_key(KeyCode::Char('q'))), Action::Quit);
    }

    #[test]
    fn test_tab() {
        assert_eq!(map_key(make_key(KeyCode::Tab)), Action::NextPanel);
    }

    #[test]
    fn test_scroll() {
        assert_eq!(map_key(make_key(KeyCode::Char('j'))), Action::ScrollDown);
        assert_eq!(map_key(make_key(KeyCode::Char('k'))), Action::ScrollUp);
        assert_eq!(map_key(make_key(KeyCode::Up)), Action::ScrollUp);
        assert_eq!(map_key(make_key(KeyCode::Down)), Action::ScrollDown);
    }

    #[test]
    fn test_refresh() {
        assert_eq!(map_key(make_key(KeyCode::Char('r'))), Action::Refresh);
    }

    #[test]
    fn test_unknown() {
        assert_eq!(map_key(make_key(KeyCode::Char('x'))), Action::None);
    }
}
