use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};

use super::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MascotState {
    Normal,
    Happy,
    Angry,
    Thinking,
    Sleeping,
}

pub fn get_mascot_lines(state: &MascotState, theme: Theme) -> Vec<Line<'static>> {
    // BEGIN GENERATED CAT PIXELS
    let mut lines = vec![
        Line::from(vec![
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
            Span::styled("▄", Style::default().fg(Color::Rgb(22, 23, 26))),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(22, 23, 26))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled("▄", Style::default().fg(Color::Rgb(22, 23, 26))),
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
            Span::styled("▄", Style::default().fg(Color::Rgb(22, 23, 26))),
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
        ]),
        Line::from(vec![
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(22, 23, 26))
                    .bg(Color::Rgb(22, 23, 26)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(22, 23, 26))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled("▄", Style::default().fg(Color::Rgb(22, 23, 26))),
            Span::styled("▄", Style::default().fg(Color::Rgb(22, 23, 26))),
            Span::styled("▄", Style::default().fg(Color::Rgb(22, 23, 26))),
            Span::styled("▄", Style::default().fg(Color::Rgb(22, 23, 26))),
            Span::styled("▄", Style::default().fg(Color::Rgb(22, 23, 26))),
            Span::styled("▄", Style::default().fg(Color::Rgb(22, 23, 26))),
            Span::styled("▄", Style::default().fg(Color::Rgb(22, 23, 26))),
            Span::styled("▄", Style::default().fg(Color::Rgb(22, 23, 26))),
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
            Span::styled("▄", Style::default().fg(Color::Rgb(22, 23, 26))),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(22, 23, 26))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(22, 23, 26))
                    .bg(Color::Rgb(22, 23, 26)),
            ),
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
        ]),
        Line::from(vec![
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(22, 23, 26))
                    .bg(Color::Rgb(22, 23, 26)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(22, 23, 26))
                    .bg(Color::Rgb(22, 23, 26)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(22, 23, 26))
                    .bg(Color::Rgb(22, 23, 26)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(22, 23, 26))
                    .bg(Color::Rgb(22, 23, 26)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(22, 23, 26))
                    .bg(Color::Rgb(22, 23, 26)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(22, 23, 26))
                    .bg(Color::Rgb(22, 23, 26)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(22, 23, 26))
                    .bg(Color::Rgb(22, 23, 26)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(22, 23, 26))
                    .bg(Color::Rgb(22, 23, 26)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(22, 23, 26))
                    .bg(Color::Rgb(22, 23, 26)),
            ),
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
        ]),
        Line::from(vec![
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
            Span::styled("▄", Style::default().fg(Color::Rgb(22, 23, 26))),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(22, 23, 26))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(22, 23, 26))
                    .bg(Color::Rgb(22, 23, 26)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(22, 23, 26))
                    .bg(Color::Rgb(22, 23, 26)),
            ),
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
        ]),
        Line::from(vec![
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
            Span::styled("▄", Style::default().fg(Color::Rgb(22, 23, 26))),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(22, 23, 26))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(22, 23, 26))
                    .bg(Color::Rgb(22, 23, 26)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(22, 23, 26))
                    .bg(Color::Rgb(22, 23, 26)),
            ),
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
            Span::raw(" "),
        ]),
        Line::from(vec![
            Span::raw(" "),
            Span::styled("▄", Style::default().fg(Color::Rgb(22, 23, 26))),
            Span::styled("▄", Style::default().fg(Color::Rgb(22, 23, 26))),
            Span::styled("▄", Style::default().fg(Color::Rgb(22, 23, 26))),
            Span::styled("▄", Style::default().fg(Color::Rgb(22, 23, 26))),
            Span::styled("▄", Style::default().fg(Color::Rgb(22, 23, 26))),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(22, 23, 26))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(22, 23, 26)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(22, 23, 26)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(22, 23, 26))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled("▄", Style::default().fg(Color::Rgb(22, 23, 26))),
            Span::raw(" "),
            Span::raw(" "),
        ]),
        Line::from(vec![
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(22, 23, 26))
                    .bg(Color::Rgb(22, 23, 26)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(246, 138, 152)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(246, 138, 152)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(246, 138, 152)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(22, 23, 26))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(22, 23, 26))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(22, 23, 26))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(22, 23, 26))
                    .bg(Color::Rgb(22, 23, 26)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(22, 23, 26))
                    .bg(Color::Rgb(22, 23, 26)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(22, 23, 26)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(22, 23, 26))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(22, 23, 26))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(22, 23, 26))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(22, 23, 26))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(119, 154, 202))
                    .bg(Color::Rgb(22, 23, 26)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(119, 154, 202)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(119, 154, 202)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(22, 23, 26))
                    .bg(Color::Rgb(22, 23, 26)),
            ),
            Span::raw(" "),
        ]),
        Line::from(vec![
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(22, 23, 26))
                    .bg(Color::Rgb(22, 23, 26)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(22, 23, 26))
                    .bg(Color::Rgb(22, 23, 26)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(22, 23, 26))
                    .bg(Color::Rgb(22, 23, 26)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(22, 23, 26))
                    .bg(Color::Rgb(22, 23, 26)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(22, 23, 26))
                    .bg(Color::Rgb(22, 23, 26)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(22, 23, 26))
                    .bg(Color::Rgb(22, 23, 26)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(246, 138, 152))
                    .bg(Color::Rgb(246, 138, 152)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(246, 138, 152))
                    .bg(Color::Rgb(246, 138, 152)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(22, 23, 26))
                    .bg(Color::Rgb(22, 23, 26)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(22, 23, 26))
                    .bg(Color::Rgb(22, 23, 26)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(119, 154, 202))
                    .bg(Color::Rgb(119, 154, 202)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(22, 23, 26))
                    .bg(Color::Rgb(22, 23, 26)),
            ),
        ]),
        Line::from(vec![
            Span::styled("▀", Style::default().fg(Color::Rgb(22, 23, 26))),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(246, 138, 152))
                    .bg(Color::Rgb(22, 23, 26)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(246, 138, 152))
                    .bg(Color::Rgb(246, 138, 152)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(246, 138, 152))
                    .bg(Color::Rgb(246, 138, 152)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(246, 138, 152))
                    .bg(Color::Rgb(246, 138, 152)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(119, 154, 202)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(22, 23, 26))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(22, 23, 26))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(22, 23, 26))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(22, 23, 26))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(22, 23, 26))
                    .bg(Color::Rgb(22, 23, 26)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(246, 138, 152)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(246, 138, 152)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(246, 138, 152)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(22, 23, 26))
                    .bg(Color::Rgb(255, 255, 255)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(119, 154, 202))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(186, 216, 247)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(22, 23, 26))
                    .bg(Color::Rgb(22, 23, 26)),
            ),
        ]),
        Line::from(vec![
            Span::raw(" "),
            Span::raw(" "),
            Span::styled("▀", Style::default().fg(Color::Rgb(22, 23, 26))),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(22, 23, 26)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(119, 154, 202)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(119, 154, 202)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(119, 154, 202)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(119, 154, 202))
                    .bg(Color::Rgb(119, 154, 202)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(119, 154, 202))
                    .bg(Color::Rgb(119, 154, 202)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(119, 154, 202)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(119, 154, 202)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(119, 154, 202)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(119, 154, 202)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(119, 154, 202)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(119, 154, 202)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(119, 154, 202)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(119, 154, 202)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(119, 154, 202)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(119, 154, 202)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(119, 154, 202)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(119, 154, 202)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(119, 154, 202)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(119, 154, 202))
                    .bg(Color::Rgb(119, 154, 202)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(119, 154, 202))
                    .bg(Color::Rgb(119, 154, 202)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(119, 154, 202))
                    .bg(Color::Rgb(119, 154, 202)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(22, 23, 26))
                    .bg(Color::Rgb(119, 154, 202)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(22, 23, 26)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(246, 138, 152))
                    .bg(Color::Rgb(119, 154, 202)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(246, 138, 152))
                    .bg(Color::Rgb(119, 154, 202)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(119, 154, 202)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(119, 154, 202)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(119, 154, 202)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(119, 154, 202)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(119, 154, 202)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(186, 216, 247))
                    .bg(Color::Rgb(119, 154, 202)),
            ),
            Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(22, 23, 26))
                    .bg(Color::Rgb(22, 23, 26)),
            ),
        ]),
    ];
    // END GENERATED CAT PIXELS

    recolor_sprite(&mut lines, theme);

    // Add a state bubble on top!
    let bubble = match state {
        MascotState::Normal => "   zZz...   ",
        MascotState::Happy => "  Meow!! ^_^",
        MascotState::Angry => "  Hiss!! >_<",
        MascotState::Thinking => "  Hmm... ?  ",
        MascotState::Sleeping => "  zZz...     ",
    };

    lines.insert(
        0,
        Line::from(vec![Span::styled(bubble, Style::default().fg(theme.amber))]),
    );
    lines.insert(1, Line::from(vec![Span::raw("")]));

    lines
}

