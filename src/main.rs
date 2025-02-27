mod app;
mod input;
mod model;
mod ui;
use app::App;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use input::{handler::handle_input, keybindings::KeyBindings};
use ratatui::{backend::CrosstermBackend, layout::Rect, Terminal};
use std::{error::Error, io, time::Duration};

fn main() -> Result<(), Box<dyn Error>> {
    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run app
    let res = run_app(&mut terminal, App::new());

    // Cleanup terminal
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

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
) -> io::Result<()> {
    let tick_rate = Duration::from_millis(250);
    let keybinds = KeyBindings::default();

    // Keep track of the waveform area for mouse interaction
    let mut waveform_area = Rect::default();

    loop {
        terminal.draw(|f| {
            let layout = ui::layout::create_layout(f.area());
            waveform_area = layout.waveforms;
            ui::draw(&app, f)
        })?;

        if event::poll(tick_rate)? {
            match event::read()? {
                Event::Key(key) => {
                    if !handle_input(&mut app, key.code, &keybinds) {
                        return Ok(());
                    }
                }
                Event::Mouse(mouse) => {
                    if !input::handler::handle_mouse(
                        &mut app,
                        mouse.kind,
                        mouse.column,
                        mouse.row,
                        mouse.modifiers,
                        waveform_area,
                    ) {
                        return Ok(());
                    }
                }
                _ => {}
            }
        }
    }
}
