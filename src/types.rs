#[derive(Clone)]
pub enum WaveValue {
    Binary(vcd::Value),
    Bus(String),
}

#[derive(Debug, Copy, Clone, Default, Eq, PartialEq)]
pub enum AppMode {
    #[default]
    Normal,
    Command,
}
