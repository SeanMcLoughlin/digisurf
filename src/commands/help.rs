use crate::{
    command_mode::{builder::CommandBuilder, registry::Command},
    state::AppState,
};
use std::rc::Rc;

pub fn create() -> Rc<Box<dyn Command<AppState>>> {
    CommandBuilder::new(
        "help",
        "Show help information",
        |_args, state: &mut AppState| {
            state.show_help = !state.show_help;
            Ok("help".to_string())
        },
    )
    .alias("h")
    .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help_toggle() {
        let command = create();
        let mut state = AppState::default();
        state.show_help = false;

        // false -> true
        let result = command.execute(&[], &mut state);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "help".to_string());
        assert_eq!(state.show_help, true);

        // true -> false
        let result = command.execute(&[], &mut state);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "help".to_string());
        assert_eq!(state.show_help, false);
    }
}
