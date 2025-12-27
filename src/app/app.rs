use crossterm::{
    event::{self, DisableBracketedPaste, EnableBracketedPaste, Event, KeyCode, KeyEvent},
    terminal::{disable_raw_mode, enable_raw_mode},
    ExecutableCommand,
};

use ratatui::{prelude::CrosstermBackend, Terminal};

use crate::engine::mode::VimMode;
use crate::engine::vim_buffer::MoveDirection;
use crate::utils::Result;

use super::screens::*;
use super::ui::UI;

/// Main application state
pub struct App {
    pub should_quit: bool,
    pub current_screen: Screen,
    pub main_menu_state: MainMenuState,
    pub practice_state: PracticeState,
    pub progress_state: ProgressState,
    pub settings_state: SettingsState,
}

impl App {
    pub fn new() -> Result<Self> {
        Ok(Self {
            should_quit: false,
            current_screen: Screen::MainMenu,
            main_menu_state: MainMenuState::new(),
            practice_state: PracticeState::new(),
            progress_state: ProgressState::new(),
            settings_state: SettingsState::new(),
        })
    }

    pub fn run(&mut self) -> Result<()> {
        enable_raw_mode()?;
        std::io::stdout().execute(EnableBracketedPaste)?;

        let mut stdout = std::io::stdout();
        let backend = CrosstermBackend::new(&mut stdout);
        let mut terminal = Terminal::new(backend)?;

        let ui = UI::new();

        loop {
            terminal.draw(|frame| ui.render(frame, self))?;

            if self.should_quit {
                break;
            }

            // Handle events
            if event::poll(std::time::Duration::from_millis(100))? {
                self.handle_event()?;
            }
        }

        std::io::stdout().execute(DisableBracketedPaste)?;
        disable_raw_mode()?;
        Ok(())
    }

