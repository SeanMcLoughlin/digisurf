use crate::config;
use ratatui::layout::{Constraint, Direction, Layout, Rect};

#[derive(Default)]
pub struct AppLayout {
    pub title: Rect,
    pub signal_list: Rect,
    pub waveform: Rect,
    pub command_bar: Rect,
}

pub fn create_layout(area: Rect) -> AppLayout {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3), // Title Bar
                Constraint::Min(1),    // Content area
                Constraint::Length(3), // Command bar
            ]
            .as_ref(),
        )
        .split(area);

    // Use config value for signal list width
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(config::read_config().ui.signal_list_width),
                Constraint::Percentage(100 - config::read_config().ui.signal_list_width),
            ]
            .as_ref(),
        )
        .split(main_chunks[1]);

    AppLayout {
        title: main_chunks[0],
        signal_list: content_chunks[0],
        waveform: content_chunks[1],
        command_bar: main_chunks[2],
    }
}
