use crate::{
    constants::DRAG_COLOR,
    parsers::types::{Value, WaveValue},
    state::AppState,
};
use ratatui::{
    layout::Rect,
    prelude::Buffer,
    style::{Color, Style},
    widgets::{
        canvas::{Canvas, Line},
        Block, Paragraph, StatefulWidget, Widget,
    },
};

#[derive(Default, Copy, Clone)]
pub struct WaveformWidget {}

impl WaveformWidget {
    fn draw_binary_signal(
        &self,
        buf: &mut Buffer,
        area: Rect,
        values: &[(u64, WaveValue)],
        time_offset: u64,
        window_size: u64,
        style: Style,
    ) {
        let canvas = Canvas::default()
            .block(Block::default())
            .x_bounds([0.0, area.width as f64])
            .y_bounds([0.0, 2.0])
            .paint(|ctx| {
                let width = area.width as f64;
                let mut last_value: Option<(f64, f64, Color)> = None;

                let time_to_x = |t: u64| -> f64 {
                    // Round to get precise pixel alignment
                    ((t - time_offset) as f64 / window_size as f64 * width).round()
                };

                for (t, v) in values {
                    let x = time_to_x(*t);
                    let (y, color) = match v {
                        WaveValue::Binary(Value::V1) => (1.5, style.fg.unwrap_or(Color::White)),
                        WaveValue::Binary(Value::V0) => (0.5, style.fg.unwrap_or(Color::White)),
                        WaveValue::Binary(Value::VZ) => (1.0, Color::Magenta),
                        WaveValue::Binary(Value::VX) => (1.0, Color::Red),
                        _ => (1.0, style.fg.unwrap_or(Color::White)),
                    };

                    if let Some((prev_y, prev_x, prev_color)) = last_value {
                        // Draw horizontal line from last position
                        ctx.draw(&Line {
                            x1: prev_x,
                            y1: prev_y,
                            x2: x,
                            y2: prev_y,
                            color: prev_color,
                        });

                        // If value changed, draw vertical transition
                        if prev_y != y {
                            ctx.draw(&Line {
                                x1: x,
                                y1: prev_y,
                                x2: x,
                                y2: y,
                                color,
                            });
                        }
                    }

                    last_value = Some((y, x, color));
                }

                // Draw remaining horizontal line to the end
                if let Some((y, x, color)) = last_value {
                    ctx.draw(&Line {
                        x1: x,
                        y1: y,
                        x2: width,
                        y2: y,
                        color,
                    });
                }
            });

        canvas.render(area, buf);
    }

    fn draw_bus_signal(
        &self,
        buf: &mut Buffer,
        area: Rect,
        values: &[(u64, WaveValue)],
        time_start: u64,
        time_range: u64,
        style: Style,
    ) {
        // Calculate transition points
        let width = area.width as f64;
        let mut transition_points = Vec::new();

        // Use the same time_to_x conversion as in draw_binary_signal
        let time_to_x = |t: u64| -> u16 {
            ((t - time_start) as f64 / time_range as f64 * width).round() as u16
        };

        for (_, (t, _)) in values.iter().enumerate() {
            // Convert the time directly using the window
            let x_pos = time_to_x(*t);
            transition_points.push(x_pos);
        }
        transition_points.push(area.width);

        // Draw the bus signal with a straight line
        let canvas = Canvas::default()
            .block(Block::default())
            .x_bounds([0.0, width])
            .y_bounds([0.0, 2.0])
            .paint(|ctx| {
                // Draw a straight line in the middle
                let y_middle = 1.0;
                ctx.draw(&Line {
                    x1: 0.0,
                    y1: y_middle,
                    x2: width,
                    y2: y_middle,
                    color: style.fg.unwrap_or(Color::White),
                });

                // Draw vertical transitions at change points
                for i in 0..transition_points.len() - 1 {
                    let x = transition_points[i] as f64;

                    // Draw transition line
                    ctx.draw(&Line {
                        x1: x,
                        y1: 0.5,
                        x2: x,
                        y2: 1.5,
                        color: style.fg.unwrap_or(Color::White),
                    });
                }
            });

        canvas.render(area, buf);

        // Draw bus value labels in the middle of segments
        for (i, (_, v)) in values.iter().enumerate() {
            if let WaveValue::Bus(value) = v {
                // Calculate midpoint between transitions
                let start_x = transition_points[i];
                let end_x = transition_points[i + 1];
                let segment_width = end_x.saturating_sub(start_x);
                let value_len = value.len() as u16;

                // Only draw if there's enough space
                if segment_width > value_len {
                    let midpoint = start_x + (segment_width - value_len) / 2;
                    let label_area = Rect::new(area.x + midpoint, area.y, value_len, 1);

                    Paragraph::new(value.clone())
                        .style(style)
                        .render(label_area, buf);
                }
            }
        }
    }

