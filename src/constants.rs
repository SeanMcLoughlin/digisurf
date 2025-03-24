/// The threshold in pixels after which the drag operation is considered to be detected. This does
/// not mean that a drag operation will execute!
pub const DRAG_DETECTED_THRESHOLD_PIXELS: i32 = 3;

/// The threshold in pixels after which the drag operation is considered to have started. Must be
/// greater than or equal to DRAG_DETECTED_THRESHOLD_PIXELS.
pub const DRAG_STARTED_THRESHOLD_PIXELS: i32 = 5;

/// The duration in seconds after which the toast of a command result will be hidden.
pub const COMMAND_RESULT_HIDE_THRESHOLD_SECONDS: u64 = 3;

/// The highlight color when clicking and dragging on the waveform.
pub const DRAG_COLOR: ratatui::style::Color = ratatui::style::Color::Rgb(100, 150, 255);

/// The height of a single wave line in terminal rows.
pub const WAVEFORM_HEIGHT: usize = 2;

/// The color of the primary marker.
pub const PRIMARY_MARKER_COLOR: ratatui::style::Color = ratatui::style::Color::Yellow;

/// The color of the secondary marker.
pub const SECONDARY_MARKER_COLOR: ratatui::style::Color = ratatui::style::Color::White;

/// The color of the default saved marker.
pub const DEFAULT_SAVED_MARKER_COLOR: ratatui::style::Color = ratatui::style::Color::Cyan;
