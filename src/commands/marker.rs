use crate::{
    command_mode::{builder::CommandBuilder, registry::Command},
    state::AppState,
    types::Marker,
};
use std::rc::Rc;

pub fn create() -> Rc<Box<dyn Command<AppState>>> {
    CommandBuilder::new(
        "marker",
        "Add or remove saved markers with names",
        |args, state: &mut AppState| {
            if args.is_empty() {
                return Err("Usage: marker add <name> [time] or marker remove <name>".to_string());
            }

            let subcommand = &args[0];
            match &**subcommand {
                "add" | "a" => {
                    if args.len() < 2 {
                        return Err(format!("Usage: marker {} <name> [time]", &**subcommand));
                    }

                    let name = args[1];

                    // If time was provided, use it. Otherwise, use the primary marker.
                    let time = if args.len() >= 3 {
                        match args[2].parse::<u64>() {
                            Ok(t) => {
                                if t > state.waveform_data.max_time {
                                    return Err(format!(
                                        "Time out of range (0-{})",
                                        state.waveform_data.max_time
                                    ));
                                }
                                t
                            }
                            Err(_) => return Err("Invalid time format".to_string()),
                        }
                    } else {
                        match state.primary_marker {
                            Some(t) => t,
                            None => {
                                return Err(
                                    "No time specified and primary marker not set".to_string()
                                )
                            }
                        }
                    };

                    // Create and add marker
                    let marker = Marker {
                        time,
                        name: name.to_string(),
                    };
                    state.saved_markers.push(marker);
                    Ok(format!("Added marker '{}' at time {}", name, time))
                }
                "remove" | "rm" => {
                    if args.len() < 2 {
                        return Err(format!("Usage: marker {} <name>", &**subcommand));
                    }

                    let name = &args[1];
                    if let Some(index) = state.saved_markers.iter().position(|m| &m.name == name) {
                        let marker = state.saved_markers.remove(index);
                        Ok(format!(
                            "Removed marker '{}' at time {}",
                            marker.name, marker.time
                        ))
                    } else {
                        Err(format!("No marker found with name '{}'", name))
                    }
                }
                _ => Err("Unknown subcommand. Use 'add' or 'remove'".to_string()),
            }
        },
    )
    .alias("m")
    .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Marker;

    fn get_state() -> AppState {
        let mut state = AppState::default();
        state.waveform_data.max_time = 1000;
        state
    }

    #[test]
    fn test_marker_no_args_is_err() {
        let command = create();
        let mut state = get_state();
        let result = command.execute(&[], &mut state);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Usage: marker add <name> [time] or marker remove <name>".to_string()
        );
    }

    #[test]
    fn test_marker_invalid_subcommand_is_err() {
        let command = create();
        let mut state = get_state();
        let result = command.execute(&["invalid", "mymarker"], &mut state);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Unknown subcommand. Use 'add' or 'remove'".to_string()
        );
    }

    #[test]
    fn test_marker_add_missing_name_is_err() {
        let command = create();
        let mut state = get_state();
        let result = command.execute(&["add"], &mut state);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Usage: marker add <name> [time]".to_string()
        );
    }

    #[test]
    fn test_marker_remove_missing_name_is_err() {
        let command = create();
        let mut state = get_state();
        let result = command.execute(&["remove"], &mut state);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Usage: marker remove <name>".to_string()
        );
    }

    #[test]
    fn test_marker_add_invalid_time_is_err() {
        let command = create();
        let mut state = get_state();
        let result = command.execute(&["add", "mymarker", "not_a_number"], &mut state);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid time format".to_string());
    }

    #[test]
    fn test_marker_add_time_out_of_range_is_err() {
        let command = create();
        let mut state = get_state();
        let result = command.execute(&["add", "mymarker", "2000"], &mut state);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Time out of range (0-1000)".to_string()
        );
    }

    #[test]
    fn test_marker_add_success_with_explicit_time() {
        let command = create();
        let mut state = get_state();
        let result = command.execute(&["add", "mymarker", "500"], &mut state);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            "Added marker 'mymarker' at time 500".to_string()
        );

        let marker = state
            .saved_markers
            .iter()
            .find(|m| m.name == "mymarker")
            .unwrap();
        assert_eq!(marker.time, 500);
    }

    #[test]
    fn test_marker_add_success_with_primary_marker() {
        let command = create();
        let mut state = get_state();
        state.primary_marker = Some(300);

        let result = command.execute(&["add", "mymarker"], &mut state);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            "Added marker 'mymarker' at time 300".to_string()
        );

        let marker = state
            .saved_markers
            .iter()
            .find(|m| m.name == "mymarker")
            .unwrap();
        assert_eq!(marker.time, 300);
    }

    #[test]
    fn test_marker_add_no_primary_marker_is_err() {
        let command = create();
        let mut state = get_state();
        state.primary_marker = None;

        let result = command.execute(&["add", "mymarker"], &mut state);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "No time specified and primary marker not set".to_string()
        );
    }

    #[test]
    fn test_marker_remove_nonexistent_is_err() {
        let command = create();
        let mut state = get_state();
        let result = command.execute(&["remove", "mymarker"], &mut state);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "No marker found with name 'mymarker'".to_string()
        );
    }

    #[test]
    fn test_marker_remove_success() {
        let command = create();
        let mut state = get_state();
        state.saved_markers.push(Marker {
            time: 500,
            name: "mymarker".to_string(),
        });

        let result = command.execute(&["remove", "mymarker"], &mut state);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            "Removed marker 'mymarker' at time 500".to_string()
        );
        assert!(state.saved_markers.is_empty());
    }
}
