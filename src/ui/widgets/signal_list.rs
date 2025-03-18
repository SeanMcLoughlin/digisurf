use crate::{constants::WAVEFORM_HEIGHT, state::AppState};
use ratatui::{
    prelude::{Buffer, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, StatefulWidget, Widget},
};

#[derive(Default, Copy, Clone)]
pub struct SignalListWidget {}

impl SignalListWidget {}

impl StatefulWidget for SignalListWidget {
    type State = AppState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Draw the overall block
        let block = Block::default().title("Signals").borders(Borders::ALL);
        let inner_area = block.inner(area);
        block.render(area, buf);

        // Calculate how many signals we can display in the visible area
        let visible_signals = area.height as usize / WAVEFORM_HEIGHT;

        // Ensure scroll offset is within valid bounds
        if state.signal_scroll_offset
            > state
                .displayed_signals
                .len()
                .saturating_sub(visible_signals)
        {
            state.signal_scroll_offset = state
                .displayed_signals
                .len()
                .saturating_sub(visible_signals);
        }

        for (rel_idx, (idx, name)) in state
            .displayed_signals
            .iter()
            .enumerate()
            .skip(state.signal_scroll_offset)
            .take(visible_signals)
            .enumerate()
        {
            let y_position = inner_area.y + (rel_idx as u16 * WAVEFORM_HEIGHT as u16);

            // Skip if we're outside the visible area
            if y_position >= inner_area.bottom() {
                break;
            }

            let style = if idx == state.selected_signal {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };

            // Calculate vertical center of the waveform area
            let vertical_center = y_position + ((WAVEFORM_HEIGHT as u16 / 2) - 1);

            // Signal name
            let signal_area = Rect::new(
                inner_area.x,
                vertical_center,
                inner_area.width.min(name.len() as u16),
                1,
            );

            Paragraph::new(name.as_str())
                .style(style)
                .render(signal_area, buf);

            // Only show signal changes for primary marker
            if let Some(marker_time) = state.primary_marker {
                // Calculate position for value display
                let text_x = inner_area.x + name.len() as u16 + 1;
                let max_width = inner_area.width.saturating_sub(name.len() as u16 + 1);

                if max_width == 0 {
                    continue;
                }

                // Check for transition at marker
                if let Some(transition) = state.get_transition_at_marker(name, marker_time) {
                    let value_area = Rect::new(
                        text_x,
                        vertical_center,
                        max_width.min(transition.len() as u16),
                        1,
                    );

                    Paragraph::new(transition)
                        .style(Style::default().fg(Color::Cyan))
                        .render(value_area, buf);
                    continue;
                }

                // Show current value if no transition
                if let Some(value) = state.get_value_at_marker(name, marker_time) {
                    let value_text = value.to_string();

                    let value_area = Rect::new(
                        text_x,
                        vertical_center,
                        max_width.min(value_text.len() as u16),
                        1,
                    );

                    Paragraph::new(value_text)
                        .style(Style::default().fg(Color::Green))
                        .render(value_area, buf);
                }
            }
        }
    }
}
