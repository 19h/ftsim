//! # ftsim-tui::input
//!
//! Handles user keyboard input and maps it to actions within the TUI app.

use crate::app::App;
use crossterm::event::{KeyCode, KeyEvent};

/// Handles a key press event and updates the app state accordingly.
pub fn handle_key_press(key: KeyEvent, app: &mut App) {
    match key.code {
        KeyCode::Char('?') => {
            app.toggle_help();
        }
        KeyCode::Char(' ') => {
            app.toggle_pause();
        }
        KeyCode::Char('.') => {
            app.single_step();
        }
        KeyCode::Char('p') => {
            app.inject_partition();
        }
        KeyCode::Char('k') => {
            app.kill_node();
        }
        KeyCode::Char('r') => {
            app.restart_node();
        }
        KeyCode::Char('/') => {
            app.toggle_filter_logs();
        }
        KeyCode::Tab => {
            app.cycle_focus();
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use ftsim_engine::control::ControlMsg;

    fn create_test_app() -> App {
        let (tx, _rx) = crossbeam_channel::unbounded::<ControlMsg>();
        App::new(tx)
    }

    #[test]
    fn test_help_key() {
        let mut app = create_test_app();
        assert!(!app.show_help);
        
        let key = KeyEvent::new(KeyCode::Char('?'), KeyModifiers::empty());
        handle_key_press(key, &mut app);
        assert!(app.show_help);
        
        // Toggle back
        handle_key_press(key, &mut app);
        assert!(!app.show_help);
    }

    #[test]
    fn test_pause_key() {
        let mut app = create_test_app();
        assert!(!app.is_paused);
        
        let key = KeyEvent::new(KeyCode::Char(' '), KeyModifiers::empty());
        handle_key_press(key, &mut app);
        assert!(app.is_paused);
        
        // Toggle back
        handle_key_press(key, &mut app);
        assert!(!app.is_paused);
    }

    #[test]
    fn test_filter_logs_key() {
        let mut app = create_test_app();
        assert!(!app.filter_logs);
        
        let key = KeyEvent::new(KeyCode::Char('/'), KeyModifiers::empty());
        handle_key_press(key, &mut app);
        assert!(app.filter_logs);
        
        // Toggle back
        handle_key_press(key, &mut app);
        assert!(!app.filter_logs);
    }

    #[test]
    fn test_tab_key() {
        let mut app = create_test_app();
        assert_eq!(app.focused_panel, 0);
        
        let key = KeyEvent::new(KeyCode::Tab, KeyModifiers::empty());
        handle_key_press(key, &mut app);
        assert_eq!(app.focused_panel, 1);
        
        handle_key_press(key, &mut app);
        assert_eq!(app.focused_panel, 2);
        
        handle_key_press(key, &mut app);
        assert_eq!(app.focused_panel, 3);
        
        // Should wrap around
        handle_key_press(key, &mut app);
        assert_eq!(app.focused_panel, 0);
    }

    #[test]
    fn test_all_keys_handled() {
        let mut app = create_test_app();
        
        // Test that all documented keys are handled without panicking
        let keys = vec![
            KeyEvent::new(KeyCode::Char('?'), KeyModifiers::empty()),
            KeyEvent::new(KeyCode::Char(' '), KeyModifiers::empty()),
            KeyEvent::new(KeyCode::Char('.'), KeyModifiers::empty()),
            KeyEvent::new(KeyCode::Char('p'), KeyModifiers::empty()),
            KeyEvent::new(KeyCode::Char('k'), KeyModifiers::empty()),
            KeyEvent::new(KeyCode::Char('r'), KeyModifiers::empty()),
            KeyEvent::new(KeyCode::Char('/'), KeyModifiers::empty()),
            KeyEvent::new(KeyCode::Tab, KeyModifiers::empty()),
            // Test an unhandled key
            KeyEvent::new(KeyCode::Char('x'), KeyModifiers::empty()),
        ];
        
        for key in keys {
            // Should not panic
            handle_key_press(key, &mut app);
        }
    }
}
