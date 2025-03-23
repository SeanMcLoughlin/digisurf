use ratatui::{buffer::Buffer, layout::Rect, widgets::StatefulWidget};

use crate::{constants::DEFAULT_SAVED_MARKER_COLOR, state::AppState};

#[derive(Default, Copy, Clone)]
pub struct MarkerNamesWidget {}

impl MarkerNamesWidget {
    pub fn draw_saved_markers(&self, buf: &mut Buffer, area: Rect, state: &AppState) {
        // First, find which markers are visible in the current time window
        let visible_markers: Vec<_> = state
            .saved_markers
            .iter()
            .filter(|marker| {
                marker.time >= state.time_start
                    && marker.time <= state.time_start + state.time_range
            })
            .collect();

        if visible_markers.is_empty() {
            return;
        }

        // Create a list of marker positions and their display names
        let mut marker_displays = Vec::new();

        // Calculate positions for each marker
        for marker in visible_markers {
            // Calculate x position based on time
            let x_pos = ((marker.time - state.time_start) as f64 / state.time_range as f64
                * area.width as f64) as u16;

            // Only consider markers that start within the visible area
            if x_pos < area.width {
                let marker_pos = area.x + x_pos;
                let marker_style = ratatui::style::Style::default().fg(DEFAULT_SAVED_MARKER_COLOR);
                marker_displays.push((marker_pos, marker.name.clone(), marker_style));
            }
        }

        // Sort markers by position
        marker_displays.sort_by_key(|(pos, _, _)| *pos);

        // First, display all marker indicators with minimum representations (just first character)
        // This ensures all markers are at least minimally visible
        for (pos, name, style) in marker_displays.iter() {
            if let Some(first_char) = name.chars().next() {
                // Draw a single character indicator for each marker
                if *pos < area.right() {
                    buf[(*pos, area.y)].set_char(first_char).set_style(*style);
                }
            }
        }

        // Now draw the full names where there's space available
        // Using a greedy approach - markers that are further apart get their full names
        let mut i = 0;
        while i < marker_displays.len() {
            let (pos, name, style) = &marker_displays[i];
            let mut end_pos = *pos + name.len() as u16;
            let mut display_name = name.clone();

            // Check for right edge truncation
            if end_pos > area.right() {
                let available_width = area.right() - *pos;
                display_name = display_name
                    .chars()
                    .take(available_width as usize)
                    .collect();
                end_pos = area.right();
            }

            // Check if this marker would overlap with the next marker
            let will_overlap = i + 1 < marker_displays.len() && end_pos > marker_displays[i + 1].0;

            if will_overlap {
                // If there would be an overlap, just show the first character
                if *pos < area.right() {
                    // First character already drawn in the previous loop
                }
            } else {
                // There's enough room, draw the full name
                for (j, c) in display_name.chars().enumerate().skip(1) {
                    // Skip first char, already drawn
                    let x = *pos + j as u16;
                    if x < area.right() {
                        buf[(x, area.y)].set_char(c).set_style(*style);
                    }
                }
            }

            i += 1;
        }
    }
}

impl StatefulWidget for MarkerNamesWidget {
    type State = AppState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if state.saved_markers.is_empty() {
            return;
        }

        self.draw_saved_markers(buf, area, &state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{state::AppState, types::Marker};
    use insta::assert_snapshot;
    use ratatui::{backend::TestBackend, Terminal};

    fn setup_state() -> AppState {
        let mut state = AppState::default();
        state.time_start = 0;
        state.time_range = 100;

        // Add some markers at different positions
        state.saved_markers.push(Marker {
            time: 10,
            name: "Marker1".to_string(),
        });

        state.saved_markers.push(Marker {
            time: 40,
            name: "Marker2".to_string(),
        });

        state
    }

    #[test]
    fn test_marker_names_normal() {
        let widget = MarkerNamesWidget::default();
        let mut state = setup_state();

        // Create a test terminal
        let backend = TestBackend::new(80, 1);
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

    #[test]
    fn test_marker_names_overlapping_right_side() {
        let widget = MarkerNamesWidget::default();
        let mut state = setup_state();

        // Add overlapping marker for the leftmost marker.
        // The leftmost marker should be reduced to a single character 'M'
        state.saved_markers.push(Marker {
            time: 15,
            name: "Overlap".to_string(),
        });

        let backend = TestBackend::new(80, 1);
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

    #[test]
    fn test_marker_names_overlapping_left_side() {
        let widget = MarkerNamesWidget::default();
        let mut state = setup_state();

        // Add overlapping marker for the rightmost marker.
        // This added marker should be reduced to a single character 'O'
        state.saved_markers.push(Marker {
            time: 35,
            name: "Overlap".to_string(),
        });

        let backend = TestBackend::new(80, 1);
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

    #[test]
    fn test_marker_names_overlapping_both_side() {
        let widget = MarkerNamesWidget::default();
        let mut state = setup_state();

        // Add overlapping marker starting at the leftmost marker, but make the name of this
        // marker long enough so it overlaps with the rightmost marker, too.
        // The two leftmost markers should be reduced to "M" and "O"
        state.saved_markers.push(Marker {
            time: 15,
            name: "OverlapOverlapOverlapOverlapOverlap".to_string(),
        });

        let backend = TestBackend::new(80, 1);
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
