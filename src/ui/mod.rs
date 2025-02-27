pub mod layout;
mod waveform;
mod widgets;

use crate::app::App;
use layout::*;
use waveform::*;
use widgets::*;

// Main draw function that coordinates all UI components
pub fn draw(app: &App, frame: &mut ratatui::Frame<'_>) {
    // If help is showing, just display that and return
    if app.show_help {
        draw_help_screen(frame, &crate::input::KEYBINDINGS);
        return;
    }

    // Otherwise draw the main UI
    let layout = create_layout(frame.area());

    // Draw the title bar
    draw_title(frame, layout.title, app);

    // Draw the signals list
    draw_signal_list(frame, layout.signals, app);

    // Draw waveforms
    draw_waveforms(frame, layout.waveforms, app);
}

// For handling additional UI states like file dialogs, error messages, etc.
#[allow(dead_code)]
pub fn draw_error_popup(frame: &mut ratatui::Frame<'_>, message: &str) {
    let area = create_centered_rect(60, 20, frame.area());
    let error_widget = ratatui::widgets::Paragraph::new(message)
        .block(
            ratatui::widgets::Block::default()
                .title("Error")
                .borders(ratatui::widgets::Borders::ALL),
        )
        .style(ratatui::style::Style::default().fg(ratatui::style::Color::Red));

    frame.render_widget(ratatui::widgets::Clear, area);
    frame.render_widget(error_widget, area);
}
