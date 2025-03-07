mod app;
mod cli;
mod command_mode;
mod commands;
mod config;
mod constants;
mod parsers;
mod state;
mod types;
mod ui;
use app::App;
use clap::Parser;
use cli::CliArgs;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io};

fn main() -> Result<(), Box<dyn Error>> {
    let args = CliArgs::parse();
    config::load_config(args.config_file)?;

    let mut app = App::default();
    if let Some(file_path) = args.file_name {
        let result = if file_path.ends_with(".vcd") {
            app.load_vcd_file(file_path)
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Unsupported file format. Only .vcd files are supported.",
            ))
        };

        match result {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error loading waveform file: {}", e);
            }
        }
    }

    // Terminal setup
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
    let terminal = ratatui::init();

    let app_result = app.run(terminal);
    ratatui::restore();

    // Terminal cleanup
    disable_raw_mode()?;
    if let Err(err) = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture) {
        println!("{:?}", err)
    }

    app_result
}
