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
    pub help_text: &'static str,
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
            help_text: "
                    Controls:
                    h - Toggle help menu
                    q - Quit
                    Up/Down - Select signal
                    Left/Right - Navigate timeline
                    +/- - Zoom in/out
                    ",
        }
    }
}
