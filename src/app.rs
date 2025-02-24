use std::{collections::HashMap, error::Error, io::BufReader};
use vcd::{Command, Parser, Value};

pub struct App {
    pub signals: Vec<String>,
    pub values: HashMap<String, Vec<(u64, Value)>>,
    pub selected_signal: usize,
    pub time_offset: u64,
    pub window_size: u64,
    pub max_time: u64,
    pub show_help: bool,
}

impl App {
    pub fn new() -> App {
        App {
            signals: Vec::new(),
            values: HashMap::new(),
            selected_signal: 0,
            time_offset: 0,
            window_size: 50,
            max_time: 0,
            show_help: false,
        }
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
                            Value::V1
                        } else {
                            Value::V0
                        }
                    }
                    "reset" => {
                        if t < 10 {
                            Value::V1
                        } else {
                            Value::V0
                        }
                    }
                    "data_valid" => {
                        if t % 10 == 0 {
                            Value::V1
                        } else {
                            Value::V0
                        }
                    }
                    "data" => {
                        if t % 20 < 10 {
                            Value::V1
                        } else {
                            Value::V0
                        }
                    }
                    _ => Value::V0,
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

        for item in header.items {
            if let vcd::ScopeItem::Var(var) = item {
                let signal_name = var.reference.clone();
                self.signals.push(signal_name.clone());
                self.values.insert(signal_name, Vec::new());
                id_to_signal.insert(var.code.clone(), var.reference.clone());
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
                            values.push((current_time, value));
                        }
                    }
                }
                _ => continue,
            }
        }

        Ok(())
    }

    pub fn get_visible_values(&self, signal: &str) -> Vec<(u64, Value)> {
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
