use crate::app::App;
use crate::input::keybindings::KeyBindings;
use crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEventKind};

pub fn handle_input(app: &mut App, key: KeyCode, keybinds: &KeyBindings) -> bool {
    if key == keybinds.quit {
        return false;
    }

    if key == keybinds.help {
        app.show_help = !app.show_help;
    } else if !app.show_help {
        handle_normal_mode(app, key, keybinds);
    }

    true
}

pub fn handle_mouse(
    app: &mut App,
    kind: MouseEventKind,
    column: u16,
    row: u16,
    modifiers: KeyModifiers, // Add modifier parameter
    waveform_area: ratatui::layout::Rect,
) -> bool {
    if app.show_help {
        return true;
    }

    // Check if click is within waveform area
    if column >= waveform_area.x
        && column <= waveform_area.right()
        && row >= waveform_area.y
        && row <= waveform_area.bottom()
    {
        // Convert column to coordinates inside waveform area
        let column_in_waveform = column - waveform_area.x;

        match kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if modifiers.contains(KeyModifiers::SHIFT) {
                    // Shift+Left Click = place secondary (white) marker
                    app.set_secondary_marker(column_in_waveform, waveform_area.width);
                } else {
                    // Regular Left Click = place primary (yellow) marker
                    app.set_primary_marker(column_in_waveform, waveform_area.width);
                }
            }
            _ => {}
        }
    }

    true
}

fn handle_normal_mode(app: &mut App, key: KeyCode, keybinds: &KeyBindings) {
    match key {
        k if k == keybinds.down => {
            if !app.show_help {
                app.selected_signal = (app.selected_signal + 1) % app.signals.len();
            }
        }
        k if k == keybinds.up => {
            if !app.show_help {
                if app.selected_signal > 0 {
                    app.selected_signal -= 1;
                } else {
                    app.selected_signal = app.signals.len() - 1;
                }
            }
        }
        k if k == keybinds.left => {
            if !app.show_help && app.waveform.time_start > 0 {
                app.waveform.time_start = app
                    .waveform
                    .time_start
                    .saturating_sub(app.waveform.time_range / 4);
            }
        }
        k if k == keybinds.right => {
            if !app.show_help && app.waveform.time_start < app.max_time {
                app.waveform.time_start = (app.waveform.time_start + app.waveform.time_range / 4)
                    .min(app.max_time - app.waveform.time_range);
            }
        }
        k if k == keybinds.zoom_out => {
            if !app.show_help {
                app.waveform.time_range = (app.waveform.time_range * 2).min(app.max_time);
            }
        }
        k if k == keybinds.zoom_in => {
            if !app.show_help {
                app.waveform.time_range = (app.waveform.time_range / 2).max(10);
            }
        }
        k if k == keybinds.delete_primary_marker => {
            app.waveform.primary_marker = None;
        }
        k if k == keybinds.delete_secondary_marker => {
            app.waveform.secondary_marker = None;
        }

        _ => {}
    }
}
