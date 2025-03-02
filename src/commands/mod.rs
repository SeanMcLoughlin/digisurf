mod goto;
mod help;
mod marker;
mod quit;
mod zoom;
mod zoomfull;

use crate::state::AppState;
use std::rc::Rc;

// Define a trait for any struct that can register commands
pub trait CommandRegistry<S> {
    fn register_command(&mut self, command: Rc<Box<dyn crate::command_mode::registry::Command<S>>>);
}

// Implement the trait for our command mode widget
impl<S> CommandRegistry<S> for crate::command_mode::CommandModeWidget<S> {
    fn register_command(
        &mut self,
        command: Rc<Box<dyn crate::command_mode::registry::Command<S>>>,
    ) {
        self.register_command(command);
    }
}

// The main function that registers all commands
pub fn register_all_commands(registry: &mut impl CommandRegistry<AppState>) {
    // Register each command
    registry.register_command(zoom::create());
    registry.register_command(zoomfull::create());
    registry.register_command(goto::create());
    registry.register_command(marker::create());
    registry.register_command(help::create());
    registry.register_command(quit::create());
}
