use crate::{
    command_mode::{builder::CommandBuilder, registry::Command},
    state::AppState,
};
use std::rc::Rc;

pub fn create() -> Rc<Box<dyn Command<AppState>>> {
    CommandBuilder::new(
        "zoomfull",
        "Zoom to show the full waveform",
        |_args, state: &mut AppState| {
            state.time_start = 0;
            state.time_range = state.waveform_data.max_time;
            Ok("Zoomed to full view".to_string())
        },
    )
    .alias("zf")
    .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_state() -> AppState {
        let mut state = AppState::default();
        state.time_start = 100;
        state.time_range = 50;
        state.waveform_data.max_time = 1000;
        state
    }

    #[test]
    fn test_zoomfull() {
        let command = create();
        let mut state = get_state();
        let result = command.execute(&[], &mut state);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Zoomed to full view".to_string());
        assert_eq!(state.time_start, 0);
        assert_eq!(state.time_range, 1000);
    }
}
