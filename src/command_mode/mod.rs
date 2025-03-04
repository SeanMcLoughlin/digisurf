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

    // The state that wraps command mode state and is used in the top-level ratatui application
    struct TestAppState {
        command_state: CommandModeState,
    }

    impl CommandModeStateAccess for TestAppState {
        fn command_state(&self) -> &CommandModeState {
            &self.command_state
        }

        fn command_state_mut(&mut self) -> &mut CommandModeState {
            &mut self.command_state
        }
    }

    #[test]
    fn test_command_builder() {
        // Create a test app state
        let mut app_state = TestAppState {
            command_state: CommandModeState::new(),
        };

        // Create a command mode widget that implements the CommandRegistry trait
        let mut command_widget = CommandModeWidget::new();

        // Define a struct to create the echo command
        struct EchoCommand;
        impl EchoCommand {
            fn create() -> Rc<Box<dyn Command<TestAppState>>> {
                CommandBuilder::new("echo", "Displays the arguments provided", |args, _state| {
                    if args.is_empty() {
                        Ok("Nothing to echo".to_string())
                    } else {
                        Ok(args.join(" "))
                    }
                })
                .build()
            }
        }

        // Define a struct to create the greet command
        struct GreetCommand;
        impl GreetCommand {
            fn create() -> Rc<Box<dyn Command<TestAppState>>> {
                CommandBuilder::new("greet", "Greets a person", |args, _state| {
                    if args.is_empty() {
                        Ok("Hello, world!".to_string())
                    } else {
                        Ok(format!("Hello, {}!", args[0]))
                    }
                })
                .alias("hello")
                .alias("hi")
                .build()
            }
        }

        command_widget.register_command(EchoCommand::create());
        command_widget.register_command(GreetCommand::create());

        // Test echo command
        let result = command_widget
            .parser()
            .execute("echo test 123", &mut app_state);
        assert_eq!(result, Ok("test 123".to_string()));

        // Test greet command
        let result = command_widget
            .parser()
            .execute("greet Alice", &mut app_state);
        assert_eq!(result, Ok("Hello, Alice!".to_string()));

        // Test greet command with no args
        let result = command_widget.parser().execute("greet", &mut app_state);
        assert_eq!(result, Ok("Hello, world!".to_string()));

        // Test alias
        let result = command_widget.parser().execute("hello Bob", &mut app_state);
        assert_eq!(result, Ok("Hello, Bob!".to_string()));

        // Test another alias
        let result = command_widget
            .parser()
            .execute("hi Charlie", &mut app_state);
        assert_eq!(result, Ok("Hello, Charlie!".to_string()));

        // Test invalid command
        let result = command_widget.parser().execute("unknown", &mut app_state);
        assert!(result.is_err());
    }

    #[test]
    fn test_command_state_cursor_movement() {
        let mut state = CommandModeState::new();
        state.input_buffer = "test".to_string();

        // Test cursor movement
        state.cursor_position = 0;
        state.move_cursor_right();
        assert_eq!(state.cursor_position, 1);

        state.move_cursor_right();
        assert_eq!(state.cursor_position, 2);

        state.move_cursor_left();
        assert_eq!(state.cursor_position, 1);

        // Test bounds
        state.cursor_position = 0;
        state.move_cursor_left();
        assert_eq!(state.cursor_position, 0);

        state.cursor_position = 4; // at the end
        state.move_cursor_right();
        assert_eq!(state.cursor_position, 4);

        // Test start/end
        state.move_cursor_start();
        assert_eq!(state.cursor_position, 0);

        state.move_cursor_end();
        assert_eq!(state.cursor_position, 4);
    }

    #[test]
    fn test_command_state_editing() {
        let mut state = CommandModeState::new();

        // Test insertion
        state.insert('a');
        assert_eq!(state.input_buffer, "a");
        assert_eq!(state.cursor_position, 1);

        state.insert('b');
        assert_eq!(state.input_buffer, "ab");
        assert_eq!(state.cursor_position, 2);

        // Test insertion in the middle
        state.move_cursor_start();
        state.insert('c');
        assert_eq!(state.input_buffer, "cab");
        assert_eq!(state.cursor_position, 1);

        // Test backspace
        state.backspace();
        assert_eq!(state.input_buffer, "ab");
        assert_eq!(state.cursor_position, 0);

        // Test delete
        state.delete();
        assert_eq!(state.input_buffer, "b");
        assert_eq!(state.cursor_position, 0);

        // Test backspace at start of buffer
        state.backspace();
        assert_eq!(state.input_buffer, "b");
        assert_eq!(state.cursor_position, 0);

        // Test delete at end of buffer
        state.move_cursor_end();
        state.delete();
        assert_eq!(state.input_buffer, "b");
        assert_eq!(state.cursor_position, 1);
    }

    #[test]
    fn test_command_history() {
        let mut state = CommandModeState::new();

        // Add commands to history
        for cmd in ["command1", "command2", "command3"] {
            state.input_buffer = cmd.to_string();
            state.add_to_history();
        }

        // Clear input and test history navigation
        state.input_buffer.clear();
        state.cursor_position = 0;

        // Navigate to previous commands
        state.previous_history();
        assert_eq!(state.input_buffer, "command3");

        state.previous_history();
        assert_eq!(state.input_buffer, "command2");

        state.previous_history();
        assert_eq!(state.input_buffer, "command1");

        // Can't go back further than oldest command
        state.previous_history();
        assert_eq!(state.input_buffer, "command1");

        // Navigate forward
        state.next_history();
        assert_eq!(state.input_buffer, "command2");

        state.next_history();
        assert_eq!(state.input_buffer, "command3");

        // Forward past the newest command should clear the buffer
        state.next_history();
        assert_eq!(state.input_buffer, "");
    }
}
