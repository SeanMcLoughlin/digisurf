use crate::app::App;
use crate::input::keybindings::KeyBindings;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

pub fn draw_title(frame: &mut ratatui::Frame<'_>, area: Rect, app: &App) {
    let title = format!(
        "DigiSurf | Time: {} to {} of {}",
        app.time_offset,
        app.time_offset + app.window_size,
        app.max_time
    );

    let title_widget = Paragraph::new(title)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::White));

    frame.render_widget(title_widget, area);
}

pub fn draw_signal_list(frame: &mut ratatui::Frame<'_>, area: Rect, app: &App) {
    let items: Vec<ListItem> = app
        .signals
        .iter()
        .enumerate()
        .map(|(idx, name)| {
            let style = if idx == app.selected_signal {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };

            ListItem::new(name.as_str()).style(style)
        })
        .collect();

    let signals_list = List::new(items)
        .block(Block::default().title("Signals").borders(Borders::ALL))
        .highlight_style(Style::default().fg(Color::Yellow));

    frame.render_widget(signals_list, area);
}

pub fn draw_help_screen(frame: &mut ratatui::Frame<'_>, keybinds: &KeyBindings) {
    // Import the layout function
    use crate::ui::layout::create_centered_rect;

    let area = create_centered_rect(60, 50, frame.area());

    let help_text = format!(
        "DigiSurf Keyboard Controls\n\n\
        {} - Toggle help menu\n\
        {} - Quit application\n\
        {}/{} - Select signal\n\
        {}/{} - Navigate timeline\n\
        {} - Zoom in\n\
        {} - Zoom out",
        key_to_string(&keybinds.help),
        key_to_string(&keybinds.quit),
        key_to_string(&keybinds.up),
        key_to_string(&keybinds.down),
        key_to_string(&keybinds.left),
        key_to_string(&keybinds.right),
        key_to_string(&keybinds.zoom_in),
        key_to_string(&keybinds.zoom_out)
    );

    let help_widget = Paragraph::new(help_text)
        .block(Block::default().title("Help").borders(Borders::ALL))
        .style(Style::default());

    frame.render_widget(ratatui::widgets::Clear, area); // Clear the area first
    frame.render_widget(help_widget, area);
}

fn key_to_string(key: &crossterm::event::KeyCode) -> String {
    match key {
        crossterm::event::KeyCode::Char(c) => format!("'{}'", c),
        crossterm::event::KeyCode::F(n) => format!("F{}", n),
        _ => format!("{:?}", key),
    }
}
