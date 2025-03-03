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
