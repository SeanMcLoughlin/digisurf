// Types that all file parsers must use to extract data from their files.

use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    V0,
    V1,
    VX,
    VZ,
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
