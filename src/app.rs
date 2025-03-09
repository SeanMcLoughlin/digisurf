use crate::{
    command_mode::{CommandModeStateAccess, CommandModeWidget},
    commands, config, constants,
    fuzzy_finder::FuzzyFinderStateAccess,
    parsers,
    state::AppState,
    types::AppMode,
    ui::{
        layout::{create_layout, AppLayout},
        widgets::{
            bottom_text_box::BottomTextBoxWidget, fuzzy_finder::FuzzyFinderWidget,
            help_menu::HelpMenuWidget, signal_list::SignalListWidget, title_bar::TitleBarWidget,
            waveform::WaveformWidget,
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
use std::io;
use std::{error::Error, path::Path, time::Duration};

pub struct App {
    pub state: AppState,
    pub layout: AppLayout,
    pub signal_list: SignalListWidget,
    pub help_menu: HelpMenuWidget,
    pub waveform: WaveformWidget,
    pub title_bar: TitleBarWidget,
    pub command_input: BottomTextBoxWidget,
    pub command_mode: CommandModeWidget<AppState>,
    pub fuzzy_finder: FuzzyFinderWidget,
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
            command_mode: CommandModeWidget::new(),
            fuzzy_finder: FuzzyFinderWidget::default(),
        };
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
                        } else if self.state.mode == AppMode::FuzzyFinder {
                            self.handle_fuzzy_finder_input(key);
                        } else {
                            self.handle_input(key.code);
                        }
                    }
                    Event::Mouse(mouse) => self.handle_mouse(mouse),
                    _ => {}
                }
            } else {
                // Check if command result should be hidden
                if let Some(time) = self.state.command_state().command_result_time {
                    if time.elapsed().as_secs() >= constants::COMMAND_RESULT_HIDE_THRESHOLD_SECONDS
                    {
                        self.state.command_state_mut().result_message = None;
                        self.state.command_state_mut().command_result_time = None;
                    }
                }
            }
        }
        Ok(())
    }

    fn register_commands(&mut self) {
        commands::register_all_commands(&mut self.command_mode);
    }

    fn handle_fuzzy_finder_input(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                // Exit fuzzy finder mode without changing selections
                self.state.mode = AppMode::Normal;
            }
            KeyCode::Enter => {
                // Accept changed selections of signals
                self.state.displayed_signals =
                    self.state.fuzzy_finder_state().get_selected_signals();

                // Ensure selected signal is within bounds
                if self.state.selected_signal >= self.state.displayed_signals.len() {
                    self.state.selected_signal = 0;
                }
                self.state.mode = AppMode::Normal;
            }
            KeyCode::Char(' ') => {
                // Toggle selection of current signal
                self.state.fuzzy_finder_state_mut().toggle_selected_signal();
            }
            KeyCode::Char('a') | KeyCode::Char('A') => {
                // Select all filtered signals
                self.state.fuzzy_finder_state_mut().select_all();
            }
            KeyCode::Char('c') | KeyCode::Char('C') => {
                // Clear all selections
                self.state.fuzzy_finder_state_mut().clear_selection();
            }
            KeyCode::Up => self.state.fuzzy_finder_state_mut().select_previous(),
            KeyCode::Down => self.state.fuzzy_finder_state_mut().select_next(),
            KeyCode::Backspace => self.state.fuzzy_finder_state_mut().handle_backspace(),
            KeyCode::Char(c) => self.state.fuzzy_finder_state_mut().handle_input(c),
            _ => {}
        }
    }

    pub fn handle_command_input(&mut self, key: KeyCode) {
        match key {
            k if k == config::read_config().keybindings.enter_normal_mode => {
                self.state.mode = AppMode::Normal;
                self.state.command_state_mut().clear();
            }
            k if k == config::read_config().keybindings.execute_command => {
                let executed = self.command_mode.execute(&mut self.state);
                if executed {
                    self.state.command_state_mut().command_result_time =
                        Some(std::time::Instant::now());
                    // If the command execution didn't switch modes, the app is still in Command
                    // mode. So return to normal mode as the command has finished executing.
                    if self.state.mode == AppMode::Command {
                        self.state.mode = AppMode::Normal;
                    }
                }
            }
            // Let command mode handle all other keys
            _ => self.command_mode.handle_input(key, &mut self.state),
        }
    }

    pub fn handle_input(&mut self, key: KeyCode) {
        if self.state.mode == AppMode::Command {
            self.handle_command_input(key);
        } else {
            if key == config::read_config().keybindings.enter_command_mode {
                self.state.mode = AppMode::Command;
                self.state.command_state_mut().clear();
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
            k if k == config::read_config().keybindings.down => {
                if !self.state.displayed_signals.is_empty() {
                    self.state.selected_signal =
                        (self.state.selected_signal + 1) % self.state.displayed_signals.len();
                }
            }
            k if k == config::read_config().keybindings.up => {
                if !self.state.displayed_signals.is_empty() {
                    if self.state.selected_signal > 0 {
                        self.state.selected_signal -= 1;
                    } else {
                        self.state.selected_signal = self.state.displayed_signals.len() - 1;
                    }
                }
            }
            k if k == config::read_config().keybindings.left => {
                if self.state.time_start > 0 {
                    self.state.time_start = self
                        .state
                        .time_start
                        .saturating_sub(self.state.time_range / 4);
                }
            }
            k if k == config::read_config().keybindings.right => {
                if self.state.time_start < self.state.waveform_data.max_time {
                    // Ensure the waveform view doesn't go beyond max_time
                    let max_start = self
                        .state
                        .waveform_data
                        .max_time
                        .saturating_sub(self.state.time_range);
                    self.state.time_start =
                        (self.state.time_start + self.state.time_range / 4).min(max_start);
                }
            }
            k if k == config::read_config().keybindings.zoom_out => {
                // Calculate the new time range, doubling but capped at max_time
                let new_time_range =
                    (self.state.time_range * 2).min(self.state.waveform_data.max_time);

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
                let adjusted_start =
                    if new_start + new_time_range > self.state.waveform_data.max_time {
                        self.state
                            .waveform_data
                            .max_time
                            .saturating_sub(new_time_range)
                    } else {
                        new_start
                    };

                self.state.time_start = adjusted_start;
                self.state.time_range = new_time_range;
            }
            k if k == config::read_config().keybindings.zoom_in => {
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
            k if k == config::read_config().keybindings.zoom_full => {
                self.state.time_start = 0;
                self.state.time_range = self.state.waveform_data.max_time;
            }

            k if k == config::read_config().keybindings.delete_primary_marker => {
                self.state.primary_marker = None;
            }
            k if k == config::read_config().keybindings.delete_secondary_marker => {
                self.state.secondary_marker = None;
            }

            _ => {}
        }
    }

    pub fn load_vcd_file<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        // Clear existing data
        self.state.waveform_data.signals.clear();
        self.state.waveform_data.values.clear();
        self.state.displayed_signals.clear();

        // Parse the VCD file
        let waveform_data = parsers::vcd::parse_vcd_file(path)?;

        // Update the state with the parsed data
        let signals_clone = waveform_data.signals.clone();
        self.state.waveform_data.signals = signals_clone.clone();
        self.state
            .fuzzy_finder_state_mut()
            .set_signals(signals_clone, &[]);
        self.state.waveform_data.values = waveform_data.values;
        self.state.waveform_data.max_time = waveform_data.max_time;

        // Reset the view to show the full waveform
        self.state.time_start = 0;
        self.state.time_range = waveform_data.max_time;
        self.state.selected_signal = 0;

        Ok(())
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.layout = create_layout(area);

        if self.state.show_help {
            self.help_menu.render(area, buf, &mut self.state);
            return; // Don't render the rest of the UI when help is shown
        }

        if self.state.mode == AppMode::FuzzyFinder {
            self.fuzzy_finder.render(area, buf, &mut self.state);
            return; // Don't render the rest of the UI when fuzzy finder is shown
        }

        // Regular UI rendering - only show signals that are in displayed_signals
        self.signal_list
            .render(self.layout.signal_list, buf, &mut self.state);
        self.waveform
            .render(self.layout.waveform, buf, &mut self.state);
        self.title_bar
            .render(self.layout.title, buf, &mut self.state);
        self.command_input
            .render(self.layout.command_bar, buf, &mut self.state);

        // Command result message if there is one
        if let Some(ref result_message) = self.state.command_state().result_message {
            let status_style = if self.state.command_state().result_is_error {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::Green)
            };

            // Create a temporary area just above the command bar for the result message
            let msg_area = Rect::new(
                self.layout.command_bar.x,
                self.layout.command_bar.y.saturating_sub(1),
                self.layout
                    .command_bar
                    .width
                    .min(result_message.len() as u16),
                1,
            );

            Paragraph::new(result_message.clone())
                .style(status_style)
                .render(msg_area, buf);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::App;
    use crate::{
        command_mode::CommandModeStateAccess, config, fuzzy_finder::FuzzyFinderStateAccess,
        types::AppMode,
    };
    use insta::assert_snapshot;
    use ratatui::{backend::TestBackend, Terminal};

    #[test]
    fn test_render_empty_app() {
        config::load_default_config();
        let mut app = App::default();
        let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
        terminal
            .draw(|frame| frame.render_widget(&mut app, frame.area()))
            .unwrap();
        assert_snapshot!(terminal.backend());
    }

    #[test]
    fn test_render_app_with_test_data() {
        use crate::parsers::types::{Value, WaveValue};
        config::load_default_config();
        let mut app = App::default();

        // Add some test data to the app state
        app.state
            .waveform_data
            .signals
            .push("test_signal_1".to_string());
        app.state
            .waveform_data
            .signals
            .push("test_signal_2".to_string());
        app.state.waveform_data.max_time = 1000;

        // Initialize displayed_signals with the same signals
        app.state
            .displayed_signals
            .push("test_signal_1".to_string());
        app.state
            .displayed_signals
            .push("test_signal_2".to_string());

        // Add some test values
        app.state.waveform_data.values.insert(
            "test_signal_1".to_string(),
            vec![
                (0, WaveValue::Binary(Value::V0)),
                (500, WaveValue::Binary(Value::V1)),
            ],
        );
        app.state.waveform_data.values.insert(
            "test_signal_2".to_string(),
            vec![(0, WaveValue::Binary(Value::V1))],
        );

        // Set up the view parameters
        app.state.time_start = 0;
        app.state.time_range = 1000;
        app.state.selected_signal = 0;

        // Render the app
        let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
        terminal
            .draw(|frame| frame.render_widget(&mut app, frame.area()))
            .unwrap();

        assert_snapshot!(terminal.backend());
    }

    #[test]
    fn test_render_app_with_markers() {
        use crate::parsers::types::{Value, WaveValue};
        config::load_default_config();
        let mut app = App::default();

        // Add some test data
        app.state.waveform_data.signals.push("signal".to_string());
        app.state.displayed_signals.push("signal".to_string());
        app.state.waveform_data.max_time = 100;
        app.state.waveform_data.values.insert(
            "signal".to_string(),
            vec![
                (0, WaveValue::Binary(Value::V0)),
                (50, WaveValue::Binary(Value::V1)),
            ],
        );

        // Set markers
        app.state.primary_marker = Some(25);
        app.state.secondary_marker = Some(75);

        // Render the app
        let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
        terminal
            .draw(|frame| frame.render_widget(&mut app, frame.area()))
            .unwrap();

        assert_snapshot!(terminal.backend());
    }

    #[test]
    fn test_render_app_in_command_mode() {
        config::load_default_config();
        let mut app = App::default();

        // Enter command mode
        app.state.mode = AppMode::Command;
        app.state.command_state_mut().input_buffer = "goto 50".to_string();
        app.state.command_state_mut().cursor_position = 7; // Cursor after "goto 50"

        // Render the app
        let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
        terminal
            .draw(|frame| frame.render_widget(&mut app, frame.area()))
            .unwrap();

        assert_snapshot!(terminal.backend());
    }

    #[test]
    fn test_render_app_with_command_result() {
        config::load_default_config();
        let mut app = App::default();

        // Set a command result message
        app.state.command_state_mut().result_message =
            Some("Command executed successfully".to_string());
        app.state.command_state_mut().result_is_error = false;
        app.state.command_state_mut().command_result_time = Some(std::time::Instant::now());

        // Render the app
        let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
        terminal
            .draw(|frame| frame.render_widget(&mut app, frame.area()))
            .unwrap();

        assert_snapshot!(terminal.backend());
    }

    #[test]
    fn test_render_app_with_help_menu() {
        config::load_default_config();
        let mut app = App::default();

        // Show help menu
        app.state.show_help = true;
        app.state.help_menu_scroll = 5; // Scrolled down a bit

        // Render the app
        let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
        terminal
            .draw(|frame| frame.render_widget(&mut app, frame.area()))
            .unwrap();

        assert_snapshot!(terminal.backend());
    }

    #[test]
    fn test_render_app_in_fuzzy_finder_mode() {
        config::load_default_config();
        let mut app = App::default();

        // Setup signals
        let signals = vec![
            "signal_1".to_string(),
            "signal_2".to_string(),
            "test_signal".to_string(),
            "data_bus".to_string(),
        ];

        // Enter fuzzy finder mode
        app.state.mode = AppMode::FuzzyFinder;
        app.state.fuzzy_finder_state_mut().set_signals(signals, &[]);

        // Add a query and handle input character by character
        for c in "sig".chars() {
            app.state.fuzzy_finder_state_mut().handle_input(c);
        }

        // Select the first signal
        app.state.fuzzy_finder_state_mut().toggle_selected_signal();

        // Render the app
        let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
        terminal
            .draw(|frame| frame.render_widget(&mut app, frame.area()))
            .unwrap();

        assert_snapshot!(terminal.backend());
    }
}
