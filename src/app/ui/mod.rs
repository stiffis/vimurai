use ratatui::prelude::*;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub struct UI;

impl UI {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, frame: &mut Frame) {
        let area = frame.size();

        let title = Paragraph::new("Vimurai - Master Vim")
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));

        frame.render_widget(title, area);
    }
}
