mod app;
mod cli;
mod command_mode;
mod commands;
mod config;
mod constants;
mod fuzzy_finder;
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
    let config = config::load_config(args.config_file)?;

    let mut app = App::with_config(config);
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

#[cfg(test)]
mod tests {
    use crate::{app::App, config, types::AppMode};
    use crossterm::event::{KeyEvent, KeyModifiers};
    use std::fs;
    use tempfile::NamedTempFile;

    // Utility function to create a test VCD file
    fn create_test_vcd_file() -> NamedTempFile {
        let temp_file = NamedTempFile::new().unwrap();

        // Write a simple VCD file
        let vcd_content = r#"
    $date November 11, 2023 $end
    $version Test VCD 1.0 $end
    $timescale 1ps $end
    $scope module test $end
    $var wire 1 # clk $end
    $var wire 1 $ reset $end
    $var wire 8 % data $end
    $upscope $end
    $enddefinitions $end
    $dumpvars
    0#
    1$
    b00000000 %
    $end
    #10
    1#
    #20
    0#
    0$
    b10101010 %
        "#;

        fs::write(&temp_file, vcd_content).unwrap();
        temp_file
    }

    #[test]
    fn test_load_vcd_file() {
        let mut app = App::with_config(config::load_config(None).unwrap());

        // Load the VCD file
        let result = app.load_vcd_file(&create_test_vcd_file());
        assert!(result.is_ok());

        // Check the loaded data
        assert_eq!(app.state.waveform_data.signals.len(), 3);
        assert_eq!(app.state.waveform_data.max_time, 20);

        // Check the view is correctly set
        assert_eq!(app.state.time_start, 0);
        assert_eq!(app.state.time_range, 20);
    }

    #[test]
    fn test_app_mode_transitions() {
        let mut app = App::with_config(config::load_config(None).unwrap());

        // Initial mode should be Normal
        assert_eq!(app.state.mode, AppMode::Normal);

        // Test entering command mode
        app.handle_input(KeyEvent::new(
            app.state.config.keybindings.enter_command_mode,
            KeyModifiers::empty(),
        ));
        assert_eq!(app.state.mode, AppMode::Command);

        // Test exiting command mode
        app.handle_command_input(KeyEvent::new(
            app.state.config.keybindings.enter_normal_mode,
            KeyModifiers::empty(),
        ));
        assert_eq!(app.state.mode, AppMode::Normal);
    }
}
