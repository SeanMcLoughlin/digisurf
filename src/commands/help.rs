use crate::{command_mode::builder::CommandBuilder, state::AppState};
use std::rc::Rc;

pub fn create() -> Rc<Box<dyn crate::command_mode::registry::Command<AppState>>> {
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
