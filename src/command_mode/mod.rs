pub mod builder;
mod parser;
pub mod registry;
pub mod state;
use crossterm::event::KeyCode;
use parser::CommandParser;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph, Widget},
};
use registry::Command;
use state::CommandModeState;
use std::rc::Rc;

pub trait CommandModeStateAccess {
    fn command_state(&self) -> &CommandModeState;
    fn command_state_mut(&mut self) -> &mut CommandModeState;
}

pub struct CommandModeWidget<S> {
    is_active: bool,
    command_parser: CommandParser<S>,
}

impl<S> CommandModeWidget<S> {
    pub fn new() -> Self {
        Self {
            is_active: false,
            command_parser: CommandParser::new(),
        }
    }

    pub fn with_parser(parser: CommandParser<S>) -> Self {
        Self {
            is_active: false,
            command_parser: parser,
        }
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn handle_input(&mut self, key: KeyCode, app_state: &mut S)
    where
        S: CommandModeStateAccess,
    {
        let cmd_state = app_state.command_state_mut();

        match key {
            // Handle keyboard navigation and editing
            KeyCode::Left => cmd_state.move_cursor_left(),
            KeyCode::Right => cmd_state.move_cursor_right(),
            KeyCode::Home => cmd_state.move_cursor_start(),
            KeyCode::End => cmd_state.move_cursor_end(),
            KeyCode::Backspace => cmd_state.backspace(),
            KeyCode::Delete => cmd_state.delete(),
            KeyCode::Char(c) => cmd_state.insert(c),
            // Up/down for history navigation
            KeyCode::Up => cmd_state.previous_history(),
            KeyCode::Down => cmd_state.next_history(),
            _ => {}
        }
    }

    pub fn execute(&mut self, app_state: &mut S) -> bool
    where
        S: CommandModeStateAccess + 'static,
    {
        let command = app_state.command_state().input_buffer.clone();

        if command.is_empty() {
            return false;
        }

        // Add to history
        app_state.command_state_mut().add_to_history();

        // Execute the command
        let result = self.command_parser.execute(&command, app_state);

        // Store result
        let cmd_state = app_state.command_state_mut();
        match result {
            Ok(msg) => {
                cmd_state.result_message = Some(msg);
                cmd_state.result_is_error = false;
            }
            Err(err) => {
                cmd_state.result_message = Some(err);
                cmd_state.result_is_error = true;
            }
        }

        // Clear input buffer but keep result visible
        cmd_state.input_buffer.clear();
        cmd_state.cursor_position = 0;

        true // Command was executed
    }

    pub fn register_command(&mut self, command: Rc<Box<dyn Command<S>>>) {
        self.command_parser.registry_mut().register(command);
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

    // Test structure needs to implement CommandModeStateAccess
    struct TestState {
        value: i32,
        command_state: CommandModeState,
    }

    impl CommandModeStateAccess for TestState {
        fn command_state(&self) -> &CommandModeState {
            &self.command_state
        }

        fn command_state_mut(&mut self) -> &mut CommandModeState {
            &mut self.command_state
        }
    }

    #[test]
    fn test_generic_commands() {
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

        let mut state = TestState {
            value: 5,
            command_state: CommandModeState::default(),
        };

        // Test command execution - set input buffer directly in command state
        state.command_state_mut().input_buffer = "increment".to_string();
        widget.execute(&mut state);

        assert_eq!(state.value, 6);
        assert_eq!(
            state.command_state.result_message,
            Some("Incremented to 6".to_string())
        );
        assert_eq!(state.command_state.result_is_error, false);

        // Test with alias
        state.command_state_mut().input_buffer = "inc 10".to_string();
        widget.execute(&mut state);

        assert_eq!(state.value, 16);
    }

    #[test]
    fn test_generic_commands_with_error() {
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

        let mut state = TestState {
            value: 5,
            command_state: CommandModeState::default(),
        };

        // Test command execution
        state.command_state_mut().input_buffer = "increment".to_string();
        widget.execute(&mut state);

        assert_eq!(state.value, 6);
        assert_eq!(
            state.command_state.result_message,
            Some("Incremented to 6".to_string())
        );
        assert_eq!(state.command_state.result_is_error, false);

        // Test with alias
        state.command_state_mut().input_buffer = "inc 10".to_string();
        widget.execute(&mut state);

        assert_eq!(state.value, 16);
    }
}
