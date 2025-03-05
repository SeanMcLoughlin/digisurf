use crate::{
    command_mode::{state::CommandModeState, CommandModeStateAccess},
    parsers::types::{WaveValue, WaveformData},
    types::AppMode,
};

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

    /// Flag indicating that the help menu is currently being displayed
    pub show_help: bool,

    /// Current scroll position in the help menu
    pub help_menu_scroll: usize,
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

impl AppState {
    pub fn new() -> Self {
        let mut app_state = AppState::default();
        app_state.time_range = 50;
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
        if let Some(values) = self.waveform_data.values.get(signal) {
            values
                .iter()
                .filter(|(t, _)| *t >= self.time_start && *t < self.time_start + self.time_range)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn command_mode_state(&self) -> &CommandModeState {
        &self.command_mode_state
    }
}
