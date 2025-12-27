use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use ratatui::{prelude::CrosstermBackend, Terminal};

use crate::utils::Result;

use super::ui::UI;

pub struct App {
    should_quit: bool,
}

impl App {
    pub fn new() -> Result<Self> {
        Ok(Self { should_quit: false })
    }

    pub fn run(&mut self) -> Result<()> {
        enable_raw_mode()?;

        let mut stdout = std::io::stdout();
        let backend = CrosstermBackend::new(&mut stdout);
        let mut terminal = Terminal::new(backend)?;

        let ui = UI::new();

        loop {
            terminal.draw(|frame| ui.render(frame))?;

            if self.should_quit {
                break;
            }
        }

        disable_raw_mode()?;
        Ok(())
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}
