#[derive(Default)]
pub struct CommandModeState {
    pub input_buffer: String,
    pub cursor_position: usize,
    pub history: Vec<String>,
    pub history_index: Option<usize>,
    pub result_message: Option<String>,
    pub command_result_time: Option<std::time::Instant>,
    pub result_is_error: bool,
}

impl CommandModeState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, c: char) {
        self.input_buffer.insert(self.cursor_position, c);
        self.cursor_position += 1;
    }

    pub fn delete(&mut self) {
        if self.cursor_position < self.input_buffer.len() {
            self.input_buffer.remove(self.cursor_position);
        }
    }

    pub fn backspace(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.input_buffer.remove(self.cursor_position);
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.input_buffer.len() {
            self.cursor_position += 1;
        }
    }

    pub fn move_cursor_end(&mut self) {
        self.cursor_position = self.input_buffer.len();
    }

    pub fn move_cursor_start(&mut self) {
        self.cursor_position = 0;
    }

    pub fn clear(&mut self) {
        self.input_buffer.clear();
        self.cursor_position = 0;
        self.history_index = None;
        self.result_message = None;
        self.result_is_error = false;
    }

    pub fn add_to_history(&mut self) {
        if !self.input_buffer.is_empty() {
            self.history.push(self.input_buffer.clone());
            self.history_index = None;
        }
    }

    pub fn previous_history(&mut self) {
        if let Some(index) = self.history_index {
            if index > 0 {
                self.history_index = Some(index - 1);
                self.input_buffer = self.history[index - 1].clone();
                self.cursor_position = self.input_buffer.len();
            }
        } else if !self.history.is_empty() {
            self.history_index = Some(self.history.len() - 1);
            self.input_buffer = self.history[self.history.len() - 1].clone();
            self.cursor_position = self.input_buffer.len();
        }
    }

    pub fn next_history(&mut self) {
        if let Some(index) = self.history_index {
            if index < self.history.len() - 1 {
                self.history_index = Some(index + 1);
                self.input_buffer = self.history[index + 1].clone();
                self.cursor_position = self.input_buffer.len();
            }
        } else if !self.history.is_empty() {
            self.history_index = Some(0);
            self.input_buffer = self.history[0].clone();
            self.cursor_position = self.input_buffer.len();
        }
    }
}
