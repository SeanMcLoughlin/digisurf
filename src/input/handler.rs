use crate::app::App;
use crate::input::keybindings::KeyBindings;
use crossterm::event::KeyCode;

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
            if !app.show_help && app.time_offset > 0 {
                app.time_offset = app.time_offset.saturating_sub(app.window_size / 4);
            }
        }
        k if k == keybinds.right => {
            if !app.show_help && app.time_offset < app.max_time {
                app.time_offset =
                    (app.time_offset + app.window_size / 4).min(app.max_time - app.window_size);
            }
        }
        k if k == keybinds.zoom_in => {
            if !app.show_help {
                app.window_size = (app.window_size * 2).min(app.max_time);
            }
        }
        k if k == keybinds.zoom_out => {
            if !app.show_help {
                app.window_size = (app.window_size / 2).max(10);
            }
        }
        _ => {}
    }
}