/// Maps the five source-image colors onto the active Gruvbox palette. The
/// sprite's background colors are intentional lower half-pixels, not an opaque
/// application canvas, so they must be recolored rather than removed.
fn recolor_sprite(lines: &mut [Line<'static>], theme: Theme) {
    for line in lines {
        line.style.fg = line.style.fg.map(|color| sprite_color(color, theme));
        line.style.bg = line.style.bg.map(|color| sprite_color(color, theme));
        for span in &mut line.spans {
            span.style.fg = span.style.fg.map(|color| sprite_color(color, theme));
            span.style.bg = span.style.bg.map(|color| sprite_color(color, theme));
        }
    }
}

fn sprite_color(color: Color, theme: Theme) -> Color {
    match color {
        Color::Rgb(22, 23, 26) => theme.mascot_outline,
        Color::Rgb(186, 216, 247) => theme.mascot_fur,
        Color::Rgb(255, 255, 255) => theme.mascot_highlight,
        Color::Rgb(119, 154, 202) => theme.mascot_shade,
        Color::Rgb(246, 138, 152) => theme.mascot_accent,
        _ => color,
    }
}

/// A responsive version of Kage for terminals where the full pixel sprite
/// would steal too much room from the practice buffer.
#[must_use]
pub fn get_compact_mascot_lines(state: MascotState, tick: u64, theme: Theme) -> Vec<Line<'static>> {
    let fur = Style::default().fg(theme.mascot_fur);
    let glow = Style::default().fg(theme.cyan);
    let alert = Style::default().fg(theme.danger);
    let amber = Style::default().fg(theme.amber);
    let blink = tick.is_multiple_of(31);
    let (eyes, mouth, accent) = match state {
        MascotState::Normal if blink => ("▄▄", "╰━╯", glow),
        MascotState::Normal => ("██", "╰━╯", glow),
        MascotState::Happy => ("▀▀", "╰┻╯", amber),
        MascotState::Angry => ("◢◣", "╭━╮", alert),
        MascotState::Thinking => ("█?", "╰─╯", amber),
        MascotState::Sleeping => ("▄▄", "╰─╯", glow),
    };

    vec![
        Line::from(Span::styled("   ▄█▄     ▄█▄", fur)),
        Line::from(Span::styled("  █▒▒█▄▄▄▄▄█▒▒█", fur)),
        Line::from(Span::styled("  █▒▒▒▒▒▒▒▒▒▒▒█", fur)),
        Line::from(vec![
            Span::styled("  █▒▒", fur),
            Span::styled(eyes, accent),
            Span::styled("▒▒▒", fur),
            Span::styled(eyes, accent),
            Span::styled("▒▒█", fur),
        ]),
        Line::from(Span::styled("  █▒▒▒▒▄▒▒▒▒▒▒█", fur)),
        Line::from(vec![
            Span::styled("   █▒▒", fur),
            Span::styled(mouth, accent),
            Span::styled("▒▒█", fur),
        ]),
        Line::from(Span::styled("    ▀███████▀", fur)),
        Line::from(Span::styled(
            if tick % 8 < 4 {
                "      ▀█▀█▀  ╲"
            } else {
                "   ╱  ▀█▀█▀"
            },
            glow,
        )),
    ]
}

