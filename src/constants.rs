use ratatui::style::Color;

pub const DRAG_DETECTED_THRESHOLD_PIXELS: i32 = 3;
pub const DRAG_STARTED_THRESHOLD_PIXELS: i32 = 5;
pub const COMMAND_RESULT_HIDE_THRESHOLD_SECONDS: u64 = 3;
pub const DRAG_COLOR: Color = Color::Rgb(100, 150, 255);
