use super::registry::Command;
use std::rc::Rc;

/// A builder pattern implementation for easily creating commands
pub struct CommandBuilder<S> {
    name: String,
    aliases: Vec<String>,
    description: String,
    handler: Box<dyn Fn(&[&str], &mut S) -> Result<String, String>>,
}

impl<S> CommandBuilder<S> {
    /// Create a new command builder with the required name, description and handler
    pub fn new<F>(name: impl Into<String>, description: impl Into<String>, handler: F) -> Self
    where
        F: Fn(&[&str], &mut S) -> Result<String, String> + 'static,
    {
        Self {
            name: name.into(),
            aliases: Vec::new(),
            description: description.into(),
            handler: Box::new(handler),
        }
    }

    /// Add an alias for the command
    pub fn alias(mut self, alias: impl Into<String>) -> Self {
        self.aliases.push(alias.into());
        self
    }

    /// Add multiple aliases for the command
    #[allow(dead_code)]
    pub fn aliases(mut self, aliases: impl IntoIterator<Item = impl Into<String>>) -> Self {
        for alias in aliases {
            self.aliases.push(alias.into());
        }
        self
    }

    /// Build the command
    pub fn build(self) -> Rc<Box<dyn Command<S>>>
    where
        S: 'static,
    {
        Rc::new(Box::new(BuiltCommand {
            name: self.name,
            aliases: self.aliases,
            description: self.description,
            handler: self.handler,
        }))
    }
}

/// The internal command implementation created by the builder
struct BuiltCommand<S> {
    name: String,
    aliases: Vec<String>,
    description: String,
    handler: Box<dyn Fn(&[&str], &mut S) -> Result<String, String>>,
}

impl<S> Command<S> for BuiltCommand<S> {
    fn name(&self) -> &str {
        &self.name
    }

    fn aliases(&self) -> Vec<&str> {
        self.aliases.iter().map(|s| s.as_str()).collect()
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn execute(&self, args: &[&str], state: &mut S) -> Result<String, String> {
        (self.handler)(args, state)
    }
}
