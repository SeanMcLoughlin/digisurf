use crate::{state::AppState, types::AppMode};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, StatefulWidget, Widget},
};

#[derive(Default, Copy, Clone)]
pub struct BottomTextBoxWidget {}

impl StatefulWidget for BottomTextBoxWidget {
    type State = AppState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let block = Block::default()
            .title(format!("{:?}", state.mode))
            .borders(Borders::TOP);
        let inner_area = block.inner(area);
        block.render(area, buf);

        // Command text to display
        let command_text = if state.mode == AppMode::Command {
            format!(":{}", state.currently_typed_text_in_bottom_text_box)
        } else {
            " ':' for command mode. :q, then <Enter> to quit. :h then <Enter> for help.".to_string()
        };

        let style = if state.mode == AppMode::Command {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        Paragraph::new(command_text)
            .style(style)
            .render(inner_area, buf);

        // In command mode, also render a cursor at the end of the text
        if state.mode == AppMode::Command {
            let cursor_x =
                inner_area.x + 1 + state.currently_typed_text_in_bottom_text_box.len() as u16; // +1 for the ':'
            let cursor_y = inner_area.y;

            // Make sure cursor is within bounds and valid for the buffer
            if cursor_x < inner_area.right() && cursor_y < buf.area().height {
                // Get the character at cursor position
                let cell = &mut buf[(cursor_x, cursor_y)];
                // Highlight the character by inverting colors
                cell.set_bg(cell.fg);
                cell.set_fg(Color::Black);
            }
        }
    }
}
