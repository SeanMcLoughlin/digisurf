use crossterm::event::KeyCode;

pub fn enter_command_mode() -> KeyCode {
    KeyCode::Char(':')
}

pub fn up() -> KeyCode {
    KeyCode::Up
}

pub fn down() -> KeyCode {
    KeyCode::Down
}

pub fn left() -> KeyCode {
    KeyCode::Left
}

pub fn right() -> KeyCode {
    KeyCode::Right
}

pub fn zoom_in() -> KeyCode {
    KeyCode::Char('+')
}

pub fn zoom_out() -> KeyCode {
    KeyCode::Char('-')
}

pub fn zoom_full() -> KeyCode {
    KeyCode::Char('0')
}

pub fn delete_primary_marker() -> KeyCode {
    KeyCode::Delete
}

pub fn delete_secondary_marker() -> KeyCode {
    KeyCode::Backspace
}

pub fn enter_normal_mode() -> KeyCode {
    KeyCode::Esc
}

pub fn execute_command() -> KeyCode {
    KeyCode::Enter
}
