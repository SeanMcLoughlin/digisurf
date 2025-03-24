use crate::config;
use ratatui::layout::{Constraint, Direction, Layout, Rect};

#[derive(Default)]
pub struct AppLayout {
    pub marker_names: Rect,
    pub signal_list: Rect,
    pub time_ruler: Rect,
    pub waveform: Rect,
    pub command_bar: Rect,
}

pub fn create_layout(area: Rect, config: &config::AppConfig) -> AppLayout {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Min(1),    // Content area
                Constraint::Length(3), // Command bar
            ]
            .as_ref(),
        )
        .split(area);
    let remainder = main_chunks[0];
    let command_bar = main_chunks[1];

    let remainder = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Time ruler height
            Constraint::Length(1), // Marker names area
            Constraint::Min(5),    // Waveform area
        ])
        .split(remainder);
    let time_ruler = remainder[0];
    let marker_names = remainder[1];
    let remainder = remainder[2];

    let remainder = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(config.ui.signal_list_width),
                Constraint::Percentage(100 - config.ui.signal_list_width),
            ]
            .as_ref(),
        )
        .split(remainder);
    let signal_list = remainder[0];
    let waveform = remainder[1];

    // Leave a gap above signal list
    let time_ruler = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(config.ui.signal_list_width),
                Constraint::Percentage(100 - config.ui.signal_list_width),
            ]
            .as_ref(),
        )
        .split(time_ruler)[1];

    // Leave a gap above signal list
    let marker_names = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(config.ui.signal_list_width),
                Constraint::Percentage(100 - config.ui.signal_list_width),
            ]
            .as_ref(),
        )
        .split(marker_names)[1];

    AppLayout {
        marker_names,
        signal_list,
        time_ruler,
        waveform,
        command_bar,
    }
}
