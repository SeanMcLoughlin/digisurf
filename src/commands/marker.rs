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

#[cfg(test)]
mod tests {
    use super::*;

    fn get_state() -> AppState {
        let mut state = AppState::default();
        state.waveform_data.max_time = 1000;
        state
    }

    #[test]
    fn test_marker_too_few_args_is_err() {
        let command = create();
        let mut state = get_state();
        let result = command.execute(&["1"], &mut state);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Usage: marker <number> <time>".to_string()
        );
    }

    #[test]
    fn test_marker_invalid_marker_number_is_err() {
        let command = create();
        let mut state = get_state();
        let result = command.execute(&["not_a_number", "500"], &mut state);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Invalid marker number format".to_string()
        );
    }

    #[test]
    fn test_marker_invalid_time_is_err() {
        let command = create();
        let mut state = get_state();
        let result = command.execute(&["1", "not_a_number"], &mut state);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid time format".to_string());
    }

    #[test]
    fn test_marker_time_out_of_range_is_err() {
        let command = create();
        let mut state = get_state();
        let result = command.execute(&["1", "2000"], &mut state);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Time out of range (0-1000)".to_string()
        );
    }

    #[test]
    fn test_marker_invalid_marker_number_range_is_err() {
        let command = create();
        let mut state = get_state();
        let result = command.execute(&["3", "500"], &mut state);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Invalid marker number (use 1 or 2)".to_string()
        );
    }

    #[test]
    fn test_marker_set_primary() {
        let command = create();
        let mut state = get_state();
        let result = command.execute(&["1", "500"], &mut state);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Set marker 1 to time 500".to_string());
        assert_eq!(state.primary_marker, Some(500));
    }

    #[test]
    fn test_marker_set_secondary() {
        let command = create();
        let mut state = get_state();
        let result = command.execute(&["2", "750"], &mut state);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Set marker 2 to time 750".to_string());
        assert_eq!(state.secondary_marker, Some(750));
    }
}
