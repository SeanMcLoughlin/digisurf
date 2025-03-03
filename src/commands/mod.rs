mod goto;
mod help;
mod marker;
mod quit;
mod zoom;
mod zoomfull;

use crate::{command_mode::registry::Command, state::AppState};
use std::rc::Rc;

pub trait CommandRegistry<S> {
    fn register_command(&mut self, command: Rc<Box<dyn Command<S>>>);
}

impl<S> CommandRegistry<S> for crate::command_mode::CommandModeWidget<S> {
    fn register_command(&mut self, command: Rc<Box<dyn Command<S>>>) {
        self.register_command(command);
    }
}

pub fn register_all_commands(registry: &mut impl CommandRegistry<AppState>) {
    registry.register_command(zoom::create());
    registry.register_command(zoomfull::create());
    registry.register_command(goto::create());
    registry.register_command(marker::create());
    registry.register_command(help::create());
    registry.register_command(quit::create());
}
