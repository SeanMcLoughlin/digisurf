use crate::{app::App, model::types::WaveValue};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{
        canvas::{Canvas, Line},
        Block, Paragraph,
    },
};

pub fn draw_waveforms(frame: &mut ratatui::Frame<'_>, area: Rect, app: &App) {
    if app.signals.is_empty() {
        return;
    }

    let waveform_height = 2;
    let time_start = app.waveform.time_start;
    let time_range = app.waveform.time_range;

    for (i, signal_name) in app.signals.iter().enumerate() {
        let signal_area = Rect::new(
            area.x,
            area.y + (i as u16 * waveform_height),
            area.width,
            waveform_height,
        );

        if signal_area.y >= area.bottom() {
            break; // Don't render signals outside of visible area
        }

        let is_selected = i == app.selected_signal;
        let style = if is_selected {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };

        // Get visible values for this signal
        let visible_values = app.get_visible_values(signal_name);

        if visible_values.is_empty() {
            continue;
        }

        // Determine if this is a bus or binary signal
        let is_bus = visible_values
            .iter()
            .any(|(_, v)| matches!(v, WaveValue::Bus(_)));

        if is_bus {
            draw_bus_signal(
                frame,
                signal_area,
                &visible_values,
                time_start,
                time_range,
                style,
            );
        } else {
            draw_binary_signal(
                frame,
                signal_area,
                &visible_values,
                time_start,
                time_range,
                style,
            );
        }
    }

    draw_markers(frame, area, app);
}

fn draw_markers(frame: &mut ratatui::Frame<'_>, area: Rect, app: &App) {
    let time_start = app.waveform.time_start;
    let time_range = app.waveform.time_range;
    let width = area.width as f64;

    // Draw primary marker. Re-convert time position to x position within the waveform Rect
    if let Some(marker_time) = app.waveform.primary_marker {
        if marker_is_visible_at_current_zoom_level(marker_time, time_start, time_range) {
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

            frame.render_widget(marker_line, area);
        }
    }

    // Draw secondary marker. Re-convert time position to x position within the waveform Rect
    if let Some(marker_time) = app.waveform.secondary_marker {
        if marker_is_visible_at_current_zoom_level(marker_time, time_start, time_range) {
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

            frame.render_widget(marker_line, area);
        }
    }
}

fn marker_is_visible_at_current_zoom_level(
    marker_time: u64,
    time_start: u64,
    time_range: u64,
) -> bool {
    marker_time >= time_start && marker_time <= time_start + time_range
}

fn draw_binary_signal(
    frame: &mut ratatui::Frame<'_>,
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
                    WaveValue::Binary(vcd::Value::V1) => (1.5, style.fg.unwrap_or(Color::White)),
                    WaveValue::Binary(vcd::Value::V0) => (0.5, style.fg.unwrap_or(Color::White)),
                    WaveValue::Binary(vcd::Value::Z) => (1.0, Color::Magenta),
                    WaveValue::Binary(vcd::Value::X) => (1.0, Color::Red),
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

    frame.render_widget(canvas, area);
}

fn draw_bus_signal(
    frame: &mut ratatui::Frame<'_>,
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
    let time_to_x =
        |t: u64| -> u16 { ((t - time_start) as f64 / time_range as f64 * width).round() as u16 };

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

    frame.render_widget(canvas, area);

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

                frame.render_widget(Paragraph::new(value.clone()).style(style), label_area);
            }
        }
    }
}
