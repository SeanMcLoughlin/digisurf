use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{
        canvas::{Canvas, Line},
        Block, Borders, List, ListItem, Paragraph,
    },
    Terminal,
};
use std::{
    collections::HashMap,
    error::Error,
    io::{self, BufReader},
    time::Duration,
};
use vcd::{self, Command, Parser, Value};

struct App {
    signals: Vec<String>,
    values: HashMap<String, Vec<(u64, Value)>>,
    selected_signal: usize,
    time_offset: u64,
    window_size: u64,
    max_time: u64,
    show_help: bool,
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
            show_help: false,
        }
    }

    fn generate_test_data(&mut self) {
        let test_signals = vec![
            "clk".to_string(),
            "reset".to_string(),
            "data_valid".to_string(),
            "data".to_string(),
        ];

        for signal in test_signals {
            self.signals.push(signal.clone());
            let mut values = Vec::new();

            for t in 0..100 {
                let value = match signal.as_str() {
                    "clk" => {
                        if t % 2 == 0 {
                            Value::V1
                        } else {
                            Value::V0
                        }
                    }
                    "reset" => {
                        if t < 10 {
                            Value::V1
                        } else {
                            Value::V0
                        }
                    }
                    "data_valid" => {
                        if t % 10 == 0 {
                            Value::V1
                        } else {
                            Value::V0
                        }
                    }
                    "data" => {
                        if t % 20 < 10 {
                            Value::V1
                        } else {
                            Value::V0
                        }
                    }
                    _ => Value::V0,
                };
                values.push((t as u64, value));
            }

            self.values.insert(signal, values);
        }
        self.max_time = 100;
    }

    #[allow(dead_code)]
    fn load_vcd(&mut self, filename: &str) -> Result<(), Box<dyn Error>> {
        let file = std::fs::File::open(filename)?;
        let reader = BufReader::new(file);
        let mut parser = Parser::new(reader);

        let header = parser.parse_header()?;

        let mut current_time = 0u64;
        let mut id_to_signal = HashMap::new();

        for item in header.items {
            if let vcd::ScopeItem::Var(var) = item {
                let signal_name = var.reference.clone();
                self.signals.push(signal_name.clone());
                self.values.insert(signal_name, Vec::new());
                id_to_signal.insert(var.code.clone(), var.reference.clone());
            }
        }

        while let Some(command) = parser.next() {
            match command? {
                Command::Timestamp(time) => {
                    current_time = time;
                    self.max_time = self.max_time.max(time);
                }
                Command::ChangeScalar(id, value) => {
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

fn draw_waveform(
    f: &mut ratatui::Frame,
    area: Rect,
    values: &[(u64, Value)],
    time_offset: u64,
    window_size: u64,
    selected: bool,
) {
    let style = if selected {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };

    let canvas = Canvas::default()
        .block(Block::default().borders(Borders::NONE))
        .paint(|ctx| {
            let width = area.width as f64;
            let mut last_value = None;
            let mut last_x = 0.0;

            let time_to_x =
                |t: u64| -> f64 { ((t - time_offset) as f64 / window_size as f64) * width };

            for (t, v) in values {
                if *t >= time_offset && *t <= time_offset + window_size {
                    let x = time_to_x(*t);
                    let y = match v {
                        Value::V1 => 0.5,
                        Value::V0 => 1.5,
                        _ => 1.0,
                    };

                    if let Some((prev_y, prev_x)) = last_value {
                        if prev_y != y {
                            ctx.draw(&Line {
                                x1: prev_x,
                                y1: prev_y,
                                x2: x,
                                y2: prev_y,
                                color: style.fg.unwrap_or(Color::White),
                            });
                            ctx.draw(&Line {
                                x1: x,
                                y1: prev_y,
                                x2: x,
                                y2: y,
                                color: style.fg.unwrap_or(Color::White),
                            });
                        }
                        ctx.draw(&Line {
                            x1: last_x,
                            y1: prev_y,
                            x2: x,
                            y2: prev_y,
                            color: style.fg.unwrap_or(Color::White),
                        });
                    }

                    last_value = Some((y, x));
                    last_x = x;
                }
            }

            if let Some((y, x)) = last_value {
                ctx.draw(&Line {
                    x1: x,
                    y1: y,
                    x2: width,
                    y2: y,
                    color: style.fg.unwrap_or(Color::White),
                });
            }
        })
        .x_bounds([0.0, area.width as f64])
        .y_bounds([0.0, 2.0]);

    f.render_widget(canvas, area);
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
) -> io::Result<()> {
    let tick_rate = Duration::from_millis(250);

    loop {
        terminal.draw(|f| {
            let size = f.area();

            if app.show_help {
                let help_text = "
                    Controls:
                    h - Toggle help menu
                    q - Quit
                    Up/Down - Select signal
                    Left/Right - Navigate timeline
                    +/- - Zoom in/out
                    ";
                let help_paragraph = Paragraph::new(help_text)
                    .block(Block::default().title("Help").borders(Borders::ALL));
                f.render_widget(help_paragraph, size);
                return;
            }

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
                .split(size);

            let title = format!(
                "VCD Waveform Viewer | Time: {} to {} of {}",
                app.time_offset,
                app.time_offset + app.window_size,
                app.max_time
            );
            let title_block = Block::default().title(title).borders(Borders::ALL);
            f.render_widget(title_block, chunks[0]);

            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
                .split(chunks[1]);

            let signals: Vec<ListItem> = app
                .signals
                .iter()
                .enumerate()
                .map(|(i, name)| {
                    ListItem::new(name.as_str()).style(if i == app.selected_signal {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default()
                    })
                })
                .collect();

            let signals_list = List::new(signals).block(Block::default().borders(Borders::ALL));

            f.render_widget(signals_list, main_chunks[0]);

            let waveform_area = main_chunks[1];
            let waveform_height = 2;

            for (i, signal) in app.signals.iter().enumerate() {
                let area = Rect::new(
                    waveform_area.x,
                    waveform_area.y + (i as u16 * waveform_height),
                    waveform_area.width,
                    waveform_height,
                );

                let values = app.get_visible_values(signal);
                draw_waveform(
                    f,
                    area,
                    &values,
                    app.time_offset,
                    app.window_size,
                    i == app.selected_signal,
                );
            }
        })?;

        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('h') => app.show_help = !app.show_help,
                    KeyCode::Down => {
                        if !app.show_help {
                            app.selected_signal = (app.selected_signal + 1) % app.signals.len();
                        }
                    }
                    KeyCode::Up => {
                        if !app.show_help {
                            if app.selected_signal > 0 {
                                app.selected_signal -= 1;
                            } else {
                                app.selected_signal = app.signals.len() - 1;
                            }
                        }
                    }
                    KeyCode::Left => {
                        if !app.show_help && app.time_offset > 0 {
                            app.time_offset = app.time_offset.saturating_sub(app.window_size / 4);
                        }
                    }
                    KeyCode::Right => {
                        if !app.show_help && app.time_offset < app.max_time {
                            app.time_offset = (app.time_offset + app.window_size / 4)
                                .min(app.max_time - app.window_size);
                        }
                    }
                    KeyCode::Char('+') => {
                        if !app.show_help {
                            app.window_size = (app.window_size * 2).min(app.max_time);
                        }
                    }
                    KeyCode::Char('-') => {
                        if !app.show_help {
                            app.window_size = (app.window_size / 2).max(10);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    app.generate_test_data();

    let res = run_app(&mut terminal, app);

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
