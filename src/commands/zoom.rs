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
                    let new_range = state.waveform_data.max_time / factor;

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

#[cfg(test)]
mod tests {
    use super::*;

    fn get_state() -> AppState {
        let mut state = AppState::default();
        state.time_start = 0;
        state.time_range = 100;
        state.waveform_data.max_time = 1000;
        state
    }

    #[test]
    fn test_zoom_empty_args_is_err() {
        let command = create();
        let mut state = get_state();
        let result = command.execute(&[], &mut state);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Usage: zoom <factor>".to_string());
    }

    #[test]
    fn test_zoom_invalid_factor_is_err() {
        let command = create();
        let mut state = get_state();
        let result = command.execute(&["not_a_number"], &mut state);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid zoom factor".to_string());
    }

    #[test]
    fn test_zoom_zero_factor_is_err() {
        let command = create();
        let mut state = get_state();
        let result = command.execute(&["0"], &mut state);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid zoom factor".to_string());
    }

    #[test]
    fn test_zoom_valid_factor() {
        let command = create();
        let mut state = get_state();
        state.time_start = 200;
        state.time_range = 200;
        let result = command.execute(&["2"], &mut state);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Zoomed to 1/2".to_string());
        assert_eq!(state.time_range, 500); // 1000 / 2
        assert_eq!(state.time_start, 50); // center is 300, half_new_range is 250, so 300-250=50
    }

    #[test]
    fn test_zoom_near_beginning() {
        let command = create();
        let mut state = get_state();
        state.time_start = 0;
        state.time_range = 100;
        let result = command.execute(&["4"], &mut state);
        assert!(result.is_ok());
        assert_eq!(state.time_range, 250); // 1000 / 4
        assert_eq!(state.time_start, 0); // Should clamp to 0 when center < half_new_range
    }
}
