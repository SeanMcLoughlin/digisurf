use crate::model::types::WaveValue;
use std::{collections::HashMap, error::Error, io::BufReader};
use vcd::{Command, Parser, Value};

pub struct App {
    pub signals: Vec<String>,
    pub values: HashMap<String, Vec<(u64, WaveValue)>>,
    pub selected_signal: usize,
    pub time_offset: u64,
    pub window_size: u64,
    pub max_time: u64,
    pub show_help: bool,
}

impl App {
    pub fn new() -> App {
        let mut app = App {
            signals: Vec::new(),
            values: HashMap::new(),
            selected_signal: 0,
            time_offset: 0,
            window_size: 50,
            max_time: 0,
            show_help: false,
        };
        app.generate_test_data();
        app
    }

    pub fn generate_test_data(&mut self) {
        let test_signals = vec![
            "clk".to_string(),
            "reset".to_string(),
            "data_valid".to_string(),
            "data".to_string(),
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
                .filter(|(t, _)| *t >= self.time_offset && *t < self.time_offset + self.window_size)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }
}
