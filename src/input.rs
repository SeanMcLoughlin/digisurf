use crossterm::event::KeyCode;
use lazy_static::lazy_static;

pub struct NormalModeKeyBindings {
    pub enter_command_mode: KeyCode,
    pub up: KeyCode,
    pub down: KeyCode,
    pub left: KeyCode,
    pub right: KeyCode,
    pub zoom_in: KeyCode,
    pub zoom_out: KeyCode,
    pub zoom_full: KeyCode,
    pub delete_primary_marker: KeyCode,
    pub delete_secondary_marker: KeyCode,
}

impl Default for NormalModeKeyBindings {
    fn default() -> Self {
        Self {
            enter_command_mode: KeyCode::Char(':'),
            up: KeyCode::Up,
            down: KeyCode::Down,
            left: KeyCode::Left,
            right: KeyCode::Right,
            zoom_in: KeyCode::Char('+'),
            zoom_out: KeyCode::Char('-'),
            zoom_full: KeyCode::Char('0'),
            delete_primary_marker: KeyCode::Delete,
            delete_secondary_marker: KeyCode::Backspace,
        }
    }
}

pub struct CommandModeKeyBindings {
    pub enter_normal_mode: KeyCode,
    pub execute_command: KeyCode,
}

impl Default for CommandModeKeyBindings {
    fn default() -> Self {
        Self {
            enter_normal_mode: KeyCode::Esc,
            execute_command: KeyCode::Enter,
        }
    }
}

// Singletons for keybindings while in different modes.
lazy_static! {
    pub static ref NORMAL_KEYBINDINGS: NormalModeKeyBindings = NormalModeKeyBindings::default();
    pub static ref COMMAND_KEYBINDINGS: CommandModeKeyBindings = CommandModeKeyBindings::default();
}
