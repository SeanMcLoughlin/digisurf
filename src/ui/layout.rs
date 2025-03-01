use ratatui::layout::{Constraint, Direction, Layout, Rect};

#[derive(Default)]
pub struct AppLayout {
    pub title: Rect,
    pub signal_list: Rect,
    pub waveform: Rect,
}

pub fn create_layout(area: Rect) -> AppLayout {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(area);

    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
        .split(main_chunks[1]);

    AppLayout {
        title: main_chunks[0],
        signal_list: content_chunks[0],
        waveform: content_chunks[1],
    }
}

// For dialogs or modal screens
#[allow(dead_code)]
pub fn create_centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
