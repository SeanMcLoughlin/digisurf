use crate::{
    command_mode::{builder::CommandBuilder, registry::Command},
    state::AppState,
};
use std::rc::Rc;

pub fn create() -> Rc<Box<dyn Command<AppState>>> {
    CommandBuilder::new(
        "marker",
        "Set a marker at a specific time",
        |args, state: &mut AppState| {
            if args.len() < 2 {
                return Err("Usage: marker <number> <time>".to_string());
            }

            if let Ok(marker_num) = args[0].parse::<u8>() {
                if let Ok(time) = args[1].parse::<u64>() {
                    if time <= state.waveform_data.max_time {
                        match marker_num {
                            1 => {
                                state.primary_marker = Some(time);
                                return Ok(format!("Set marker 1 to time {}", time));
                            }
                            2 => {
                                state.secondary_marker = Some(time);
                                return Ok(format!("Set marker 2 to time {}", time));
                            }
                            _ => return Err("Invalid marker number (use 1 or 2)".to_string()),
                        }
                    }
                    return Err(format!(
                        "Time out of range (0-{})",
                        state.waveform_data.max_time
                    ));
                }
                return Err("Invalid time format".to_string());
            }
            Err("Invalid marker number format".to_string())
        },
    )
    .alias("m")
    .build()
}
