use crate::{command_mode::builder::CommandBuilder, state::AppState};
use std::rc::Rc;

pub fn create() -> Rc<Box<dyn crate::command_mode::registry::Command<AppState>>> {
    CommandBuilder::new(
        "goto",
        "Move to a specific time",
        |args, state: &mut AppState| {
            if args.is_empty() {
                return Err("Usage: goto <time>".to_string());
            }

            if let Ok(time) = args[0].parse::<u64>() {
                if time <= state.max_time {
                    // Center the view around the time point
                    let half_range = state.time_range / 2;
                    state.time_start = if time > half_range {
                        time - half_range
                    } else {
                        0
                    };
                    return Ok(format!("Moved to time {}", time));
                }
                return Err(format!("Time out of range (0-{})", state.max_time));
            }
            Err("Invalid time format".to_string())
        },
    )
    .build()
}
