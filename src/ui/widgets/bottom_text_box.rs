use crate::{command_mode::CommandModeStateAccess, state::AppState, types::AppMode};
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
            format!(":{}", state.command_state().input_buffer)
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

        // In command mode, render cursor
        if state.mode == AppMode::Command {
            // Position cursor at cursor_position, not just at the end
            let cursor_x = inner_area.x + 1 + state.command_state().cursor_position as u16;
            let cursor_y = inner_area.y;

            if cursor_x < inner_area.right() && cursor_y < buf.area().height {
                let cell = &mut buf[(cursor_x, cursor_y)];
                cell.set_bg(cell.fg);
                cell.set_fg(Color::Black);
            }
        }
    }
}
