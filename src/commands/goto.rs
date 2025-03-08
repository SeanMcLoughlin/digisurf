use crate::{
    command_mode::{builder::CommandBuilder, registry::Command},
    state::AppState,
};
use std::rc::Rc;

pub fn create() -> Rc<Box<dyn Command<AppState>>> {
    CommandBuilder::new(
        "goto",
        "Move to a specific time",
        |args, state: &mut AppState| {
            if args.is_empty() {
                return Err("Usage: goto <time>".to_string());
            }

            if let Ok(time) = args[0].parse::<u64>() {
                if time <= state.waveform_data.max_time {
                    // Center the view around the time point
                    let half_range = state.time_range / 2;
                    state.time_start = if time > half_range {
                        time - half_range
                    } else {
                        0
                    };
                    return Ok(format!("Moved to time {}", time));
                }
                return Err(format!(
                    "Time out of range (0-{})",
                    state.waveform_data.max_time
                ));
            }
            Err("Invalid time format".to_string())
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
    fn test_goto_empty_args() {
        let command = create();
        let mut state = get_state();
        let result = command.execute(&[], &mut state);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Usage: goto <time>".to_string());
    }

    #[test]
    fn test_goto_invalid_time() {
        let command = create();
        let mut state = get_state();
        let result = command.execute(&["not_a_number"], &mut state);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid time format".to_string());
    }

    #[test]
    fn test_goto_time_out_of_range() {
        let command = create();
        let mut state = get_state();
        let result = command.execute(&[&"2000"], &mut state);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Time out of range (0-1000)".to_string()
        );
    }

    #[test]
    fn test_goto_valid_time_with_centering() {
        let command = create();
        let mut state = get_state();
        let result = command.execute(&["500"], &mut state);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Moved to time 500".to_string());
        assert_eq!(state.time_start, 450); // 500 - (100/2)
    }

    #[test]
    fn test_goto_near_beginning() {
        let command = create();
        let mut state = get_state();
        let result = command.execute(&["30"], &mut state);
        assert!(result.is_ok());
        assert_eq!(state.time_start, 0); // Should clamp to 0 when time < half_range
    }
}
