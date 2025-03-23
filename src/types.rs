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
}
