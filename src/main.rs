use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, List, ListItem},
    Terminal,
};
use std::{
    collections::HashMap,
    error::Error,
    io::{self, BufReader},
    time::Duration,
};
use vcd::{self, Command, Parser, Value};

// App state
struct App {
    signals: Vec<String>,
    values: HashMap<String, Vec<(u64, Value)>>, // (timestamp, value) pairs for each signal
    selected_signal: usize,
    time_offset: u64,
    window_size: u64,
    max_time: u64,
}

impl App {
    fn new() -> App {
        App {
            signals: Vec::new(),
            values: HashMap::new(),
            selected_signal: 0,
            time_offset: 0,
            window_size: 50,
            max_time: 0,
        }
    }

    // Add this function here
    fn generate_test_data(&mut self) {
        // Create some test signals
        let test_signals = vec![
            "clk".to_string(),
            "reset".to_string(),
            "data_valid".to_string(),
            "data".to_string(),
        ];

        // Generate test data
        for signal in test_signals {
            self.signals.push(signal.clone());
            let mut values = Vec::new();

            // Generate 100 time units of data
            for t in 0..100 {
                let value = match signal.as_str() {
                    "clk" => {
                        if t % 2 == 0 {
                            Value::V1
                        } else {
                            Value::V0
                        }
                    } // Clock toggles every time unit
                    "reset" => {
                        if t < 10 {
                            Value::V1
                        } else {
                            Value::V0
                        }
                    } // Reset active for first 10 cycles
                    "data_valid" => {
                        if t % 10 == 0 {
                            Value::V1
                        } else {
                            Value::V0
                        }
                    } // Data valid every 10 cycles
                    "data" => {
                        if t % 20 < 10 {
                            Value::V1
                        } else {
                            Value::V0
                        }
                    } // Data alternates every 10 cycles
                    _ => Value::V0,
                };
                values.push((t as u64, value));
            }

            self.values.insert(signal, values);
        }
        self.max_time = 100;
    }

    fn load_vcd(&mut self, filename: &str) -> Result<(), Box<dyn Error>> {
        let file = std::fs::File::open(filename)?;
        let reader = BufReader::new(file);
        let mut parser = Parser::new(reader);

        let header = parser.parse_header()?;

        // Track current timestamp
        let mut current_time = 0u64;

        let mut id_to_signal = HashMap::new();

        for item in header.items {
            if let vcd::ScopeItem::Var(var) = item {
                // Extract signal names from scopes
                let signal_name = var.reference.clone();
                self.signals.push(signal_name.clone());
                self.values.insert(signal_name, Vec::new());

                // Create a mapping from IdCode to signal name during header parsing
                id_to_signal.insert(var.code.clone(), var.reference.clone());
            }
        }

        // Parse value changes
        while let Some(command) = parser.next() {
            match command? {
                Command::Timestamp(time) => {
                    current_time = time;
                    self.max_time = self.max_time.max(time);
                }
                Command::ChangeScalar(id, value) => {
                    // Use our id_to_signal mapping
                    if let Some(signal_name) = id_to_signal.get(&id) {
                        if let Some(values) = self.values.get_mut(signal_name) {
                            values.push((current_time, value));
                        }
                    }
                }
                _ => continue,
            }
        }

        Ok(())
    }

    fn get_visible_values(&self, signal: &str) -> Vec<(u64, Value)> {
        if let Some(values) = self.values.get(signal) {
            values
                .iter()
                .filter(|(t, _)| *t >= self.time_offset && *t < self.time_offset + self.window_size)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
) -> io::Result<()> {
    let tick_rate = Duration::from_millis(250);

    loop {
        terminal.draw(|f| {
            let size = f.area();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
                .split(size);

            // Title block with time information
            let title = format!(
                "VCD Waveform Viewer | Time: {} to {} of {}",
                app.time_offset,
                app.time_offset + app.window_size,
                app.max_time
            );
            let title_block = Block::default().title(title).borders(Borders::ALL);
            f.render_widget(title_block, chunks[0]);

            // Signal list
            let signals: Vec<ListItem> = app
                .signals
                .iter()
                .enumerate()
                .map(|(i, name)| {
                    let style = if i == app.selected_signal {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default()
                    };

                    // Get current value at time_offset
                    let current_value = if let Some(values) = app.values.get(name) {
                        let val = values
                            .iter()
                            .rev()
                            .find(|(t, _)| *t <= app.time_offset)
                            .map(|(_, v)| v.to_string())
                            .unwrap_or_else(|| "x".to_string());
                        format!("{}: {}", name, val)
                    } else {
                        name.clone()
                    };

                    ListItem::new(Span::styled(current_value, style))
                })
                .collect();

            let signals_list = List::new(signals).block(Block::default().borders(Borders::ALL));

            f.render_widget(signals_list, chunks[1]);
        })?;

        // Handle input
        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Down => {
                        app.selected_signal = (app.selected_signal + 1) % app.signals.len();
                    }
                    KeyCode::Up => {
                        if app.selected_signal > 0 {
                            app.selected_signal -= 1;
                        } else {
                            app.selected_signal = app.signals.len() - 1;
                        }
                    }
                    KeyCode::Left => {
                        if app.time_offset > 0 {
                            app.time_offset = app.time_offset.saturating_sub(app.window_size / 4);
                        }
                    }
                    KeyCode::Right => {
                        if app.time_offset < app.max_time {
                            app.time_offset = (app.time_offset + app.window_size / 4)
                                .min(app.max_time - app.window_size);
                        }
                    }
                    KeyCode::Char('+') => {
                        app.window_size = (app.window_size * 2).min(app.max_time);
                    }
                    KeyCode::Char('-') => {
                        app.window_size = (app.window_size / 2).max(10);
                    }
                    _ => {}
                }
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and generate test data
    let mut app = App::new();
    app.generate_test_data();

    // Run app
    let res = run_app(&mut terminal, app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}
