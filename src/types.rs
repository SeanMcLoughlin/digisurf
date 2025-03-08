#[derive(Debug, Copy, Clone, Default, Eq, PartialEq)]
pub enum AppMode {
    #[default]
    Normal,
    Command,
    FuzzyFinder,
}
