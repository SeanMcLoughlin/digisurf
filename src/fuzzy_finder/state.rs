use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use ratatui::widgets::ListState;
use std::collections::HashSet;

#[derive(Default)]
pub struct FuzzyFinderState {
    pub list_state: ListState,
    pub query: String,
    pub filtered_signals: Vec<String>,
    pub all_signals: Vec<String>,
    pub selected_signals: HashSet<String>,
    pub matcher: SkimMatcherV2,
}

impl FuzzyFinderState {
    pub fn set_signals(&mut self, signals: Vec<String>, displayed_signals: &[String]) {
        self.all_signals = signals;

        // Mark currently displayed signals as selected
        self.selected_signals = displayed_signals.iter().cloned().collect();

        self.update_filtered_signals();

        // Set initial selection if there are items
        if !self.filtered_signals.is_empty() {
            self.list_state.select(Some(0));
        }
    }

    pub fn handle_input(&mut self, c: char) {
        self.query.push(c);
        self.update_filtered_signals();
    }

    pub fn handle_backspace(&mut self) {
        if !self.query.is_empty() {
            self.query.pop();
            self.update_filtered_signals();
        }
    }

    pub fn select_next(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.filtered_signals.len().saturating_sub(1) {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn select_previous(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.filtered_signals.len().saturating_sub(1)
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn toggle_selected_signal(&mut self) {
        if let Some(idx) = self.list_state.selected() {
            if let Some(signal) = self.filtered_signals.get(idx) {
                if self.selected_signals.contains(signal) {
                    self.selected_signals.remove(signal);
                } else {
                    self.selected_signals.insert(signal.clone());
                }
            }
        }
    }

    pub fn get_selected_signals(&self) -> Vec<String> {
        self.selected_signals.iter().cloned().collect()
    }

    pub fn clear_selection(&mut self) {
        self.selected_signals.clear();
    }

    pub fn select_all(&mut self) {
        for signal in &self.filtered_signals {
            self.selected_signals.insert(signal.clone());
        }
    }

    fn update_filtered_signals(&mut self) {
        if self.query.is_empty() {
            // If query is empty, show all signals
            self.filtered_signals = self.all_signals.clone();
        } else {
            // Otherwise, filter signals based on fuzzy matching
            let mut matches: Vec<(String, i64)> = self
                .all_signals
                .iter()
                .filter_map(|signal| {
                    self.matcher
                        .fuzzy_match(signal, &self.query)
                        .map(|score| (signal.clone(), score))
                })
                .collect();

            // Sort by match score (descending)
            matches.sort_by(|a, b| b.1.cmp(&a.1));

            // Extract just the signal names
            self.filtered_signals = matches.into_iter().map(|(signal, _)| signal).collect();
        }

        // Adjust selection if necessary
        if let Some(selected) = self.list_state.selected() {
            if selected >= self.filtered_signals.len() {
                if self.filtered_signals.is_empty() {
                    self.list_state.select(None);
                } else {
                    self.list_state.select(Some(0));
                }
            }
        } else if !self.filtered_signals.is_empty() {
            self.list_state.select(Some(0));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_finder_filtering() {
        let mut state = FuzzyFinderState::default();

        // Initialize with some signals
        let signals = vec![
            "test_signal_1".to_string(),
            "test_signal_2".to_string(),
            "another_signal".to_string(),
            "control_pin".to_string(),
        ];

        state.set_signals(signals, &[]);

        // Initial state should have all signals
        assert_eq!(state.filtered_signals.len(), 4);

        // Test basic filtering
        state.query = "test".to_string();
        state.update_filtered_signals();
        assert_eq!(state.filtered_signals.len(), 2);
        assert!(state
            .filtered_signals
            .contains(&"test_signal_1".to_string()));
        assert!(state
            .filtered_signals
            .contains(&"test_signal_2".to_string()));

        // Test fuzzy matching
        state.query = "tsg".to_string(); // should match test_signal
        state.update_filtered_signals();
        assert!(state.filtered_signals.len() > 0);

        // Test selection operations
        state.query = "".to_string();
        state.update_filtered_signals();

        // Select an item
        state.list_state.select(Some(0));
        state.toggle_selected_signal();
        assert_eq!(state.selected_signals.len(), 1);

        // Select all
        state.select_all();
        assert_eq!(state.selected_signals.len(), 4);

        // Clear all
        state.clear_selection();
        assert_eq!(state.selected_signals.len(), 0);
    }

    #[test]
    fn test_fuzzy_finder_navigation() {
        let mut state = FuzzyFinderState::default();

        // Initialize with some signals
        let signals = vec![
            "signal1".to_string(),
            "signal2".to_string(),
            "signal3".to_string(),
        ];

        state.set_signals(signals, &[]);

        // Test navigation
        assert_eq!(state.list_state.selected(), Some(0));

        state.select_next();
        assert_eq!(state.list_state.selected(), Some(1));

        state.select_previous();
        assert_eq!(state.list_state.selected(), Some(0));

        // Test wrap-around
        state.select_previous();
        assert_eq!(state.list_state.selected(), Some(2)); // Wrapped to last

        state.select_next();
        assert_eq!(state.list_state.selected(), Some(0)); // Wrapped to first
    }
}