#[must_use]
pub const fn coach_line(state: MascotState) -> &'static str {
    match state {
        MascotState::Normal => "[KAGE] Elige una ruta. Las flechas hacen ruido.",
        MascotState::Happy => "[KAGE] Mrrp. Ruta limpia; la memoria ya está aprendiendo.",
        MascotState::Angry => "[KAGE] Señal inválida. Respira y lee el comando pendiente.",
        MascotState::Thinking => "[KAGE] Piensa en intención: verbo + movimiento.",
        MascotState::Sleeping => "[KAGE] Sesión en pausa. Las garras también descansan.",
    }
}

#[cfg(test)]
mod responsive_tests {
    use super::*;
    use crate::terminal_appearance::TerminalTheme;

    #[test]
    fn compact_cat_has_a_stable_footprint() {
        for state in [
            MascotState::Normal,
            MascotState::Happy,
            MascotState::Angry,
            MascotState::Thinking,
            MascotState::Sleeping,
        ] {
            let lines = get_compact_mascot_lines(
                state,
                0,
                Theme::gruvbox(TerminalTheme::Dark, false, false),
            );
            assert_eq!(lines.len(), 8);
            assert!(lines.iter().all(|line| line.width() <= 18));
        }
    }

    #[test]
    fn full_cat_is_recolored_for_both_gruvbox_variants() {
        let dark = get_mascot_lines(
            &MascotState::Normal,
            Theme::gruvbox(TerminalTheme::Dark, false, false),
        );
        let light = get_mascot_lines(
            &MascotState::Normal,
            Theme::gruvbox(TerminalTheme::Light, false, false),
        );

        let colors = |lines: &[Line<'_>]| {
            lines
                .iter()
                .flat_map(|line| &line.spans)
                .flat_map(|span| [span.style.fg, span.style.bg])
                .flatten()
                .collect::<Vec<_>>()
        };
        let dark_colors = colors(&dark);
        let light_colors = colors(&light);
        assert!(dark_colors.contains(&Color::Rgb(250, 189, 47)));
        assert!(light_colors.contains(&Color::Rgb(181, 118, 20)));
        assert!(!dark_colors.contains(&Color::Rgb(186, 216, 247)));
        assert!(!light_colors.contains(&Color::Rgb(186, 216, 247)));
        assert_ne!(dark_colors, light_colors);
    }
}
