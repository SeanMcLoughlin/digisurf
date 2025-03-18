use crate::{
    command_mode::{builder::CommandBuilder, registry::Command},
    state::AppState,
    types::AppMode,
};
use std::rc::Rc;

pub fn create() -> Rc<Box<dyn Command<AppState>>> {
    CommandBuilder::new(
        "findsignal",
        "Open signal finder to select signals to display",
        |_args, state: &mut AppState| {
            state.mode = AppMode::FuzzyFinder;
            Ok(String::new()) // Finder window will pop up, so don't have a confirmation message
        },
    )
    .alias("fs")
    .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_findsignal_command() {
        let result = create().execute(&[], &mut AppState::default());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), String::new());
    }
}
