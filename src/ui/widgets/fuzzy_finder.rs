use crate::{fuzzy_finder::FuzzyFinderStateAccess, state::AppState};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Clear, List, ListItem, StatefulWidget, Widget},
};

#[derive(Default, Copy, Clone)]
pub struct FuzzyFinderWidget {}

impl StatefulWidget for &mut FuzzyFinderWidget {
    type State = AppState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Clear the entire area first
        Clear.render(area, buf);

        // Create a centered popup area that's smaller than the full screen
        let width = area.width.min(100);
        let height = area.height.min(33);
        let x = area.x + (area.width.saturating_sub(width)) / 2;
        let y = area.y + (area.height.saturating_sub(height)) / 2;
        let popup_area = Rect::new(x, y, width, height);

        // Create a block for the popup
        let block = Block::default()
            .title("Signal Finder")
            .borders(Borders::ALL);

        let inner_area = block.inner(popup_area);
        block.render(popup_area, buf);

        // Render search query area
        let query_text = format!("> {}", state.fuzzy_finder_state().query);
        let query_span = Span::styled(query_text, Style::default().fg(Color::Yellow));
        ratatui::widgets::Paragraph::new(query_span).render(
            Rect::new(inner_area.x, inner_area.y, inner_area.width, 1),
            buf,
        );

        // Render info text
        let selected_count = state.fuzzy_finder_state().selected_signals.len();
        let info_text = format!(
            "Selected: {}/{}",
            selected_count,
            state.fuzzy_finder_state().all_signals.len()
        );
        let info_span = Span::styled(info_text, Style::default().fg(Color::Cyan));
        ratatui::widgets::Paragraph::new(info_span).render(
            Rect::new(inner_area.x, inner_area.y + 1, inner_area.width, 1),
            buf,
        );

        // Render the list of filtered signals
        let list_area = Rect::new(
            inner_area.x,
            inner_area.y + 2, // Below the query with a space
            inner_area.width,
            inner_area.height.saturating_sub(4), // Leave room for query, info and help text
        );

        let items: Vec<ListItem> = state
            .fuzzy_finder_state()
            .filtered_signals
            .iter()
            .map(|s| {
                let prefix = if state.fuzzy_finder_state().selected_signals.contains(s) {
                    "[✓] "
                } else {
                    "[ ] "
                };
                ListItem::new(format!("{}{}", prefix, s))
            })
            .collect();

        let list = List::new(items).highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

        // Get a mutable reference to the list state
        let mut list_state = state.fuzzy_finder_state_mut().list_state.clone();

        // Render using the cloned list state
        StatefulWidget::render(list, list_area, buf, &mut list_state);

        // Update the original list state with our changes
        state.fuzzy_finder_state_mut().list_state = list_state;

        // Render help text at bottom
        let help_text = "Up/Down: Navigate | Space: Toggle | A: Select All | C: Clear All | Enter: Done | Esc: Cancel";
        let help_span = Span::styled(help_text, Style::default().fg(Color::DarkGray));
        let help_area = Rect::new(
            inner_area.x,
            inner_area.bottom().saturating_sub(1),
            inner_area.width,
            1,
        );
        ratatui::widgets::Paragraph::new(help_span).render(help_area, buf);
    }
}