    pub fn draw_signals(&self, buf: &mut Buffer, area: Rect, state: &AppState) {
        let waveform_height = 2;
        let time_start = state.time_start;
        let time_range = state.time_range;

        for (idx, signal_name) in state.displayed_signals.iter().enumerate() {
            let signal_area = Rect::new(
                area.x,
                area.y + (idx as u16 * waveform_height),
                area.width,
                waveform_height,
            );

            if signal_area.y >= area.bottom() {
                continue; // Don't render signals outside of visible area
            }

            let is_selected = idx == state.selected_signal;
            let style = if is_selected {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::White)
            };

            // Get visible values for this signal
            let visible_values = state.get_visible_values(signal_name);

            if visible_values.is_empty() {
                continue;
            }

            // Determine if this is a bus or binary signal
            let is_bus = visible_values
                .iter()
                .any(|(_, v)| matches!(v, WaveValue::Bus(_)));

            if is_bus {
                self.draw_bus_signal(
                    buf,
                    signal_area,
                    &visible_values,
                    time_start,
                    time_range,
                    style,
                );
            } else {
                self.draw_binary_signal(
                    buf,
                    signal_area,
                    &visible_values,
                    time_start,
                    time_range,
                    style,
                );
            }
        }
    }

    pub fn draw_markers(&self, buf: &mut Buffer, area: Rect, state: &AppState) {
        let time_start = state.time_start;
        let time_range = state.time_range;
        let width = area.width as f64;

        // Draw primary marker if exists and is visible
        if let Some(marker_time) = state.primary_marker {
            if self.is_marker_visible(marker_time, time_start, time_range) {
                // Calculate x position
                let x_ratio = (marker_time - time_start) as f64 / time_range as f64;
                let x_pos = (x_ratio * width).round() as u16;

                // Draw vertical line
                let marker_line = Canvas::default()
                    .block(Block::default())
                    .x_bounds([0.0, width])
                    .y_bounds([0.0, area.height as f64])
                    .paint(|ctx| {
                        ctx.draw(&Line {
                            x1: x_pos as f64,
                            y1: 0.0,
                            x2: x_pos as f64,
                            y2: area.height as f64,
                            color: Color::Yellow,
                        });
                    });

                marker_line.render(area, buf);
            }
        }

        // Draw secondary marker if exists and is visible
        if let Some(marker_time) = state.secondary_marker {
            if self.is_marker_visible(marker_time, time_start, time_range) {
                // Calculate x position
                let x_ratio = (marker_time - time_start) as f64 / time_range as f64;
                let x_pos = (x_ratio * width).round() as u16;

                // Draw vertical line
                let marker_line = Canvas::default()
                    .block(Block::default())
                    .x_bounds([0.0, width])
                    .y_bounds([0.0, area.height as f64])
                    .paint(|ctx| {
                        ctx.draw(&Line {
                            x1: x_pos as f64,
                            y1: 0.0,
                            x2: x_pos as f64,
                            y2: area.height as f64,
                            color: Color::White,
                        });
                    });

                marker_line.render(area, buf);
            }
        }
    }

    pub fn draw_drag_selection(&self, buf: &mut Buffer, area: Rect, state: &AppState) {
        if state.is_dragging {
            if let (Some((start_x, _)), Some((current_x, _))) =
                (state.drag_start, state.drag_current)
            {
                // Determine selection box coordinates
                let left = start_x.min(current_x);
                let right = start_x.max(current_x);
                let width = right - left;

                // Draw selection box if the user has dragged a minimum distance
                if width > 1 {
                    let selection_area = Rect::new(area.x + left, area.y, width, area.height);

                    // Draw a semi-transparent selection box
                    for y in selection_area.top()..selection_area.bottom() {
                        for x in selection_area.left()..selection_area.right() {
                            buf[(x, y)].set_bg(DRAG_COLOR);
                        }
                    }
                }
            }
        }
    }

    fn is_marker_visible(&self, marker_time: u64, time_start: u64, time_range: u64) -> bool {
        marker_time >= time_start && marker_time <= time_start + time_range
    }
}

impl StatefulWidget for WaveformWidget {
    type State = AppState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if state.waveform_data.signals.is_empty() {
            return;
        }

        self.draw_signals(buf, area, &state);
        self.draw_markers(buf, area, &state);
        self.draw_drag_selection(buf, area, &state);
    }
}

// Add to digisurf/src/ui/widgets/waveform.rs

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        parsers::types::{Value, WaveValue},
        state::AppState,
    };
    use insta::assert_snapshot;
    use ratatui::{backend::TestBackend, Terminal};
    use std::collections::HashMap;

    fn create_test_state() -> AppState {
        let mut state = AppState::new();

        // Add test waveform data
        state.waveform_data.signals = vec!["sig1".to_string(), "sig2".to_string()];
        state.displayed_signals = vec!["sig1".to_string(), "sig2".to_string()];

        // Add signal values
        let mut values = HashMap::new();
        values.insert(
            "sig1".to_string(),
            vec![
                (0, WaveValue::Binary(Value::V0)),
                (10, WaveValue::Binary(Value::V1)),
                (20, WaveValue::Binary(Value::V0)),
            ],
        );
        values.insert(
            "sig2".to_string(),
            vec![
                (0, WaveValue::Bus("00".to_string())),
                (15, WaveValue::Bus("FF".to_string())),
            ],
        );

        state.waveform_data.values = values;
        state.waveform_data.max_time = 50;

        state
    }

    #[test]
    fn test_render_waveform() {
        let mut state = create_test_state();
        state.time_start = 0;
        state.time_range = 50;

        let widget = WaveformWidget::default();

        // Create a test terminal
        let backend = TestBackend::new(80, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        // Render the widget
        terminal
            .draw(|f| {
                let size = f.area();
                widget.render(size, &mut f.buffer_mut(), &mut state);
            })
            .unwrap();

        // Verify the widget rendered something to the buffer
        let buffer = terminal.backend().buffer();
        assert!(buffer.content().len() > 0);
        assert_snapshot!(terminal.backend());
    }

    #[test]
    fn test_render_markers() {
        let mut state = create_test_state();
        state.time_start = 0;
        state.time_range = 50;

        // Set markers
        state.primary_marker = Some(10);
        state.secondary_marker = Some(20);

        let widget = WaveformWidget::default();

        // Create a test terminal
        let backend = TestBackend::new(80, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        // Render the widget
        terminal
            .draw(|f| {
                let size = f.area();
                widget.render(size, &mut f.buffer_mut(), &mut state);
            })
            .unwrap();

        assert_snapshot!(terminal.backend());
    }
}
