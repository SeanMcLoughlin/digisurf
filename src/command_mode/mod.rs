pub mod builder;
mod parser;
pub mod registry;
use parser::CommandParser;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph, Widget},
};
use registry::Command;
use std::rc::Rc;

pub struct CommandModeWidget<S> {
    is_active: bool,
    input_buffer: String,
    result_message: Option<String>,
    result_is_error: bool,
    command_parser: CommandParser<S>,
}

impl<S> CommandModeWidget<S> {
    pub fn new() -> Self {
        Self {
            is_active: false,
            input_buffer: String::new(),
            result_message: None,
            result_is_error: false,
            command_parser: CommandParser::new(),
        }
    }

    pub fn with_parser(parser: CommandParser<S>) -> Self {
        Self {
            is_active: false,
            input_buffer: String::new(),
            result_message: None,
            result_is_error: false,
            command_parser: parser,
        }
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn activate(&mut self) {
        self.is_active = true;
        self.input_buffer.clear();
        self.result_message = None;
    }

    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.input_buffer.clear();
    }

    pub fn input(&mut self, key: char) {
        if self.is_active {
            self.input_buffer.push(key);
        }
    }

    pub fn backspace(&mut self) {
        if self.is_active && !self.input_buffer.is_empty() {
            self.input_buffer.pop();
        }
    }

    pub fn execute(&mut self, state: &mut S) {
        if self.is_active {
            let command = self.input_buffer.clone();
            let result = self.command_parser.execute(&command, state);
            match result {
                Ok(msg) => {
                    self.result_message = Some(msg);
                    self.result_is_error = false;
                }
                Err(err) => {
                    self.result_message = Some(err);
                    self.result_is_error = true;
                }
            }
            self.deactivate();
        }
    }

    pub fn register_command(&mut self, command: Rc<Box<dyn Command<S>>>) {
        self.command_parser.registry_mut().register(command);
    }

    pub fn render(&self) -> CommandModeRender {
        CommandModeRender {
            is_active: self.is_active,
            input_buffer: &self.input_buffer,
            result_message: self.result_message.as_deref(),
            result_is_error: self.result_is_error,
        }
    }

    pub fn clear_result(&mut self) {
        self.result_message = None;
    }

    pub fn parser(&self) -> &CommandParser<S> {
        &self.command_parser
    }

    pub fn parser_mut(&mut self) -> &mut CommandParser<S> {
        &mut self.command_parser
    }
}

pub struct CommandModeRender<'a> {
    is_active: bool,
    input_buffer: &'a str,
    result_message: Option<&'a str>,
    result_is_error: bool,
}

impl<'a> Widget for CommandModeRender<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let block = Block::default()
            .borders(Borders::TOP)
            .style(Style::default().fg(Color::White));

        if self.is_active {
            // Render command input
            let prompt = Span::styled(":", Style::default().fg(Color::Yellow));
            let input = Span::raw(self.input_buffer);
            let text = ratatui::text::Line::from(vec![prompt, input]);

            Paragraph::new(text)
                .block(block)
                .style(Style::default().fg(Color::White))
                .render(area, buf);
        } else if let Some(message) = self.result_message {
            // Render command result
            let style = if self.result_is_error {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::Green)
            };

            let text = ratatui::text::Line::from(Span::styled(message, style));

            Paragraph::new(text)
                .block(block)
                .style(Style::default().fg(Color::White))
                .render(area, buf);
        } else {
            // Render empty command box
            Paragraph::new("")
                .block(block)
                .style(Style::default().fg(Color::White))
                .render(area, buf);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use builder::CommandBuilder;

    #[test]
    fn test_generic_commands() {
        struct TestState {
            value: i32,
        }

        let mut widget = CommandModeWidget::<TestState>::new();

        widget.register_command(
            CommandBuilder::new(
                "increment",
                "Increment the value",
                |args, state: &mut TestState| {
                    if args.is_empty() {
                        state.value += 1;
                        Ok(format!("Incremented to {}", state.value))
                    } else if let Ok(amount) = args[0].parse::<i32>() {
                        state.value += amount;
                        Ok(format!("Incremented by {} to {}", amount, state.value))
                    } else {
                        Err("Invalid increment amount".to_string())
                    }
                },
            )
            .alias("inc")
            .build(),
        );

        let mut state = TestState { value: 5 };

        // Test command execution
        widget.activate();
        widget.input_buffer = "increment".to_string();
        widget.execute(&mut state);

        assert_eq!(state.value, 6);
        assert_eq!(widget.result_message, Some("Incremented to 6".to_string()));
        assert_eq!(widget.result_is_error, false);

        // Test with alias
        widget.activate();
        widget.input_buffer = "inc 10".to_string();
        widget.execute(&mut state);

        assert_eq!(state.value, 16);
    }

    #[test]
    fn test_generic_commands_with_error() {
        struct TestState {
            value: i32,
        }

        let mut widget = CommandModeWidget::<TestState>::new();

        widget.register_command(
            CommandBuilder::new(
                "increment",
                "Increment the value",
                |args, state: &mut TestState| {
                    if args.is_empty() {
                        state.value += 1;
                        Ok(format!("Incremented to {}", state.value))
                    } else if let Ok(amount) = args[0].parse::<i32>() {
                        state.value += amount;
                        Ok(format!("Incremented by {} to {}", amount, state.value))
                    } else {
                        Err("Invalid increment amount".to_string())
                    }
                },
            )
            .alias("inc")
            .build(),
        );

        let mut state = TestState { value: 5 };

        // Test command execution
        widget.activate();
        widget.input_buffer = "increment".to_string();
        widget.execute(&mut state);

        assert_eq!(state.value, 6);
        assert_eq!(widget.result_message, Some("Incremented to 6".to_string()));
        assert_eq!(widget.result_is_error, false);

        // Test with alias
        widget.activate();
        widget.input_buffer = "inc 10".to_string();
        widget.execute(&mut state);

        assert_eq!(state.value, 16);
    }
}
