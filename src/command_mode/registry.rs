use std::{collections::HashMap, rc::Rc};

pub trait Command<S> {
    fn name(&self) -> &str;
    fn aliases(&self) -> Vec<&str> {
        vec![]
    }
    fn description(&self) -> &str;
    fn execute(&self, args: &[&str], state: &mut S) -> Result<String, String>;
}

pub struct CommandRegistry<S> {
    commands: HashMap<String, Rc<Box<dyn Command<S>>>>,
}

impl<S> CommandRegistry<S> {
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }

    pub fn register(&mut self, command: Rc<Box<dyn Command<S>>>) {
        let name = command.name().to_string();
        self.commands.insert(name.clone(), Rc::clone(&command));

        // Register any aliases
        for alias in command.aliases() {
            let command_ref = Rc::clone(&command);
            self.commands.insert(alias.to_string(), command_ref);
        }
    }

    pub fn get(&self, name: &str) -> Option<&Rc<Box<dyn Command<S>>>> {
        self.commands.get(name)
    }

    pub fn list_commands(&self) -> Vec<(&str, &str)> {
        // Only include primary commands (not aliases)
        let mut unique_commands = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for cmd in self.commands.values() {
            if !seen.contains(cmd.name()) {
                unique_commands.push((cmd.name(), cmd.description()));
                seen.insert(cmd.name());
            }
        }
        unique_commands
    }
}
