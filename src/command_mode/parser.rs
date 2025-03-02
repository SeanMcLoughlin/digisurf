use super::registry::CommandRegistry;

pub struct CommandParser<S> {
    registry: CommandRegistry<S>,
}

impl<S> CommandParser<S> {
    pub fn new() -> Self {
        let registry = CommandRegistry::new();
        Self { registry }
    }

    pub fn with_registry(registry: CommandRegistry<S>) -> Self {
        Self { registry }
    }

    pub fn execute(&self, input: &str, state: &mut S) -> Result<String, String> {
        let parts: Vec<&str> = input.trim().split_whitespace().collect();

        if parts.is_empty() {
            return Err("No command provided".to_string());
        }

        let command_name = parts[0];
        let args = &parts[1..];

        match self.registry.get(command_name) {
            Some(command) => command.execute(args, state),
            None => Err(format!("Unknown command: {}", command_name)),
        }
    }

    pub fn list_commands(&self) -> Vec<(&str, &str)> {
        self.registry.list_commands()
    }

    pub fn registry_mut(&mut self) -> &mut CommandRegistry<S> {
        &mut self.registry
    }
}
