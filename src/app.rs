use crate::{
    command_mode::{CommandModeStateAccess, CommandModeWidget},
    commands, config,
    constants::{self, WAVEFORM_HEIGHT},
    fuzzy_finder::FuzzyFinderStateAccess,
    parsers,
    state::AppState,
    types::AppMode,
    ui::{
        layout::{create_layout, AppLayout},
        widgets::{
            bottom_text_box::BottomTextBoxWidget, fuzzy_finder::FuzzyFinderWidget,
            help_menu::HelpMenuWidget, marker_names::MarkerNamesWidget,
            signal_list::SignalListWidget, time_ruler::TimeRulerWidget, waveform::WaveformWidget,
        },
    },
};
use crossterm::event::{
    self, Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
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
    pub marker_names: MarkerNamesWidget,
    pub time_ruler: TimeRulerWidget,
    pub command_input: BottomTextBoxWidget,
    pub command_mode: CommandModeWidget<AppState>,
    pub fuzzy_finder: FuzzyFinderWidget,
}

impl Default for App {
    fn default() -> Self {
        let config = config::AppConfig::default();
        let mut app = App {
            state: AppState::new(),
            layout: AppLayout::default(),
            signal_list: SignalListWidget::default(),
            help_menu: HelpMenuWidget::default(),
            waveform: WaveformWidget::default(),
            marker_names: MarkerNamesWidget::default(),
            time_ruler: TimeRulerWidget::default(),
            command_input: BottomTextBoxWidget::default(),
            command_mode: CommandModeWidget::new(),
            fuzzy_finder: FuzzyFinderWidget::default(),
        };
        app.state.config = config;
        app.register_commands();
        app
    }
}

impl App {
    pub fn with_config(config: config::AppConfig) -> Self {
        let mut app = Self::default();
        app.state.config = config;
        app
    }

