use crate::state::AppState;
use crossterm::event::KeyCode;
use ratatui::{
    prelude::{Buffer, Rect},
    style::Style,
    widgets::{Block, Borders, Clear, Paragraph, StatefulWidget, Widget},
};

#[derive(Default, Clone, Copy, Eq, PartialEq)]
pub struct HelpMenuWidget {}

impl HelpMenuWidget {
    fn key_to_string(&self, key: &KeyCode) -> String {
        match key {
            KeyCode::Char(c) => format!("'{}'", c),
            KeyCode::F(n) => format!("F{}", n),
            _ => format!("{:?}", key),
        }
    }
}

impl StatefulWidget for HelpMenuWidget {
    type State = AppState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Clear the entire screen first so only the help menu is visible
        Clear.render(area, buf);

        let help_text = format!(
            "DigiSurf Keyboard Controls\n\
            \n\
            Navigation:\n\
            {}/{} - Select signal\n\
            {}+Shift/{}+Shift - Move signal up/down\n\
            {}/{} - Navigate timeline\n\
            {} - Zoom in\n\
            {} - Zoom out\n\
            {} - Zoom full\n\
            {} - Enter command mode\n\
            \n\
            Markers:\n\
            Left Click - Place yellow marker (primary)\n\
            Shift+Left Click - Place white marker (secondary)\n\
            {} - Remove primary marker\n\
            {} - Remove secondary marker\n\
            \n\
            Selection:\n\
            Click and Drag - Zoom to selection\n\
            \n\
            Commands:\n\
            :zoom <factor> - Zoom to 1/factor of total\n\
            :zoomfull (:zf) - Zoom to full view\n\
            :goto <time> - Go to specific time\n\
            :marker <1|2> <time> - Set marker\n\
            :q - Quit digisurf\n\
            :help (:h) - Show this help\n\
            \n\
            Help Navigation:\n\
            Up/Down arrows - Scroll help content\n\
            Esc - Close help",
            self.key_to_string(&state.config.keybindings.up),
            self.key_to_string(&state.config.keybindings.down),
            self.key_to_string(&state.config.keybindings.up),
            self.key_to_string(&state.config.keybindings.down),
            self.key_to_string(&state.config.keybindings.left),
            self.key_to_string(&state.config.keybindings.right),
            self.key_to_string(&state.config.keybindings.zoom_in),
            self.key_to_string(&state.config.keybindings.zoom_out),
            self.key_to_string(&state.config.keybindings.zoom_full),
            self.key_to_string(&state.config.keybindings.enter_command_mode),
            self.key_to_string(&state.config.keybindings.delete_primary_marker),
            self.key_to_string(&state.config.keybindings.delete_secondary_marker)
        );

        // Calculate a centered rectangle for the help menu
        let help_width = area.width.min(70);
        let help_height = area.height.min(20);
        let help_x = area.x + (area.width.saturating_sub(help_width)) / 2;
        let help_y = area.y + (area.height.saturating_sub(help_height)) / 2;

        let help_area = Rect::new(help_x, help_y, help_width, help_height);

        // Create the block with borders
        let block = Block::default()
            .title("Help [Scroll with Up/Down]")
            .borders(Borders::ALL);

        // Calculate inner area for text content
        let inner_area = block.inner(help_area);

        // Split the text into lines
        let lines: Vec<&str> = help_text.split('\n').collect();

        // Calculate max scroll value (ensure we can't scroll past content)
        let max_scroll = lines.len().saturating_sub(inner_area.height as usize);
        let scroll = state.help_menu_scroll.min(max_scroll);
        if state.help_menu_scroll > max_scroll {
            state.help_menu_scroll = max_scroll;
        }

        // Get visible lines based on scroll position
        let visible_lines = lines
            .iter()
            .skip(scroll)
            .take(inner_area.height as usize)
            .cloned()
            .collect::<Vec<&str>>()
            .join("\n");

        // Render the block
        block.render(help_area, buf);

        // Render the text content
        Paragraph::new(visible_lines)
            .style(Style::default())
            .render(inner_area, buf);
    }
}
