use crate::model::types::WaveValue;
use std::{collections::HashMap, error::Error, io::BufReader};
use vcd::{Command, Parser, Value};

pub struct WaveformState {
    // In the waveform view, the starting time step value visible at the current time. For example,
    // if zoomed out all the way, this value will be 0. If zoomed in, this value will be the time
    // step value of the leftmost visible time step.
    pub time_start: u64,

    // The size of the waveform view in time step units. For example, if time_offset is 3 and this
    // variable is 10, the waveform view will show time steps 3 through 13.
    pub time_range: u64,

    // Primary marker position in time step units.
    pub primary_marker: Option<u64>,

    // Secondary marker position in time step units.
    pub secondary_marker: Option<u64>,
}

impl WaveformState {
    pub fn new() -> Self {
        Self {
            time_start: 0,
            time_range: 50,
            primary_marker: None,
            secondary_marker: None,
        }
    }

    pub fn set_primary_marker(&mut self, x_pos: u16, window_width: u16) {
        self.primary_marker = Some(self.screen_pos_to_time(x_pos, window_width));
    }

    pub fn set_secondary_marker(&mut self, x_pos: u16, window_width: u16) {
        self.secondary_marker = Some(self.screen_pos_to_time(x_pos, window_width));
    }

    // Markers are saved with the time at which they're placed -- not the x coordinate at which
    // they're placed. This method converts the x coordinate to a time value.
    fn screen_pos_to_time(&self, x_pos: u16, window_width: u16) -> u64 {
        let time_range = self.time_range as f64;
        let position_ratio = x_pos as f64 / window_width as f64;
        let exact_time = self.time_start as f64 + (position_ratio * time_range);
        exact_time.round() as u64
    }
}

pub struct App {
    pub signals: Vec<String>,
    pub values: HashMap<String, Vec<(u64, WaveValue)>>,
    pub selected_signal: usize,
    pub max_time: u64,
    pub show_help: bool,
    pub waveform: WaveformState,
}

impl App {
    pub fn new() -> App {
        let mut app = App {
            signals: Vec::new(),
            values: HashMap::new(),
            selected_signal: 0,
            max_time: 0,
            show_help: false,
            waveform: WaveformState::new(),
        };
        app.generate_test_data();
        app
    }

    pub fn set_primary_marker(&mut self, x_pos: u16, window_width: u16) {
        self.waveform.set_primary_marker(x_pos, window_width);
    }

    pub fn set_secondary_marker(&mut self, x_pos: u16, window_width: u16) {
        self.waveform.set_secondary_marker(x_pos, window_width);
    }

    pub fn get_value_at_marker(&self, signal: &str, marker_time: u64) -> Option<WaveValue> {
        if let Some(values) = self.values.get(signal) {
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
        if let Some(values) = self.values.get(signal) {
            for i in 0..values.len() {
                let (time, _) = values[i];

                if time == marker_time && i > 0 {
                    // We found our transition point
                    let (_, before_val) = &values[i - 1];
                    let (_, after_val) = &values[i];

                    // Only report if values are different (it's a real transition)
                    if !values_equal(before_val, after_val) {
                        return Some(format_transition(before_val, after_val));
                    }
                }
            }
        }
        None
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
            self.signals.push(signal.clone());
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

            self.values.insert(signal, values);
        }
        self.max_time = 100;
    }

    #[allow(dead_code)]
    pub fn load_vcd(&mut self, filename: &str) -> Result<(), Box<dyn Error>> {
        let file = std::fs::File::open(filename)?;
        let reader = BufReader::new(file);
        let mut parser = Parser::new(reader);

        let header = parser.parse_header()?;

        let mut current_time = 0u64;
        let mut id_to_signal = HashMap::new();
        let mut id_to_size = HashMap::new();

        for item in header.items {
            if let vcd::ScopeItem::Var(var) = item {
                let signal_name = var.reference.clone();
                self.signals.push(signal_name.clone());
                self.values.insert(signal_name.clone(), Vec::new());
                id_to_signal.insert(var.code.clone(), var.reference.clone());
                id_to_size.insert(var.code.clone(), var.size);
            }
        }

        while let Some(command) = parser.next() {
            match command? {
                Command::Timestamp(time) => {
                    current_time = time;
                    self.max_time = self.max_time.max(time);
                }
                Command::ChangeScalar(id, value) => {
                    if let Some(signal_name) = id_to_signal.get(&id) {
                        if let Some(values) = self.values.get_mut(signal_name) {
                            values.push((current_time, WaveValue::Binary(value)));
                        }
                    }
                }
                Command::ChangeVector(id, value) => {
                    if let Some(signal_name) = id_to_signal.get(&id) {
                        if let Some(size) = id_to_size.get(&id) {
                            if let Some(values) = self.values.get_mut(signal_name) {
                                let hex_val = value.iter().fold(0u64, |acc, v| {
                                    (acc << 1)
                                        | match v {
                                            Value::V1 => 1,
                                            _ => 0,
                                        }
                                });
                                let width = ((size + 3) / 4) as usize;
                                let hex_str = format!("{:0width$X}", hex_val, width = width);
                                values.push((current_time, WaveValue::Bus(hex_str)));
                            }
                        }
                    }
                }
                _ => continue,
            }
        }

        Ok(())
    }

    pub fn get_visible_values(&self, signal: &str) -> Vec<(u64, WaveValue)> {
        if let Some(values) = self.values.get(signal) {
            values
                .iter()
                .filter(|(t, _)| {
                    *t >= self.waveform.time_start
                        && *t < self.waveform.time_start + self.waveform.time_range
                })
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }
}

// Helper function to check if two WaveValues are equal
fn values_equal(v1: &WaveValue, v2: &WaveValue) -> bool {
    match (v1, v2) {
        (WaveValue::Binary(b1), WaveValue::Binary(b2)) => b1 == b2,
        (WaveValue::Bus(s1), WaveValue::Bus(s2)) => s1 == s2,
        _ => false,
    }
}

// Helper function to format transition
fn format_transition(before: &WaveValue, after: &WaveValue) -> String {
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
