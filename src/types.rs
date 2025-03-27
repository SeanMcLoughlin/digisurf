#[derive(Debug, Copy, Clone, Default, Eq, PartialEq)]
pub enum AppMode {
    #[default]
    Normal,
    Command,
    FuzzyFinder,
}
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct Marker {
    pub time: u64,
    pub name: String,
    pub color: ratatui::style::Color,
}

impl Marker {
    pub fn new(time: u64, name: String) -> Self {
        Self {
            time,
            name,
            color: crate::constants::DEFAULT_SAVED_MARKER_COLOR,
        }
    }
}
