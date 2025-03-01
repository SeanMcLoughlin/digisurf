use lazy_static::lazy_static;

use crossterm::event::KeyCode;

pub struct KeyBindings {
    pub quit: KeyCode,
    pub help: KeyCode,
    pub up: KeyCode,
    pub down: KeyCode,
    pub left: KeyCode,
    pub right: KeyCode,
    pub zoom_in: KeyCode,
    pub zoom_out: KeyCode,
    pub delete_primary_marker: KeyCode,
    pub delete_secondary_marker: KeyCode,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            quit: KeyCode::Char('q'),
            help: KeyCode::Char('h'),
            up: KeyCode::Up,
            down: KeyCode::Down,
            left: KeyCode::Left,
            right: KeyCode::Right,
            zoom_in: KeyCode::Char('+'),
            zoom_out: KeyCode::Char('-'),
            delete_primary_marker: KeyCode::Delete,
            delete_secondary_marker: KeyCode::Backspace,
        }
    }
}

// This is a singleton for the default keybindings
lazy_static! {
    pub static ref KEYBINDINGS: KeyBindings = KeyBindings::default();
}
