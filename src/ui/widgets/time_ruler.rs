// digisurf/src/ui/widgets/time_ruler.rs
use crate::state::AppState;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::StatefulWidget,
};

#[derive(Default, Copy, Clone)]
pub struct TimeRulerWidget {}

impl StatefulWidget for TimeRulerWidget {
    type State = AppState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if state.waveform_data.max_time == 0 {
            return;
        }

        // Draw the ruler background
        for x in area.left()..area.right() {
            for y in area.top()..area.bottom() {
                buf[(x, y)].set_bg(Color::DarkGray);
            }
        }

        // Calculate appropriate time intervals based on current zoom level
        let time_span = state.time_range;
        let width = area.width as u64;
        let tick_interval = calculate_tick_interval(time_span, width);

        // Draw ticks and labels
        let mut time = (state.time_start / tick_interval) * tick_interval;
        if time < state.time_start {
            time += tick_interval;
        }

        while time <= state.time_start + time_span {
            let x_pos = time_to_x(time, state.time_start, time_span, area.width);
            if x_pos < area.width {
                // Draw tick
                let tick_style = Style::default().fg(Color::Yellow);
                if area.height >= 1 {
                    buf[(area.x + x_pos, area.y + area.height - 1)].set_style(tick_style);
                }
                if area.height >= 2 {
                    buf[(area.x + x_pos, area.y + area.height - 2)].set_style(tick_style);
                }

                // Draw time label
                let label = format!("{}", time);
                let label_start = x_pos.saturating_sub(label.len() as u16 / 2);
                for (i, c) in label.chars().enumerate() {
                    let x = area.x + label_start + i as u16;
                    if x < area.right() {
                        buf[(x, area.y)].set_char(c).set_style(tick_style);
                    }
                }
            }
            time += tick_interval;
        }
    }
}

// Helper function to convert time to x position
fn time_to_x(time: u64, time_start: u64, time_span: u64, width: u16) -> u16 {
    ((time.saturating_sub(time_start)) as f64 / time_span as f64 * width as f64) as u16
}

// Helper function to calculate appropriate tick intervals
fn calculate_tick_interval(time_span: u64, width: u64) -> u64 {
    // Target roughly 5-10 ticks across the visible width
    let target_num_ticks = width / 10;
    let approx_interval = if target_num_ticks > 0 {
        time_span / target_num_ticks
    } else {
        time_span
    };

    // Round to a nice number (1, 2, 5, 10, 20, 50, 100, etc.)
    let magnitude = if approx_interval > 0 {
        10_u64.pow((approx_interval as f64).log10().floor() as u32)
    } else {
        1
    };

    let normalized = if magnitude > 0 {
        approx_interval as f64 / magnitude as f64
    } else {
        1.0
    };

    if normalized < 1.5 {
        magnitude.max(1)
    } else if normalized < 3.5 {
        (2 * magnitude).max(1)
    } else if normalized < 7.5 {
        (5 * magnitude).max(1)
    } else {
        (10 * magnitude).max(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_to_x() {
        let time_start = 10;
        let time_span = 100;
        let width = 50;

        assert_eq!(time_to_x(time_start, time_start, time_span, width), 0);
        assert_eq!(
            time_to_x(time_start + time_span / 2, time_start, time_span, width),
            width / 2
        );
        assert_eq!(
            time_to_x(time_start + time_span, time_start, time_span, width),
            width
        );
    }

    #[test]
    fn test_calculate_tick_interval() {
        let time_span = 100;
        let width = 50;

        assert_ne!(calculate_tick_interval(time_span, width), 0);
    }
}
