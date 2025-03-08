use crate::{
    command_mode::{builder::CommandBuilder, registry::Command},
    state::AppState,
};
use std::rc::Rc;

pub fn create() -> Rc<Box<dyn Command<AppState>>> {
    CommandBuilder::new("quit", "Quit digisurf", |_args, state: &mut AppState| {
        state.exit = true;
        Ok("Exiting digisurf...".to_string())
    })
    .alias("q")
    .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quit_command() {
        let command = create();
        let mut state = AppState::default();

        assert_eq!(state.exit, false);

        let result = command.execute(&[], &mut state);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Exiting digisurf...".to_string());
        assert_eq!(state.exit, true);
    }

    #[test]
    fn test_quit_command_with_args() {
        let command = create();
        let mut state = AppState::default();
        let result = command.execute(&["unused_arg"], &mut state);

        assert!(result.is_ok());
        assert_eq!(state.exit, true);
    }
}
