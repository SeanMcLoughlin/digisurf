use crate::{
    command_mode::{state::CommandModeState, CommandModeStateAccess},
    config,
    fuzzy_finder::{state::FuzzyFinderState, FuzzyFinderStateAccess},
    parsers::types::{WaveValue, WaveformData},
    types::AppMode,
};
use std::collections::HashSet;

#[derive(Default)]
pub struct AppState {
    /// Flag indicating whether the application should exit.
    pub exit: bool,

    /// The mode the application is currently in.
    pub mode: AppMode,

    /// All data about the waveform parsed from the input file.
    pub waveform_data: WaveformData,

    // Currently highlighted signal
    pub selected_signal: usize,

    /// In the waveform view, the starting time step value visible at the current time. For example,
    /// if zoomed out all the way, this value will be 0. If zoomed in, this value will be the time
    /// step value of the leftmost visible time step.
    pub time_start: u64,

    /// The size of the waveform view in time step units. For example, if time_offset is 3 and this
    /// variable is 10, the waveform view will show time steps 3 through 13.
    pub time_range: u64,

    /// Primary marker position in time step units.
    pub primary_marker: Option<u64>,

    /// Secondary marker position in time step units.
    pub secondary_marker: Option<u64>,

    /// Is Some(Screen X coordinate, Time Step) if starting dragging for zoom selection
    pub drag_start: Option<(u16, u64)>,

    /// Is Some(Screen X coordinate, Time Step) if currently dragging for zoom selection
    pub drag_current: Option<(u16, u64)>,

    /// Flag used to differentiate between a drag operation and a potential click
    pub is_dragging: bool,

    /// State of command mode to render. This is accessed via methods in a trait implementation of
    /// CommandModeStateAccess, so it is not public.
    command_mode_state: CommandModeState,

    /// State of fuzzy finder to render. This is accessed via methods in a trait implementation of
    /// FuzzyFinderStateAccess, so it is not public.
    fuzzy_finder_state: FuzzyFinderState,

    /// Flag indicating that the help menu is currently being displayed
    pub show_help: bool,

    /// Current scroll position in the help menu
    pub help_menu_scroll: usize,

    /// Configuration state. Originally loaded from a file, but saved in app state so that the user
    /// can update configuration values while the application is running.
    pub config: config::AppConfig,

    /// List of signals that are currently being displayed
    pub displayed_signals: Vec<String>,

    /// Flag indicating if signals have been loaded but not yet filtered/selected
    pub signals_need_selection: bool,

    /// Set of signals that are currently selected in visual mode
    pub selected_signals: HashSet<usize>,

    /// Visual mode start position - used for determining selection range
    pub visual_start: Option<usize>,
}

// Access command mode state in the overall app state via a trait implementation
impl CommandModeStateAccess for AppState {
    fn command_state(&self) -> &CommandModeState {
        &self.command_mode_state
    }

    fn command_state_mut(&mut self) -> &mut CommandModeState {
        &mut self.command_mode_state
    }
}

impl FuzzyFinderStateAccess for AppState {
    fn fuzzy_finder_state(&self) -> &FuzzyFinderState {
        &self.fuzzy_finder_state
    }

    fn fuzzy_finder_state_mut(&mut self) -> &mut FuzzyFinderState {
        &mut self.fuzzy_finder_state
    }
}

impl AppState {
    pub fn new() -> Self {
        let mut app_state = AppState::default();
        app_state.time_range = 50;
        app_state.config =
            config::load_config(None).unwrap_or_else(|_| config::AppConfig::default());
        app_state
    }

    pub fn set_primary_marker(&mut self, x_pos: u16, window_width: u16) {
        self.primary_marker = Some(self.screen_pos_to_time(x_pos, window_width));
    }

    pub fn set_secondary_marker(&mut self, x_pos: u16, window_width: u16) {
        self.secondary_marker = Some(self.screen_pos_to_time(x_pos, window_width));
    }

    // Markers are saved with the time at which they're placed -- not the x coordinate at which
    // they're placed. This method converts the x coordinate to a time value.
    pub fn screen_pos_to_time(&self, x_pos: u16, window_width: u16) -> u64 {
        let time_range = self.time_range as f64;
        let position_ratio = x_pos as f64 / window_width as f64;
        let exact_time = self.time_start as f64 + (position_ratio * time_range);
        exact_time.round() as u64
    }

    pub fn get_value_at_marker(&self, signal: &str, marker_time: u64) -> Option<WaveValue> {
        if let Some(values) = self.waveform_data.values.get(signal) {
            // Find the value at or just before the marker time
            let mut last_value = None;

            for (time, value) in values {
                if *time > marker_time {
                    break;
                }
                last_value = Some(value.clone());
            }

            return last_value;
        }
        None
    }

    pub fn get_transition_at_marker(&self, signal: &str, marker_time: u64) -> Option<String> {
        if let Some(values) = self.waveform_data.values.get(signal) {
            for i in 0..values.len() {
                let (time, _) = values[i];

                if time == marker_time && i > 0 {
                    // We found our transition point
                    let (_, before_val) = &values[i - 1];
                    let (_, after_val) = &values[i];

                    // Only report if values are different (it's a real transition)
                    if !self.values_equal(before_val, after_val) {
                        return Some(self.format_transition(before_val, after_val));
                    }
                }
            }
        }
        None
    }