    fn handle_event(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key_event) => self.handle_key(key_event),
            Event::Resize(_, _) => Ok(()),
            Event::Paste(_) => Ok(()),
            Event::FocusGained | Event::FocusLost | Event::Mouse(_) => Ok(()),
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        match self.current_screen {
            Screen::MainMenu => self.handle_main_menu_key(key),
            Screen::DailyDrill | Screen::FreePractice => self.handle_practice_key(key),
            Screen::GuidedLearning => self.handle_practice_key(key),
            Screen::Progress => self.handle_progress_key(key),
            Screen::Settings => self.handle_settings_key(key),
            Screen::Help => self.handle_help_key(key),
        }
    }

    fn handle_main_menu_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.should_quit = true;
            }
            KeyCode::Char('j') | KeyCode::Down | KeyCode::Tab => {
                self.main_menu_state.next();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.main_menu_state.previous();
            }
            KeyCode::Enter => {
                let item = self.main_menu_state.current_item().title.as_str();
                match item {
                    "Daily Drill" => {
                        self.current_screen = Screen::DailyDrill;
                        self.practice_state.reset();
                        self.start_daily_drill();
                    }
                    "Guided Learning" => {
                        self.current_screen = Screen::GuidedLearning;
                        self.practice_state.reset();
                    }
                    "Free Practice" => {
                        self.current_screen = Screen::FreePractice;
                        self.practice_state.reset();
                    }
                    "Progress" => {
                        self.current_screen = Screen::Progress;
                    }
                    "Settings" => {
                        self.current_screen = Screen::Settings;
                    }
                    "Help" => {
                        self.current_screen = Screen::Help;
                    }
                    "Quit" => {
                        self.should_quit = true;
                    }
                    _ => {}
                }
            }
            // Shortcut keys
            KeyCode::Char('d') => {
                self.current_screen = Screen::DailyDrill;
                self.practice_state.reset();
                self.start_daily_drill();
            }
            KeyCode::Char('g') => {
                self.current_screen = Screen::GuidedLearning;
                self.practice_state.reset();
            }
            KeyCode::Char('f') => {
                self.current_screen = Screen::FreePractice;
                self.practice_state.reset();
            }
            KeyCode::Char('p') => {
                self.current_screen = Screen::Progress;
            }
            KeyCode::Char('s') => {
                self.current_screen = Screen::Settings;
            }
            KeyCode::Char('?') => {
                self.current_screen = Screen::Help;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_practice_key(&mut self, key: KeyEvent) -> Result<()> {
        // Return to main menu on Escape
        if key.code == KeyCode::Esc {
            self.current_screen = Screen::MainMenu;
            self.practice_state.reset();
            return Ok(());
        }

        // Handle mode switching first (in Normal mode)
        if self.practice_state.vim_mode == VimMode::Normal {
            match key.code {
                KeyCode::Char('i') => {
                    self.practice_state.vim_mode = VimMode::Insert;
                    return Ok(());
                }
                KeyCode::Char('a') => {
                    self.practice_state.vim_buffer.cursor_col = self.practice_state.vim_buffer.cursor_col.saturating_add(1);
                    self.practice_state.vim_mode = VimMode::Insert;
                    return Ok(());
                }
                KeyCode::Char('v') => {
                    self.practice_state.vim_mode = VimMode::Visual;
                    return Ok(());
                }
                KeyCode::Char(c) => {
                    // Check if this triggers a command
                    self.check_command(c.to_string());
                    return Ok(());
                }
                _ => {}
            }
        }

        // In practice mode, pass keys to vim buffer
        match key.code {
            KeyCode::Char(c) => {
                self.practice_state.vim_buffer.insert_char(c);
            }
            KeyCode::Up => {
                self.practice_state.vim_buffer.move_cursor(MoveDirection::Up);
            }
            KeyCode::Down => {
                self.practice_state.vim_buffer.move_cursor(MoveDirection::Down);
            }
            KeyCode::Left => {
                self.practice_state.vim_buffer.move_cursor(MoveDirection::Left);
            }
            KeyCode::Right => {
                self.practice_state.vim_buffer.move_cursor(MoveDirection::Right);
            }
            KeyCode::Backspace => {
                // Simple backspace handling
                if self.practice_state.vim_buffer.cursor_col > 0 {
                    self.practice_state.vim_buffer.cursor_col -= 1;
                    let line = &mut self.practice_state.vim_buffer.lines[self.practice_state.vim_buffer.cursor_row];
                    if !line.is_empty() {
                        line.remove(self.practice_state.vim_buffer.cursor_col);
                    }
                }
            }
            KeyCode::Enter => {
                // Add new line
                let mut current_line = self.practice_state.vim_buffer.lines[self.practice_state.vim_buffer.cursor_row].clone();
                let after_cursor = current_line.split_off(self.practice_state.vim_buffer.cursor_col);
                self.practice_state.vim_buffer.lines.insert(self.practice_state.vim_buffer.cursor_row + 1, after_cursor);
                self.practice_state.vim_buffer.cursor_row += 1;
                self.practice_state.vim_buffer.cursor_col = 0;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_progress_key(&mut self, key: KeyEvent) -> Result<()> {
        if key.code == KeyCode::Esc || key.code == KeyCode::Char('q') || key.code == KeyCode::Char('b') {
            self.current_screen = Screen::MainMenu;
        }
        Ok(())
    }

    fn handle_settings_key(&mut self, key: KeyEvent) -> Result<()> {
        if key.code == KeyCode::Esc || key.code == KeyCode::Char('q') || key.code == KeyCode::Char('b') {
            self.current_screen = Screen::MainMenu;
            return Ok(());
        }

        match key.code {
            KeyCode::Char('j') | KeyCode::Down | KeyCode::Tab => {
                self.settings_state.selected_index = (self.settings_state.selected_index + 1) % 4;
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.settings_state.selected_index = if self.settings_state.selected_index == 0 {
                    3
                } else {
                    self.settings_state.selected_index - 1
                };
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                // Toggle settings
                match self.settings_state.selected_index {
                    0 => self.settings_state.hints_enabled = !self.settings_state.hints_enabled,
                    1 => {
                        self.settings_state.drill_duration = match self.settings_state.drill_duration {
                            DrillDuration::Short => DrillDuration::Medium,
                            DrillDuration::Medium => DrillDuration::Long,
                            DrillDuration::Long => DrillDuration::Short,
                        }
                    }
                    2 => {
                        self.settings_state.difficulty = match self.settings_state.difficulty {
                            Difficulty::Beginner => Difficulty::Intermediate,
                            Difficulty::Intermediate => Difficulty::Advanced,
                            Difficulty::Advanced => Difficulty::Beginner,
                        }
                    }
                    3 => self.settings_state.sound_enabled = !self.settings_state.sound_enabled,
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_help_key(&mut self, key: KeyEvent) -> Result<()> {
        if key.code == KeyCode::Esc || key.code == KeyCode::Char('q') || key.code == KeyCode::Char('b') {
            self.current_screen = Screen::MainMenu;
        }
        Ok(())
    }

    fn start_daily_drill(&mut self) {
        self.practice_state.current_instruction = "Daily Drill: Practice these commands".to_string();
        self.practice_state.hint = "Press the keybinding for the command shown".to_string();
    }

    fn check_command(&mut self, key: String) {
        // Placeholder for command checking logic
        self.practice_state.key_buffer.push_str(&key);
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}