    pub fn load_config(&mut self, path_override: Option<String>) -> Result<(), String> {
        match config::load_config(path_override) {
            Ok(config) => {
                self.state.config = config;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

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
                            self.handle_command_input(key);
                        } else if self.state.mode == AppMode::FuzzyFinder {
                            self.handle_fuzzy_finder_input(key);
                        } else {
                            self.handle_input(key);
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

    fn handle_fuzzy_finder_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                // Exit fuzzy finder mode without changing selections
                self.state.mode = AppMode::Normal;
            }
            KeyCode::Enter => {
                // Get selected signals
                let selected_signals = self.state.fuzzy_finder_state().get_selected_signals();

                // Maintain original signal order from waveform_data
                let mut displayed_signals = Vec::new();
                for signal in &self.state.waveform_data.signals {
                    if selected_signals.contains(signal) {
                        displayed_signals.push(signal.clone());
                    }
                }

                // Set the displayed signals in the original order
                self.state.displayed_signals = displayed_signals;

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
            KeyCode::Char(c) if key.modifiers.contains(KeyModifiers::CONTROL) => {
                match c {
                    'a' | 'A' => {
                        // Select all filtered signals
                        self.state.fuzzy_finder_state_mut().select_all();
                    }
                    'x' | 'X' => {
                        // Clear all selections
                        self.state.fuzzy_finder_state_mut().clear_selection();
                    }
                    _ => {}
                }
            }
            KeyCode::Up => self.state.fuzzy_finder_state_mut().select_previous(),
            KeyCode::Down => self.state.fuzzy_finder_state_mut().select_next(),
            KeyCode::Backspace => self.state.fuzzy_finder_state_mut().handle_backspace(),
            KeyCode::Char(c) => self.state.fuzzy_finder_state_mut().handle_input(c),
            _ => {}
        }
    }

    pub fn handle_command_input(&mut self, key: KeyEvent) {
        match key {
            k if k.code == self.state.config.keybindings.enter_normal_mode => {
                self.state.mode = AppMode::Normal;
                self.state.command_state_mut().clear();
            }
            k if k.code == self.state.config.keybindings.execute_command => {
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

    pub fn handle_input(&mut self, key: KeyEvent) {
        if self.state.mode == AppMode::Command {
            self.handle_command_input(key);
        } else {
            if key.code == self.state.config.keybindings.enter_command_mode {
                self.state.mode = AppMode::Command;
                self.state.command_state_mut().clear();
            } else {
                self.handle_normal_mode(key);
            }
        }
    }

    pub fn handle_mouse(&mut self, mouse: MouseEvent) {
        if self.mouse_within_rect(&mouse, &self.layout.waveform) {
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
        } else if self.mouse_within_rect(&mouse, &self.layout.signal_list) {
            // Handle clicking on signal rows in the signal list area
            if mouse.kind == MouseEventKind::Down(MouseButton::Left) {
                // Calculate which signal was clicked based on mouse position
                // Each row in signal list has WAVEFORM_HEIGHT height
                let row_in_signal_list = mouse.row as usize - self.layout.signal_list.y as usize;
                let clicked_signal_index =
                    row_in_signal_list / WAVEFORM_HEIGHT + self.state.signal_scroll_offset;

                // Check if it's a valid signal index
                if clicked_signal_index < self.state.displayed_signals.len() {
                    self.state.selected_signal = clicked_signal_index;
                    self.adjust_scroll_if_needed();
                }
            }
        } else {
            // Clear drag state if clicked outside the waveform area
            self.state.drag_start = None;
            self.state.drag_current = None;
            self.state.is_dragging = false;
        }
    }

    fn handle_normal_mode(&mut self, key: KeyEvent) {
        match key {
            k if k.code == self.state.config.keybindings.up => {
                if k.modifiers.contains(KeyModifiers::SHIFT) {
                    // Move the selected signal up in the list
                    if !self.state.displayed_signals.is_empty() && self.state.selected_signal > 0 {
                        // Swap the signal with the one above it
                        self.state
                            .displayed_signals
                            .swap(self.state.selected_signal, self.state.selected_signal - 1);
                        // Keep the selection on the moved signal
                        self.state.selected_signal -= 1;
                        self.adjust_scroll_if_needed();
                    }
                } else {
                    if !self.state.displayed_signals.is_empty() {
                        if self.state.selected_signal > 0 {
                            self.state.selected_signal -= 1;
                            self.adjust_scroll_if_needed();
                        }
                    }
                }
            }
            k if k.code == self.state.config.keybindings.down => {
                if k.modifiers.contains(KeyModifiers::SHIFT) {
                    // Move the selected signal down in the list
                    if !self.state.displayed_signals.is_empty()
                        && self.state.selected_signal < self.state.displayed_signals.len() - 1
                    {
                        // Swap the signal with the one below it
                        self.state
                            .displayed_signals
                            .swap(self.state.selected_signal, self.state.selected_signal + 1);
                        // Keep the selection on the moved signal
                        self.state.selected_signal += 1;
                        self.adjust_scroll_if_needed();
                    }
                } else {
                    if !self.state.displayed_signals.is_empty() {
                        if self.state.selected_signal < self.state.displayed_signals.len() - 1 {
                            self.state.selected_signal += 1;
                            self.adjust_scroll_if_needed();
                        }
                    }
                }
            }
            k if k.code == self.state.config.keybindings.down => {}
            k if k.code == self.state.config.keybindings.left => {
                if self.state.time_start > 0 {
                    self.state.time_start = self
                        .state
                        .time_start
                        .saturating_sub(self.state.time_range / 4);
                }
            }
            k if k.code == self.state.config.keybindings.right => {
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
            k if k.code == self.state.config.keybindings.zoom_out => {
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
            k if k.code == self.state.config.keybindings.zoom_in => {
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
            k if k.code == self.state.config.keybindings.zoom_full => {
                self.state.time_start = 0;
                self.state.time_range = self.state.waveform_data.max_time;
            }

            k if k.code == self.state.config.keybindings.delete_primary_marker => {
                self.state.primary_marker = None;
            }
            k if k.code == self.state.config.keybindings.delete_secondary_marker => {
                self.state.secondary_marker = None;
            }

            _ => {}
        }
    }

    fn mouse_within_rect(&self, mouse: &MouseEvent, rect: &Rect) -> bool {
        mouse.column >= rect.x
            && mouse.column <= rect.right()
            && mouse.row >= rect.y
            && mouse.row <= rect.bottom()
    }

    fn adjust_scroll_if_needed(&mut self) {
        // Calculate how many signals can be displayed at once
        let visible_signals = self.layout.waveform.height as usize / WAVEFORM_HEIGHT;

        if visible_signals == 0 {
            return; // Avoid division by zero
        }

        // If the selected signal is before the current scroll offset, scroll up
        if self.state.selected_signal < self.state.signal_scroll_offset {
            self.state.signal_scroll_offset = self.state.selected_signal;
        }

        // If the selected signal is beyond the visible area, scroll down
        let last_visible = self.state.signal_scroll_offset + visible_signals - 1;
        if self.state.selected_signal > last_visible {
            // Set scroll to position the selected signal at the bottom of the view
            self.state.signal_scroll_offset = self
                .state
                .selected_signal
                .saturating_sub(visible_signals - 1);
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
        self.layout = create_layout(area, &self.state.config);

        if self.state.show_help {
            self.help_menu.render(area, buf, &mut self.state);
            return; // Don't render the rest of the UI when help is shown
        }

        if self.state.mode == AppMode::FuzzyFinder {
            self.fuzzy_finder.render(area, buf, &mut self.state);
            return; // Don't render the rest of the UI when fuzzy finder is shown
        }

        self.signal_list
            .render(self.layout.signal_list, buf, &mut self.state);
        self.waveform
            .render(self.layout.waveform, buf, &mut self.state);
        self.marker_names
            .render(self.layout.marker_names, buf, &mut self.state);
        self.time_ruler
            .render(self.layout.time_ruler, buf, &mut self.state);
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
        command_mode::CommandModeStateAccess,
        config,
        constants::WAVEFORM_HEIGHT,
        fuzzy_finder::FuzzyFinderStateAccess,
        parsers::types::{Value, WaveValue},
        types::AppMode,
    };
    use crossterm::event::{
        KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
    };
    use insta::assert_snapshot;
    use ratatui::{backend::TestBackend, Terminal};

    fn setup_arrow_key_test_app(time_start: u64, time_range: u64) -> App {
        use crate::parsers::types::{Value, WaveValue};
        let mut app = App::with_config(config::AppConfig::default());

        // Set up waveform data
        app.state.waveform_data.max_time = 1000;
        app.state.time_start = time_start;
        app.state.time_range = time_range;

        // Add test signals
        app.state.waveform_data.signals = vec![
            "clock".to_string(),
            "data".to_string(),
            "enable".to_string(),
        ];

        // Initialize displayed_signals with the same signals
        app.state.displayed_signals = app.state.waveform_data.signals.clone();

        // Clock signal with consistent pattern (50 time unit cycles)
        let mut clock_values = Vec::new();
        // Only add the time zero value if time_start isn't already zero
        if time_start > 0 {
            clock_values.push((0, WaveValue::Binary(Value::V0)));
        }
        clock_values.extend(vec![
            (time_start, WaveValue::Binary(Value::V0)),
            (time_start + 50, WaveValue::Binary(Value::V1)),
            (time_start + 100, WaveValue::Binary(Value::V0)),
            (time_start + 150, WaveValue::Binary(Value::V1)),
            (time_start + 200, WaveValue::Binary(Value::V0)),
        ]);
        app.state
            .waveform_data
            .values
            .insert("clock".to_string(), clock_values);

        // Data signal with transitions
        let mut data_values = Vec::new();
        // Only add the time zero value if time_start isn't already zero
        if time_start > 0 {
            data_values.push((0, WaveValue::Binary(Value::V0)));
        }
        data_values.extend(vec![
            (time_start, WaveValue::Binary(Value::V0)),
            (time_start + 20, WaveValue::Binary(Value::V0)),
            (time_start + 120, WaveValue::Binary(Value::V1)),
            (time_start + 180, WaveValue::Binary(Value::V0)),
        ]);
        app.state
            .waveform_data
            .values
            .insert("data".to_string(), data_values);

        // Enable signal with one transition
        let mut enable_values = Vec::new();
        // Only add the time zero value if time_start isn't already zero
        if time_start > 0 {
            enable_values.push((0, WaveValue::Binary(Value::V0)));
        }
        enable_values.extend(vec![
            (time_start, WaveValue::Binary(Value::V1)),
            (time_start + 150, WaveValue::Binary(Value::V0)),
        ]);
        app.state
            .waveform_data
            .values
            .insert("enable".to_string(), enable_values);

        app
    }

    #[test]
    fn test_render_empty_app() {
        let mut app = App::with_config(config::AppConfig::default());
        let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
        terminal
            .draw(|frame| frame.render_widget(&mut app, frame.area()))
            .unwrap();
        assert_snapshot!(terminal.backend());
    }

    #[test]
    fn test_help_menu_no_scroll() {
        let mut app = App::with_config(config::AppConfig::default());
        let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();

        app.state.mode = AppMode::Command;
        app.state.show_help = true;

        terminal
            .draw(|frame| frame.render_widget(&mut app, frame.area()))
            .unwrap();
        assert_snapshot!(terminal.backend());
    }

    #[test]
    fn test_help_menu_with_scroll() {
        let mut app = App::with_config(config::AppConfig::default());
        let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();

        app.state.mode = AppMode::Command;
        app.state.show_help = true;
        app.state.help_menu_scroll = 10;

        terminal
            .draw(|frame| frame.render_widget(&mut app, frame.area()))
            .unwrap();

        assert_snapshot!(terminal.backend());
    }

    #[test]
    fn test_render_app_with_test_data() {
        use crate::parsers::types::{Value, WaveValue};
        let mut app = App::with_config(config::AppConfig::default());

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
    fn test_render_app_with_bus_values() {
        use crate::parsers::types::{Value, WaveValue};
        let mut app = App::with_config(config::AppConfig::default());

        // Add test data with binary and bus signals
        app.state.waveform_data.signals = vec![
            "binary_signal".to_string(),
            "narrow_bus".to_string(),
            "wide_bus".to_string(),
            "mixed_bus".to_string(),
        ];
        app.state.displayed_signals = app.state.waveform_data.signals.clone();
        app.state.waveform_data.max_time = 200;

        // Add a binary signal
        app.state.waveform_data.values.insert(
            "binary_signal".to_string(),
            vec![
                (0, WaveValue::Binary(Value::V0)),
                (100, WaveValue::Binary(Value::V1)),
            ],
        );

        // Add a narrow bus (4-bit)
        app.state.waveform_data.values.insert(
            "narrow_bus".to_string(),
            vec![
                (0, WaveValue::Bus("0A".to_string())),
                (80, WaveValue::Bus("0F".to_string())),
            ],
        );

        // Add a wide bus (16-bit)
        app.state.waveform_data.values.insert(
            "wide_bus".to_string(),
            vec![
                (0, WaveValue::Bus("DEAD".to_string())),
                (50, WaveValue::Bus("BEEF".to_string())),
                (150, WaveValue::Bus("CAFE".to_string())),
            ],
        );

        // Add a bus with x/z values
        app.state.waveform_data.values.insert(
            "mixed_bus".to_string(),
            vec![
                (0, WaveValue::Bus("00".to_string())),
                (60, WaveValue::Bus("xZ".to_string())),
                (120, WaveValue::Bus("FF".to_string())),
            ],
        );

        // Set up the view parameters
        app.state.time_start = 0;
        app.state.time_range = 200;

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
        let mut app = App::with_config(config::AppConfig::default());

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
        let mut app = App::with_config(config::AppConfig::default());

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
        let mut app = App::with_config(config::AppConfig::default());

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
    fn test_render_app_in_fuzzy_finder_mode() {
        let mut app = App::with_config(config::AppConfig::default());

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

    #[test]
    fn test_fuzzy_finder_maintains_signal_order() {
        let config = config::AppConfig::default();
        let mut app = App::with_config(config);

        // Setup test data with signals in specific order
        app.state.waveform_data.signals = vec![
            "first_signal".to_string(),
            "middle_signal".to_string(),
            "last_signal".to_string(),
        ];

        // Add some test values
        app.state.waveform_data.values.insert(
            "first_signal".to_string(),
            vec![(0, WaveValue::Binary(Value::V0))],
        );
        app.state.waveform_data.values.insert(
            "middle_signal".to_string(),
            vec![(0, WaveValue::Binary(Value::V1))],
        );
        app.state.waveform_data.values.insert(
            "last_signal".to_string(),
            vec![(0, WaveValue::Binary(Value::V0))],
        );

        // Enter fuzzy finder mode
        app.state.mode = AppMode::FuzzyFinder;
        let signals = app.state.waveform_data.signals.clone();
        app.state.fuzzy_finder_state_mut().set_signals(signals, &[]);

        // Select signals in reverse order
        app.state
            .fuzzy_finder_state_mut()
            .list_state
            .select(Some(2)); // last_signal
        app.state.fuzzy_finder_state_mut().toggle_selected_signal();

        app.state
            .fuzzy_finder_state_mut()
            .list_state
            .select(Some(0)); // first_signal
        app.state.fuzzy_finder_state_mut().toggle_selected_signal();

        // Accept selection
        app.handle_fuzzy_finder_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()));

        // Verify signals are displayed in the original order
        assert_eq!(app.state.displayed_signals.len(), 2);
        assert_eq!(app.state.displayed_signals[0], "first_signal");
        assert_eq!(app.state.displayed_signals[1], "last_signal");

        // Verify app is now in normal mode
        assert_eq!(app.state.mode, AppMode::Normal);
    }

    #[test]
    fn test_arrow_keys_with_empty_signal_list_do_not_panic() {
        let mut app = App::with_config(config::AppConfig::default());

        app.state.waveform_data.max_time = 1000;
        app.state.displayed_signals = vec![];
        app.state.selected_signal = 0;

        app.handle_input(KeyEvent::new(KeyCode::Up, KeyModifiers::empty()));
        app.handle_input(KeyEvent::new(KeyCode::Down, KeyModifiers::empty()));
        app.handle_input(KeyEvent::new(KeyCode::Left, KeyModifiers::empty()));
        app.handle_input(KeyEvent::new(KeyCode::Right, KeyModifiers::empty()));
    }

    #[test]
    fn test_arrow_keys_up_down_signal_selection() {
        let mut app = App::with_config(config::AppConfig::default());

        app.state.displayed_signals = vec![
            "signal_1".to_string(),
            "signal_2".to_string(),
            "signal_3".to_string(),
        ];

        // Start at the first signal
        app.state.selected_signal = 0;

        app.handle_input(KeyEvent::new(KeyCode::Down, KeyModifiers::empty()));
        assert_eq!(app.state.selected_signal, 1);

        app.handle_input(KeyEvent::new(KeyCode::Down, KeyModifiers::empty()));
        assert_eq!(app.state.selected_signal, 2);

        // Signal should not wrap
        app.handle_input(KeyEvent::new(KeyCode::Down, KeyModifiers::empty()));
        assert_eq!(app.state.selected_signal, 2);

        app.handle_input(KeyEvent::new(KeyCode::Up, KeyModifiers::empty()));
        assert_eq!(app.state.selected_signal, 1);

        app.handle_input(KeyEvent::new(KeyCode::Up, KeyModifiers::empty()));
        assert_eq!(app.state.selected_signal, 0);

        // Signal should not wrap
        app.handle_input(KeyEvent::new(KeyCode::Up, KeyModifiers::empty()));
        assert_eq!(app.state.selected_signal, 0);
    }

    #[test]
    fn test_arrow_keys_left_once() {
        let mut app = setup_arrow_key_test_app(400, 200);
        app.handle_input(KeyEvent::new(KeyCode::Left, KeyModifiers::empty()));

        let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
        terminal
            .draw(|frame| frame.render_widget(&mut app, frame.area()))
            .unwrap();

        assert_snapshot!(terminal.backend());
    }

    #[test]
    fn test_arrow_keys_left_twice() {
        let mut app = setup_arrow_key_test_app(400, 200);

        for _ in 0..2 {
            app.handle_input(KeyEvent::new(KeyCode::Left, KeyModifiers::empty()));
        }

        let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
        terminal
            .draw(|frame| frame.render_widget(&mut app, frame.area()))
            .unwrap();

        assert_snapshot!(terminal.backend());
    }

    #[test]
    fn test_arrow_keys_right_once() {
        let mut app = setup_arrow_key_test_app(400, 200);
        app.handle_input(KeyEvent::new(KeyCode::Right, KeyModifiers::empty()));

        let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
        terminal
            .draw(|frame| frame.render_widget(&mut app, frame.area()))
            .unwrap();

        assert_snapshot!(terminal.backend());
    }

    #[test]
    fn test_arrow_keys_right_twice() {
        let mut app = setup_arrow_key_test_app(400, 200);

        for _ in 0..2 {
            app.handle_input(KeyEvent::new(KeyCode::Right, KeyModifiers::empty()));
        }

        let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
        terminal
            .draw(|frame| frame.render_widget(&mut app, frame.area()))
            .unwrap();

        assert_snapshot!(terminal.backend());
    }

    #[test]
    fn test_arrow_keys_right_twice_then_left_once() {
        let mut app = setup_arrow_key_test_app(400, 200);

        for _ in 0..2 {
            app.handle_input(KeyEvent::new(KeyCode::Right, KeyModifiers::empty()));
        }
        app.handle_input(KeyEvent::new(KeyCode::Left, KeyModifiers::empty()));

        let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
        terminal
            .draw(|frame| frame.render_widget(&mut app, frame.area()))
            .unwrap();

        assert_snapshot!(terminal.backend());
    }

    #[test]
    fn test_arrow_keys_left_at_time_start_does_not_move() {
        let mut app = setup_arrow_key_test_app(0, 200);
        app.handle_input(KeyEvent::new(KeyCode::Left, KeyModifiers::empty()));

        let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
        terminal
            .draw(|frame| frame.render_widget(&mut app, frame.area()))
            .unwrap();

        assert_snapshot!(terminal.backend());
    }

    #[test]
    fn test_arrow_keys_right_at_time_end_does_not_move() {
        let mut app = setup_arrow_key_test_app(800, 200);
        app.handle_input(KeyEvent::new(KeyCode::Right, KeyModifiers::empty()));

        let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
        terminal
            .draw(|frame| frame.render_widget(&mut app, frame.area()))
            .unwrap();

        assert_snapshot!(terminal.backend());
    }

    // Create an app with a large number of signals so that scrolling is required to view all of
    // them.
    fn setup_up_down_scroll_test_app() -> App {
        use crate::parsers::types::{Value, WaveValue};
        let mut app = App::with_config(config::AppConfig::default());

        // Set up waveform data
        app.state.waveform_data.max_time = 1000;
        app.state.time_start = 0;
        app.state.time_range = 400;
        app.state.selected_signal = 0;
        app.state.signal_scroll_offset = 0;

        // Add more signals than can fit in the viewport
        for c in 'a'..='z' {
            let signal_name = format!("signal_{}", c);
            app.state.waveform_data.signals.push(signal_name.clone());

            // Create values for each signal with the same pattern
            let mut signal_values = Vec::new();
            signal_values.extend(vec![
                (0, WaveValue::Binary(Value::V0)),
                (50, WaveValue::Binary(Value::V1)),
                (100, WaveValue::Binary(Value::V0)),
                (150, WaveValue::Binary(Value::V1)),
                (200, WaveValue::Binary(Value::V0)),
            ]);

            app.state
                .waveform_data
                .values
                .insert(signal_name, signal_values);
        }

        // Initialize displayed_signals with the same signals
        app.state.displayed_signals = app.state.waveform_data.signals.clone();

        app
    }

    #[test]
    fn test_scroll_down() {
        let mut app = setup_up_down_scroll_test_app();

        // We need to render the app once to initialize the layout
        // so that adjust_scroll_if_needed has the correct dimensions
        let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
        terminal
            .draw(|frame| frame.render_widget(&mut app, frame.area()))
            .unwrap();

        // Now move down far enough to trigger scrolling
        for _ in 0..10 {
            app.handle_input(KeyEvent::new(KeyCode::Down, KeyModifiers::empty()));
        }

        // Render again to capture the scrolled state
        terminal
            .draw(|frame| frame.render_widget(&mut app, frame.area()))
            .unwrap();

        // The snapshot assert should capture signals further in the alphabet than a-f
        assert_snapshot!(terminal.backend());
    }

    #[test]
    fn test_scroll_up() {
        let mut app = setup_up_down_scroll_test_app();

        // We need to render the app once to initialize the layout
        // so that adjust_scroll_if_needed has the correct dimensions
        let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
        terminal
            .draw(|frame| frame.render_widget(&mut app, frame.area()))
            .unwrap();

        // Now move down far enough to trigger scrolling
        for _ in 0..10 {
            app.handle_input(KeyEvent::new(KeyCode::Down, KeyModifiers::empty()));
        }

        // Scroll back up
        for _ in 0..10 {
            app.handle_input(KeyEvent::new(KeyCode::Up, KeyModifiers::empty()));
        }

        // Render again to capture the scrolled state
        terminal
            .draw(|frame| frame.render_widget(&mut app, frame.area()))
            .unwrap();

        // The snapshot assert should capture signals a-f
        assert_snapshot!(terminal.backend());
    }

    #[test]
    fn test_mouse_signal_selection() {
        let mut app = App::with_config(config::AppConfig::default());
        for i in 0..5 {
            let signal_name = format!("signal_{}", i);
            app.state.waveform_data.signals.push(signal_name.clone());
            let mut signal_values = Vec::new();
            signal_values.extend(vec![
                (0, WaveValue::Binary(Value::V0)),
                (50, WaveValue::Binary(Value::V1)),
                (100, WaveValue::Binary(Value::V0)),
            ]);

            app.state
                .waveform_data
                .values
                .insert(signal_name, signal_values);
        }
        app.state.displayed_signals = app.state.waveform_data.signals.clone();

        // We need to render the app once to initialize the layout
        let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
        terminal
            .draw(|frame| frame.render_widget(&mut app, frame.area()))
            .unwrap();

        // Create a mouse event for clicking on multiple different signals.
        // Calculate position (each signal takes WAVEFORM_HEIGHT rows)
        for signal_index in vec![2, 1, 3, 0] {
            // +1 to click in middle of signal
            let y_position = app.layout.signal_list.y + (signal_index * WAVEFORM_HEIGHT) as u16 + 1;

            let mouse_event = MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column: app.layout.signal_list.x + 2, // "Near" the beginning of the signal name
                row: y_position,
                modifiers: KeyModifiers::empty(),
            };

            // Handle the mouse event
            app.handle_mouse(mouse_event);

            // Verify the selection changed
            assert_eq!(app.state.selected_signal, signal_index);
        }
    }
}
