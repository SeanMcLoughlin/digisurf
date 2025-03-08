pub mod state;
use state::FuzzyFinderState;

pub trait FuzzyFinderStateAccess {
    fn fuzzy_finder_state(&self) -> &FuzzyFinderState;
    fn fuzzy_finder_state_mut(&mut self) -> &mut FuzzyFinderState;
}
