#[derive(Clone)]
pub enum WaveValue {
    Binary(vcd::Value),
    Bus(String),
}
