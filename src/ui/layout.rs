use crate::config;
use ratatui::layout::{Constraint, Direction, Layout, Rect};

#[derive(Default)]
pub struct AppLayout {
    pub title: Rect,
    pub command_display: Rect,
    pub marker_names: Rect,
    pub signal_list: Rect,
    pub waveform: Rect,
    pub command_bar: Rect,
}

pub fn create_layout(area: Rect, config: &config::AppConfig) -> AppLayout {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3), // Title Bar
                Constraint::Length(2), // Marker names area
                Constraint::Min(1),    // Content area
                Constraint::Length(3), // Command bar
            ]
            .as_ref(),
        )
        .split(area);

    // Split the marker names area horizontally to match signal_list and waveform widths
    let marker_area_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(config.ui.signal_list_width),
                Constraint::Percentage(100 - config.ui.signal_list_width),
            ]
            .as_ref(),
        )
        .split(main_chunks[1]);

    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(config.ui.signal_list_width),
                Constraint::Percentage(100 - config.ui.signal_list_width),
            ]
            .as_ref(),
        )
        .split(main_chunks[2]);

    AppLayout {
        title: main_chunks[0],
        command_display: marker_area_chunks[0],
        marker_names: marker_area_chunks[1],
        signal_list: content_chunks[0],
        waveform: content_chunks[1],
        command_bar: main_chunks[3],
    }
}
