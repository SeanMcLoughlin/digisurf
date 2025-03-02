use crate::{command_mode::builder::CommandBuilder, state::AppState};
use std::rc::Rc;

pub fn create() -> Rc<Box<dyn crate::command_mode::registry::Command<AppState>>> {
    CommandBuilder::new("quit", "Quit digisurf", |_args, state: &mut AppState| {
        state.exit = true;
        Ok("Exiting digisurf...".to_string())
    })
    .alias("q")
    .build()
}
