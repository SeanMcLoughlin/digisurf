use crate::app::App;
use crate::input::keybindings::KeyBindings;
use crate::model::types::WaveValue;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn draw_title(frame: &mut ratatui::Frame<'_>, area: Rect, app: &App) {
    // Create the base title with time information
    let mut title = format!(
        "DigiSurf | Time: {} to {} of {}",
        app.waveform.time_start,
        app.waveform.time_start + app.waveform.time_range,
        app.max_time
    );

    // Combine base title with marker information
    title.push_str(&get_marker_info(app));

    let title_widget = Paragraph::new(title)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::White));

    frame.render_widget(title_widget, area);
}

fn get_marker_info(app: &App) -> String {
    let mut marker_info = String::new();

    if let Some(primary) = app.waveform.primary_marker {
        marker_info.push_str(&format!(" | M1: {}", primary));
    }

    if let Some(secondary) = app.waveform.secondary_marker {
        marker_info.push_str(&format!(" | M2: {}", secondary));
    }

    // Print delta between markers if both are present
    if let (Some(primary), Some(secondary)) =
        (app.waveform.primary_marker, app.waveform.secondary_marker)
    {
        let delta = if primary > secondary {
            primary - secondary
        } else {
            secondary - primary
        };
        marker_info.push_str(&format!(" | Δ: {}", delta));
    }
    marker_info
}

pub fn draw_signal_list(frame: &mut ratatui::Frame<'_>, area: Rect, app: &App) {
    // Draw the overall block
    let block = Block::default().title("Signals").borders(Borders::ALL);
    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let waveform_height = 2; // Match the value in draw_waveforms

    // Now manually render each signal name at the proper position
    for (idx, name) in app.signals.iter().enumerate() {
        let y_position = inner_area.y + (idx as u16 * waveform_height);

        // Skip if we're outside the visible area
        if y_position >= inner_area.bottom() {
            break;
        }

        let style = if idx == app.selected_signal {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        // Calculate vertical center of the waveform area
        let vertical_center = y_position + ((waveform_height / 2) - 1);

        // Signal name
        let signal_area = Rect::new(
            inner_area.x,
            vertical_center,
            inner_area.width.min(name.len() as u16),
            1,
        );
        frame.render_widget(Paragraph::new(name.as_str()).style(style), signal_area);

        // Only show signal changes for primary marker
        if let Some(marker_time) = app.waveform.primary_marker {
            // Calculate position for value display
            let text_x = inner_area.x + name.len() as u16 + 1;
            let max_width = inner_area.width.saturating_sub(name.len() as u16 + 1);

            if max_width == 0 {
                continue;
            }

            // Check for transition at marker
            if let Some(transition) = app.get_transition_at_marker(name, marker_time) {
                let value_area = Rect::new(
                    text_x,
                    vertical_center,
                    max_width.min(transition.len() as u16),
                    1,
                );

                frame.render_widget(
                    Paragraph::new(transition).style(Style::default().fg(Color::Cyan)),
                    value_area,
                );
                continue;
            }

            // Show current value if no transition
            if let Some(value) = app.get_value_at_marker(name, marker_time) {
                let value_text = match value {
                    WaveValue::Binary(v) => format!("{:?}", v),
                    WaveValue::Bus(s) => s,
                };

                let value_area = Rect::new(
                    text_x,
                    vertical_center,
                    max_width.min(value_text.len() as u16),
                    1,
                );

                frame.render_widget(
                    Paragraph::new(value_text).style(Style::default().fg(Color::Green)),
                    value_area,
                );
            }
        }
    }
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
        {} - Zoom out\n\
        Left Click - Place yellow marker\n\
        Shift+Left Click - Place white marker\n\
        {} - Remove primary marker\n\
        {} - Remove secondary marker",
        key_to_string(&keybinds.help),
        key_to_string(&keybinds.quit),
        key_to_string(&keybinds.up),
        key_to_string(&keybinds.down),
        key_to_string(&keybinds.left),
        key_to_string(&keybinds.right),
        key_to_string(&keybinds.zoom_in),
        key_to_string(&keybinds.zoom_out),
        key_to_string(&keybinds.delete_primary_marker),
        key_to_string(&keybinds.delete_secondary_marker)
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
