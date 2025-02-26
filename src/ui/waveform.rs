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
    let time_offset = app.time_offset;
    let window_size = app.window_size;

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
            draw_bus_signal(frame, signal_area, &visible_values, style);
        } else {
            draw_binary_signal(
                frame,
                signal_area,
                &visible_values,
                time_offset,
                window_size,
                style,
            );
        }
    }
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

            let time_to_x =
                |t: u64| -> f64 { ((t - time_offset) as f64 / window_size as f64) * width };

            for (t, v) in values {
                let x = time_to_x(*t);
                let (y, color) = match v {
                    WaveValue::Binary(vcd::Value::V1) => (0.5, style.fg.unwrap_or(Color::White)),
                    WaveValue::Binary(vcd::Value::V0) => (1.5, style.fg.unwrap_or(Color::White)),
                    WaveValue::Binary(vcd::Value::Z) => (1.0, Color::Magenta), // FIXME: I want orange.
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
                            color: color,
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
                    color: color,
                });
            }
        });

    frame.render_widget(canvas, area);
}

fn draw_bus_signal(
    frame: &mut ratatui::Frame<'_>,
    area: Rect,
    values: &[(u64, WaveValue)],
    style: Style,
) {
    // Determine if we're too zoomed out to show details
    let transitions_too_dense = if values.len() >= 2 {
        // Check if average space between transitions is too small
        let avg_space_per_transition = area.width as f64 / values.len() as f64;
        avg_space_per_transition < 3.0 // If less than 3 pixels between transitions
    } else {
        false
    };

    // Draw the bus signal with a straight line
    let canvas = Canvas::default()
        .block(Block::default())
        .x_bounds([0.0, area.width as f64])
        .y_bounds([0.0, 2.0])
        .paint(|ctx| {
            let width = area.width as f64;

            if transitions_too_dense {
                // Draw zigzag pattern for zoomed out bus signal
                let zigzag_width = 3.0;
                let zigzag_height = 0.3;
                let y_middle = 1.0;
                let mut x = 0.0;

                while x < width {
                    let x_end = (x + zigzag_width).min(width);
                    ctx.draw(&Line {
                        x1: x,
                        y1: y_middle - zigzag_height,
                        x2: x + zigzag_width / 2.0,
                        y2: y_middle + zigzag_height,
                        color: style.fg.unwrap_or(Color::White),
                    });

                    if x_end < width {
                        ctx.draw(&Line {
                            x1: x + zigzag_width / 2.0,
                            y1: y_middle + zigzag_height,
                            x2: x_end,
                            y2: y_middle - zigzag_height,
                            color: style.fg.unwrap_or(Color::White),
                        });
                    }

                    x += zigzag_width;
                }
            } else {
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
                let mut prev_x = None;

                for (i, (t, _)) in values.iter().enumerate() {
                    // Removed the "if i == 0 { continue; }" to draw the first transition too
                    let x = if i == 0 {
                        0.0 // First value starts at the beginning
                    } else {
                        ((*t - values[0].0) as f64
                            / (values.last().unwrap().0 - values[0].0) as f64)
                            * width
                    };

                    if let Some(px) = prev_x {
                        if x > px + 2.0 {
                            // Avoid drawing transitions too close
                            ctx.draw(&Line {
                                x1: x,
                                y1: 0.5,
                                x2: x,
                                y2: 1.5,
                                color: style.fg.unwrap_or(Color::White),
                            });
                        }
                    } else {
                        // Draw transition for the first value
                        ctx.draw(&Line {
                            x1: x,
                            y1: 0.5,
                            x2: x,
                            y2: 1.5,
                            color: style.fg.unwrap_or(Color::White),
                        });
                    }

                    prev_x = Some(x);
                }
            }
        });

    frame.render_widget(canvas, area);

    // Only show values if not too zoomed out
    if transitions_too_dense {
        // Show a "zoomed out" indicator
        let msg = "[zoomed out]";
        let midpoint = (area.width - msg.len() as u16) / 2;
        let label_area = Rect::new(area.x + midpoint, area.y, msg.len() as u16, 1);
        frame.render_widget(Paragraph::new(msg).style(style), label_area);
        return;
    }

    // Calculate transition points
    let mut transition_points = Vec::new();
    for (i, (t, _)) in values.iter().enumerate() {
        if i == 0 {
            transition_points.push(0);
            continue;
        }

        let x_ratio =
            (*t - values[0].0) as f64 / (values.last().unwrap().0 - values[0].0).max(1) as f64;
        let x_pos = (x_ratio * area.width as f64) as u16;
        transition_points.push(x_pos);
    }
    transition_points.push(area.width);

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
