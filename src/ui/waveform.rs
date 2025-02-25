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
            let mut last_value: Option<(f64, f64)> = None;

            let time_to_x =
                |t: u64| -> f64 { ((t - time_offset) as f64 / window_size as f64) * width };

            for (t, v) in values {
                let x = time_to_x(*t);
                let y = match v {
                    WaveValue::Binary(vcd::Value::V1) => 0.5,
                    WaveValue::Binary(vcd::Value::V0) => 1.5,
                    _ => 1.0, // Middle for undefined values
                };

                if let Some((prev_y, prev_x)) = last_value {
                    // Draw horizontal line from last position
                    ctx.draw(&Line {
                        x1: prev_x,
                        y1: prev_y,
                        x2: x,
                        y2: prev_y,
                        color: style.fg.unwrap_or(Color::White),
                    });

                    // If value changed, draw vertical transition
                    if prev_y != y {
                        ctx.draw(&Line {
                            x1: x,
                            y1: prev_y,
                            x2: x,
                            y2: y,
                            color: style.fg.unwrap_or(Color::White),
                        });
                    }
                }

                last_value = Some((y, x));
            }

            // Draw remaining horizontal line to the end
            if let Some((y, x)) = last_value {
                ctx.draw(&Line {
                    x1: x,
                    y1: y,
                    x2: width,
                    y2: y,
                    color: style.fg.unwrap_or(Color::White),
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
    // Draw the bus signal baseline
    let canvas = Canvas::default()
        .block(Block::default())
        .x_bounds([0.0, area.width as f64])
        .y_bounds([0.0, 2.0])
        .paint(|ctx| {
            let width = area.width as f64;

            // Draw a straight line in the middle for bus signals
            ctx.draw(&Line {
                x1: 0.0,
                y1: 1.0,
                x2: width,
                y2: 1.0,
                color: style.fg.unwrap_or(Color::White),
            });

            // Draw vertical transitions at change points
            let mut prev_x = None;

            for (i, (t, _)) in values.iter().enumerate() {
                if i == 0 {
                    continue;
                }

                let x = ((*t - values[0].0) as f64
                    / (values.last().unwrap().0 - values[0].0) as f64)
                    * width;

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
                }

                prev_x = Some(x);
            }
        });

    frame.render_widget(canvas, area);

    // Draw bus value labels
    for (t, v) in values {
        if let WaveValue::Bus(value) = v {
            // Calculate x position
            let x_ratio =
                (*t - values[0].0) as f64 / (values.last().unwrap().0 - values[0].0).max(1) as f64;
            let x_pos = (x_ratio * area.width as f64) as u16;

            if x_pos < area.width.saturating_sub(value.len() as u16) {
                let label_area = Rect::new(area.x + x_pos, area.y, value.len() as u16, 1);

                frame.render_widget(Paragraph::new(value.clone()).style(style), label_area);
            }
        }
    }
}
