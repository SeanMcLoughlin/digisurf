mod app;
mod input;
mod model;
mod ui;
use app::App;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io};

fn main() -> Result<(), Box<dyn Error>> {
    // Terminal setup
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;

    let terminal = ratatui::init();
    let app_result = App::default().run(terminal);
    ratatui::restore();

    // Terminal cleanup
    disable_raw_mode()?;
    if let Err(err) = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture) {
        println!("{:?}", err)
    }

    app_result
}
