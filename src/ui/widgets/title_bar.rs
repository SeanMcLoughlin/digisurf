use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, StatefulWidget, Widget},
};

use crate::app::AppState;

#[derive(Default, Copy, Clone)]
pub struct TitleBarWidget {}

impl StatefulWidget for TitleBarWidget {
    type State = AppState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if state.show_help {
            return;
        }
        let mut title = format!(
            "DigiSurf | Time: {} to {} of {}",
            state.time_start,
            state.time_start + state.time_range,
            state.max_time
        );

        // Combine base title with marker information
        let mut marker_info = String::new();

        if let Some(primary) = state.primary_marker {
            marker_info.push_str(&format!(" | M1: {}", primary));
        }

        if let Some(secondary) = state.secondary_marker {
            marker_info.push_str(&format!(" | M2: {}", secondary));
        }

        // Print delta between markers if both are present
        if let (Some(primary), Some(secondary)) = (state.primary_marker, state.secondary_marker) {
            let delta = if primary > secondary {
                primary - secondary
            } else {
                secondary - primary
            };
            marker_info.push_str(&format!(" | Δ: {}", delta));
        }
        title.push_str(&marker_info);

        Paragraph::new(title)
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::White))
            .render(area, buf);
    }
}
