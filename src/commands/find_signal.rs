// src/commands/find_signal.rs
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
            Ok("Opening signal finder".to_string())
        },
    )
    .alias("fs")
    .build()
}
