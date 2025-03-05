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
