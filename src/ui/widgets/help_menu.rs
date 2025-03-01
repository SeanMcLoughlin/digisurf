use crate::input::KEYBINDINGS;
use crossterm::event::KeyCode;
use ratatui::{
    style::Style,
    widgets::{Block, Borders, Paragraph, Widget},
};

#[derive(Default, Copy, Clone)]
pub struct HelpMenuWidget {}

impl HelpMenuWidget {
    fn key_to_string(self, key: &KeyCode) -> String {
        match key {
            KeyCode::Char(c) => format!("'{}'", c),
            KeyCode::F(n) => format!("F{}", n),
            _ => format!("{:?}", key),
        }
    }
}

impl Widget for HelpMenuWidget {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let help_text = format!(
            "DigiSurf Keyboard Controls\n\n\
            {} - Toggle help menu\n\
            {} - Quit application\n\
            {}/{} - Select signal\n\
            {}/{} - Navigate timeline\n\
            {} - Zoom in\n\
            {} - Zoom out\n\
            Left Click - Place yellow marker\n\
            Shift+Left Click - Place white marker\n\
            {} - Remove primary marker\n\
            {} - Remove secondary marker",
            self.key_to_string(&KEYBINDINGS.help),
            self.key_to_string(&KEYBINDINGS.quit),
            self.key_to_string(&KEYBINDINGS.up),
            self.key_to_string(&KEYBINDINGS.down),
            self.key_to_string(&KEYBINDINGS.left),
            self.key_to_string(&KEYBINDINGS.right),
            self.key_to_string(&KEYBINDINGS.zoom_in),
            self.key_to_string(&KEYBINDINGS.zoom_out),
            self.key_to_string(&KEYBINDINGS.delete_primary_marker),
            self.key_to_string(&KEYBINDINGS.delete_secondary_marker)
        );

        // Clear the entire screen first so only the help menu is visible
        ratatui::widgets::Clear.render(area, buf);

        // Then, render the help menu
        Paragraph::new(help_text)
            .block(Block::default().title("Help").borders(Borders::ALL))
            .style(Style::default())
            .render(area, buf);
    }
}
