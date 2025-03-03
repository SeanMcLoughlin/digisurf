use crate::{
    command_mode::{builder::CommandBuilder, registry::Command},
    state::AppState,
};
use std::rc::Rc;

pub fn create() -> Rc<Box<dyn Command<AppState>>> {
    CommandBuilder::new(
        "zoom",
        "Zoom to a specific level",
        |args, state: &mut AppState| {
            if args.is_empty() {
                return Err("Usage: zoom <factor>".to_string());
            }

            if let Ok(factor) = args[0].parse::<u64>() {
                if factor > 0 {
                    let center = state.time_start + (state.time_range / 2);
                    let new_range = state.max_time / factor;

                    // Calculate new start time based on center point
                    let half_new_range = new_range / 2;
                    let new_start = if center > half_new_range {
                        center - half_new_range
                    } else {
                        0
                    };

                    state.time_start = new_start;
                    state.time_range = new_range;
                    return Ok(format!("Zoomed to 1/{}", factor));
                }
            }
            Err("Invalid zoom factor".to_string())
        },
    )
    .build()
}
