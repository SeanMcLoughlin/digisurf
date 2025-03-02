use crate::{
    command_mode::CommandModeWidget,
    commands, constants,
    input::{COMMAND_KEYBINDINGS, NORMAL_KEYBINDINGS},
    state::AppState,
    types::{AppMode, WaveValue},
    ui::{
        layout::{create_layout, AppLayout},
        widgets::{
            bottom_text_box::BottomTextBoxWidget, help_menu::HelpMenuWidget,
            signal_list::SignalListWidget, title_bar::TitleBarWidget, waveform::WaveformWidget,
        },
    },
};
use crossterm::event::{
    self, Event, KeyCode, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::{
    layout::Rect,
    prelude::*,
    widgets::{Paragraph, Widget},
    DefaultTerminal,
};
use std::{error::Error, time::Duration};
use vcd::Value;

pub struct App {
    pub state: AppState,
    pub layout: AppLayout,
    pub signal_list: SignalListWidget,
    pub help_menu: HelpMenuWidget,
    pub waveform: WaveformWidget,
    pub title_bar: TitleBarWidget,
    pub command_input: BottomTextBoxWidget,
    pub command_result: Option<(String, bool)>, // (message, is_error)
    pub command_result_time: Option<std::time::Instant>,
    pub command_mode: CommandModeWidget<AppState>,
}

impl Default for App {
    fn default() -> Self {
        let mut app = App {
            state: AppState::new(),
            layout: AppLayout::default(),
            signal_list: SignalListWidget::default(),
            help_menu: HelpMenuWidget::default(),
            waveform: WaveformWidget::default(),
            title_bar: TitleBarWidget::default(),
            command_input: BottomTextBoxWidget::default(),
            command_result: None,
            command_result_time: None,
            command_mode: CommandModeWidget::new(),
        };
        app.generate_test_data();
        app.register_commands();
        app
    }
}

impl App {
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<(), Box<dyn Error>> {
        let tick_rate = Duration::from_millis(250);
        while !self.state.exit {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;

            if event::poll(tick_rate)? {
                match event::read()? {
                    Event::Key(key) => {
                        if self.state.show_help {
                            match key.code {
                                KeyCode::Esc => {
                                    self.state.show_help = false;
                                    self.state.help_menu_scroll = 0;
                                }
                                KeyCode::Up => {
                                    if self.state.help_menu_scroll > 0 {
                                        self.state.help_menu_scroll -= 1;
                                    }
                                }
                                KeyCode::Down => {
                                    self.state.help_menu_scroll += 1;
                                }
                                _ => {}
                            }
                        } else if self.state.mode == AppMode::Command {
                            self.handle_command_input(key.code);
                        } else {
                            self.handle_input(key.code);
                        }
                    }
                    Event::Mouse(mouse) => self.handle_mouse(mouse),
                    _ => {}
                }
            } else {
                // Check if command result should be hidden
                if let Some(time) = self.command_result_time {
                    if time.elapsed().as_secs() >= constants::COMMAND_RESULT_HIDE_THRESHOLD_SECONDS
                    {
                        self.command_result = None;
                        self.command_result_time = None;
                    }
                }
            }
        }
        Ok(())
    }

    fn register_commands(&mut self) {
        commands::register_all_commands(&mut self.command_mode);
    }

    pub fn handle_command_input(&mut self, key: KeyCode) {
        match key {
            k if k == COMMAND_KEYBINDINGS.enter_normal_mode => {
                self.state.mode = AppMode::Normal;
                self.state.currently_typed_text_in_bottom_text_box.clear();
            }
            k if k == COMMAND_KEYBINDINGS.execute_command => {
                let command = self.state.currently_typed_text_in_bottom_text_box.clone();
                let result = self
                    .command_mode
                    .parser()
                    .execute(&command, &mut self.state);

                match result {
                    Ok(msg) => {
                        self.command_result = Some((msg, false));
                    }
                    Err(err) => {
                        self.command_result = Some((err, true));
                    }
                }
                self.command_result_time = Some(std::time::Instant::now());
                self.state.mode = AppMode::Normal;
                self.state.currently_typed_text_in_bottom_text_box.clear();
            }
            KeyCode::Backspace => {
                self.state.currently_typed_text_in_bottom_text_box.pop();
            }
            KeyCode::Char(c) => {
                self.state.currently_typed_text_in_bottom_text_box.push(c);
            }
            _ => {}
        }
    }

    pub fn handle_input(&mut self, key: KeyCode) {
        if self.state.mode == AppMode::Command {
            self.handle_command_input(key);
        } else {
            if key == NORMAL_KEYBINDINGS.enter_command_mode {
                self.state.mode = AppMode::Command;
                self.state.currently_typed_text_in_bottom_text_box.clear();
            } else {
                self.handle_normal_mode(key);
            }
        }
    }

    pub fn handle_mouse(&mut self, mouse: MouseEvent) {
        // Check if click is within waveform area
        if mouse.column >= self.layout.waveform.x
            && mouse.column <= self.layout.waveform.right()
            && mouse.row >= self.layout.waveform.y
            && mouse.row <= self.layout.waveform.bottom()
        {
            // Convert column to coordinates inside waveform area
            let column_in_waveform = mouse.column - self.layout.waveform.x;

            match mouse.kind {
                MouseEventKind::Down(MouseButton::Left) => {
                    if mouse.modifiers.contains(KeyModifiers::SHIFT) {
                        // Shift is still used for secondary marker
                        self.state
                            .set_secondary_marker(column_in_waveform, self.layout.waveform.width);
                    } else {
                        // Start potential drag or click - we don't know which yet
                        let time = self
                            .state
                            .screen_pos_to_time(column_in_waveform, self.layout.waveform.width);
                        self.state.drag_start = Some((column_in_waveform, time));
                        self.state.drag_current = Some((column_in_waveform, time));
                        self.state.is_dragging = false; // Not dragging yet
                    }
                }
                MouseEventKind::Drag(MouseButton::Left) => {
                    if self.state.drag_start.is_some() {
                        let time = self
                            .state
                            .screen_pos_to_time(column_in_waveform, self.layout.waveform.width);

                        if let Some((start_x, _)) = self.state.drag_start {
                            // Detect if we've moved enough to consider this a drag
                            if !self.state.is_dragging
                                && (start_x as i32 - column_in_waveform as i32).abs()
                                    >= constants::DRAG_DETECTED_THRESHOLD_PIXELS
                            {
                                self.state.is_dragging = true;
                            }
                        }

                        // Update current position regardless
                        self.state.drag_current = Some((column_in_waveform, time));
                    }
                }
                MouseEventKind::Up(MouseButton::Left) => {
                    if let (Some((start_x, start_time)), Some((end_x, end_time))) =
                        (self.state.drag_start, self.state.drag_current)
                    {
                        if self.state.is_dragging {
                            // This was a drag operation - zoom to selection
                            // Only zoom if dragged a minimum distance
                            if (start_x as i32 - end_x as i32).abs()
                                > constants::DRAG_STARTED_THRESHOLD_PIXELS
                            {
                                // Order the times correctly
                                let (min_time, max_time) = if start_time < end_time {
                                    (start_time, end_time)
                                } else {
                                    (end_time, start_time)
                                };

                                // Set the new zoom area
                                self.state.time_start = min_time;
                                self.state.time_range = max_time.saturating_sub(min_time).max(1);
                            }
                        } else {
                            // This was a click (not a drag) - set marker
                            self.state
                                .set_primary_marker(start_x, self.layout.waveform.width);
                        }
                    }

                    // Reset drag state
                    self.state.drag_start = None;
                    self.state.drag_current = None;
                    self.state.is_dragging = false;
                }
                _ => {}
            }
        } else {
            // Clear drag state if clicked outside the waveform area
            self.state.drag_start = None;
            self.state.drag_current = None;
            self.state.is_dragging = false;
        }
    }

    fn handle_normal_mode(&mut self, key: KeyCode) {
        match key {
            k if k == NORMAL_KEYBINDINGS.down => {
                self.state.selected_signal =
                    (self.state.selected_signal + 1) % self.state.signals.len();
            }
            k if k == NORMAL_KEYBINDINGS.up => {
                if self.state.selected_signal > 0 {
                    self.state.selected_signal -= 1;
                } else {
                    self.state.selected_signal = self.state.signals.len() - 1;
                }
            }
            k if k == NORMAL_KEYBINDINGS.left => {
                if self.state.time_start > 0 {
                    self.state.time_start = self
                        .state
                        .time_start
                        .saturating_sub(self.state.time_range / 4);
                }
            }
            k if k == NORMAL_KEYBINDINGS.right => {
                if self.state.time_start < self.state.max_time {
                    // Ensure the waveform view doesn't go beyond max_time
                    let max_start = self.state.max_time.saturating_sub(self.state.time_range);
                    self.state.time_start =
                        (self.state.time_start + self.state.time_range / 4).min(max_start);
                }
            }
            k if k == NORMAL_KEYBINDINGS.zoom_out => {
                // Calculate the new time range, doubling but capped at max_time
                let new_time_range = (self.state.time_range * 2).min(self.state.max_time);

                // Calculate center point of current view
                let center = self.state.time_start + (self.state.time_range / 2);

                // Calculate new start time, keeping the center point if possible
                let half_new_range = new_time_range / 2;
                let new_start = if center > half_new_range {
                    center.saturating_sub(half_new_range)
                } else {
                    0
                };

                // Make sure the end time (start + range) doesn't exceed max_time
                let adjusted_start = if new_start + new_time_range > self.state.max_time {
                    self.state.max_time.saturating_sub(new_time_range)
                } else {
                    new_start
                };

                self.state.time_start = adjusted_start;
                self.state.time_range = new_time_range;
            }
            k if k == NORMAL_KEYBINDINGS.zoom_in => {
                // Calculate center point of current view
                let center = self.state.time_start + (self.state.time_range / 2);

                // Calculate new time range, halving but with a minimum
                let new_time_range = (self.state.time_range / 2).max(10);

                // Calculate new start time, trying to keep the center point
                let half_new_range = new_time_range / 2;
                let new_start = center.saturating_sub(half_new_range);

                self.state.time_start = new_start;
                self.state.time_range = new_time_range;
            }
            k if k == NORMAL_KEYBINDINGS.zoom_full => {
                self.state.time_start = 0;
                self.state.time_range = self.state.max_time;
            }

            k if k == NORMAL_KEYBINDINGS.delete_primary_marker => {
                self.state.primary_marker = None;
            }
            k if k == NORMAL_KEYBINDINGS.delete_secondary_marker => {
                self.state.secondary_marker = None;
            }

            _ => {}
        }
    }

    pub fn generate_test_data(&mut self) {
        let test_signals = vec![
            "clk".to_string(),
            "reset".to_string(),
            "data_valid".to_string(),
            "data".to_string(),
            "tristate".to_string(),  // For Z states
            "undefined".to_string(), // For X states
        ];

        for signal in test_signals {
            self.state.signals.push(signal.clone());
            let mut values = Vec::new();

            for t in 0..100 {
                let value = match signal.as_str() {
                    "clk" => {
                        if t % 2 == 0 {
                            WaveValue::Binary(Value::V1)
                        } else {
                            WaveValue::Binary(Value::V0)
                        }
                    }
                    "reset" => {
                        if t < 10 {
                            WaveValue::Binary(Value::V1)
                        } else {
                            WaveValue::Binary(Value::V0)
                        }
                    }
                    "data_valid" => {
                        if t % 10 == 0 {
                            WaveValue::Binary(Value::V1)
                        } else {
                            WaveValue::Binary(Value::V0)
                        }
                    }
                    "data" => WaveValue::Bus(format!("{:02X}", t % 256)),
                    "tristate" => {
                        // Demonstrate high-impedance (Z) state every 3 cycles
                        if t % 3 == 0 {
                            WaveValue::Binary(Value::Z)
                        } else {
                            WaveValue::Binary(Value::V1)
                        }
                    }
                    "undefined" => {
                        // Demonstrate undefined (X) state every 5 cycles
                        if t % 5 == 0 {
                            WaveValue::Binary(Value::X)
                        } else {
                            WaveValue::Binary(Value::V0)
                        }
                    }
                    _ => WaveValue::Binary(Value::V0),
                };
                values.push((t as u64, value));
            }

            self.state.values.insert(signal, values);
        }
        self.state.max_time = 100;
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.layout = create_layout(area);

        if self.state.show_help {
            self.help_menu.render(area, buf, &mut self.state);
            return; // Don't render the rest of the UI when help is shown
        }

        // Regular UI rendering
        self.signal_list
            .render(self.layout.signal_list, buf, &mut self.state);
        self.waveform
            .render(self.layout.waveform, buf, &mut self.state);
        self.title_bar
            .render(self.layout.title, buf, &mut self.state);
        self.command_input
            .render(self.layout.command_bar, buf, &mut self.state);

        // Command result message if there is one
        if let Some((msg, is_error)) = &self.command_result {
            let status_style = if *is_error {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::Green)
            };

            // Create a temporary area just above the command bar for the result message
            let msg_area = Rect::new(
                self.layout.command_bar.x,
                self.layout.command_bar.y.saturating_sub(1),
                self.layout.command_bar.width.min(msg.len() as u16),
                1,
            );

            Paragraph::new(msg.clone())
                .style(status_style)
                .render(msg_area, buf);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::App;
    use insta::assert_snapshot;
    use ratatui::{backend::TestBackend, Terminal};

    #[test]
    fn test_render_app() {
        let mut app = App::default();
        let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
        terminal
            .draw(|frame| frame.render_widget(&mut app, frame.area()))
            .unwrap();
        assert_snapshot!(terminal.backend());
    }
}