    // Helper function to check if two WaveValues are equal
    fn values_equal(&self, v1: &WaveValue, v2: &WaveValue) -> bool {
        match (v1, v2) {
            (WaveValue::Binary(b1), WaveValue::Binary(b2)) => b1 == b2,
            (WaveValue::Bus(s1), WaveValue::Bus(s2)) => s1 == s2,
            _ => false,
        }
    }

    // Helper function to format transition
    fn format_transition(&self, before: &WaveValue, after: &WaveValue) -> String {
        match (before, after) {
            (WaveValue::Binary(v1), WaveValue::Binary(v2)) => {
                format!("{:?}->{:?}", v1, v2)
            }
            (WaveValue::Bus(v1), WaveValue::Bus(v2)) => {
                format!("{}->{}", v1, v2)
            }
            _ => "???".to_string(),
        }
    }

    pub fn get_visible_values(&self, signal: &str) -> Vec<(u64, WaveValue)> {
        if !self.displayed_signals.contains(&signal.to_string()) {
            return Vec::new();
        }

        if let Some(values) = self.waveform_data.values.get(signal) {
            let mut result = Vec::new();

            // Find the last value before the visible range
            let mut last_before_view = None;
            for (t, v) in values {
                if *t < self.time_start {
                    last_before_view = Some((*t, v.clone()));
                } else {
                    break;
                }
            }

            // Add the last value before the view with an adjusted timestamp
            // to ensure it appears at the left edge of the view
            if let Some((_, v)) = last_before_view {
                result.push((self.time_start, v));
            }

            // Add all values within the view range
            for (t, v) in values {
                if *t >= self.time_start && *t < self.time_start + self.time_range {
                    result.push((*t, v.clone()));
                }
            }

            result
        } else {
            Vec::new()
        }
    }

    pub fn command_mode_state(&self) -> &CommandModeState {
        &self.command_mode_state
    }
}

#[cfg(test)]
mod tests {
    use crate::parsers::types::{Value, WaveValue};
    use crate::state::AppState;
    use std::collections::HashMap;

    fn create_test_state() -> AppState {
        let mut state = AppState::new();

        // Add test waveform data
        state.waveform_data.signals = vec!["sig1".to_string(), "sig2".to_string()];
        state.displayed_signals = vec!["sig1".to_string(), "sig2".to_string()];

        // Add signal values
        let mut values = HashMap::new();
        values.insert(
            "sig1".to_string(),
            vec![
                (0, WaveValue::Binary(Value::V0)),
                (10, WaveValue::Binary(Value::V1)),
                (20, WaveValue::Binary(Value::V0)),
            ],
        );
        values.insert(
            "sig2".to_string(),
            vec![
                (0, WaveValue::Binary(Value::V1)),
                (15, WaveValue::Binary(Value::V0)),
            ],
        );

        state.waveform_data.values = values;
        state.waveform_data.max_time = 50;

        state
    }

    #[test]
    fn test_screen_pos_to_time_conversion() {
        let mut state = create_test_state();
        state.time_start = 0;
        state.time_range = 100;

        // Test different positions
        assert_eq!(state.screen_pos_to_time(0, 100), 0); // left edge
        assert_eq!(state.screen_pos_to_time(50, 100), 50); // middle
        assert_eq!(state.screen_pos_to_time(100, 100), 100); // right edge

        // Test with offset
        state.time_start = 50;
        assert_eq!(state.screen_pos_to_time(0, 100), 50);
        assert_eq!(state.screen_pos_to_time(50, 100), 100);
    }

    #[test]
    fn test_get_value_at_marker() {
        let state = create_test_state();

        // Test value at different times
        assert_eq!(
            state.get_value_at_marker("sig1", 0),
            Some(WaveValue::Binary(Value::V0))
        );

        assert_eq!(
            state.get_value_at_marker("sig1", 15),
            Some(WaveValue::Binary(Value::V1))
        );

        // Test transition detection
        assert_eq!(
            state.get_transition_at_marker("sig1", 10),
            Some("V0->V1".to_string())
        );

        // Test invalid signal
        assert_eq!(state.get_value_at_marker("nonexistent", 0), None);
    }

    #[test]
    fn test_get_visible_values() {
        let mut state = create_test_state();

        // All values visible
        state.time_start = 0;
        state.time_range = 50;
        let visible = state.get_visible_values("sig1");
        assert_eq!(visible.len(), 3);

        // Still all...
        state.time_start = 10;
        state.time_range = 20;
        let visible = state.get_visible_values("sig1");
        assert_eq!(visible.len(), 3);

        // ...Now fewer
        state.time_start = 11;
        state.time_range = 20;
        let visible = state.get_visible_values("sig1");
        assert_eq!(visible.len(), 2);

        // Can only see the end state of the signal...
        state.time_start = 21;
        state.time_range = 50;
        let visible = state.get_visible_values("sig1");
        assert_eq!(visible.len(), 1);

        // ...even at a later time_start
        state.time_start = 30;
        state.time_range = 50;
        let visible = state.get_visible_values("sig1");
        assert_eq!(visible.len(), 1);

        // Test with non-displayed signal
        state.displayed_signals = vec!["sig2".to_string()];
        let visible = state.get_visible_values("sig1");
        assert_eq!(visible.len(), 0);
    }
}
