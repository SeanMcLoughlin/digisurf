// Types that all file parsers must use to extract data from their files.

use std::{collections::HashMap, fmt::Display};

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    V0,
    V1,
    VX,
    VZ,
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::V0 => write!(f, "0"),
            Value::V1 => write!(f, "1"),
            Value::VX => write!(f, "X"),
            Value::VZ => write!(f, "Z"),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum WaveValue {
    Binary(Value),
    Bus(String),
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct WaveformData {
    pub signals: Vec<String>,
    pub values: HashMap<String, Vec<(u64, WaveValue)>>,
    pub max_time: u64,
}
