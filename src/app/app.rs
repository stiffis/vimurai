use crossterm::{
    cursor::{Hide, Show, SetCursorStyle},
    event::{self, DisableBracketedPaste, EnableBracketedPaste, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};

use ratatui::{prelude::CrosstermBackend, Terminal};

use crate::engine::mode::VimMode;
use crate::engine::vim_buffer::{MoveDirection, Point};
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
    pub show_quit_confirm: bool,
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
            show_quit_confirm: false,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        // Setup panic handler to restore terminal on crash
        let original_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            let _ = disable_raw_mode();
            let _ = execute!(std::io::stdout(), LeaveAlternateScreen, Show);
            original_hook(panic_info);
        }));

        // Setup terminal: enter alternate screen and hide cursor
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen, Hide)?;
        enable_raw_mode()?;
        stdout.execute(EnableBracketedPaste)?;

        let backend = CrosstermBackend::new(&mut stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;

        let ui = UI::new();

        // Track previous mode for cursor shape changes
        let mut previous_mode: Option<VimMode> = None;

        loop {
            // Use a scope to limit the borrow lifetime
            {
                let mut stdout = std::io::stdout();
                self.update_cursor_shape(&mut stdout, previous_mode);
            }
            previous_mode = Some(self.practice_state.vim_mode);

            terminal.draw(|frame| ui.render(frame, self))?;

            if self.should_quit {
                break;
            }

            // Handle events
            if event::poll(std::time::Duration::from_millis(50))? {
                self.handle_event()?;
            }
        }

        // Cleanup: restore terminal state
        terminal.show_cursor()?;
        execute!(
            std::io::stdout(),
            SetCursorStyle::DefaultUserShape,
            Show,
            LeaveAlternateScreen
        )?;
        std::io::stdout().execute(DisableBracketedPaste)?;
        disable_raw_mode()?;
        Ok(())
    }

    fn update_cursor_shape(&self, stdout: &mut std::io::Stdout, previous_mode: Option<VimMode>) {
        // Only update if mode changed and we're in a practice screen
        if previous_mode == Some(self.practice_state.vim_mode) {
            return;
        }

        let shape = match self.practice_state.vim_mode {
            VimMode::Insert => SetCursorStyle::SteadyBar,
            VimMode::Visual => SetCursorStyle::SteadyUnderScore,
            _ => SetCursorStyle::SteadyBlock,
        };

        // Only change cursor if we're in practice mode
        if matches!(
            self.current_screen,
            Screen::DailyDrill | Screen::FreePractice | Screen::GuidedLearning
        ) {
            let _ = stdout.execute(shape);
        }
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
        // Solo manejar keys en Practice screens
        if matches!(
            self.current_screen,
            Screen::DailyDrill | Screen::FreePractice | Screen::GuidedLearning
        ) {
            self.handle_practice_key(key)?;
        } else {
            // Menu screens
            match self.current_screen {
                Screen::MainMenu => self.handle_main_menu_key(key)?,
                Screen::Progress => self.handle_progress_key(key)?,
                Screen::Settings => self.handle_settings_key(key)?,
                Screen::Help => self.handle_help_key(key)?,
                _ => {}
            };
        }
        Ok(())
    }

    fn handle_main_menu_key(&mut self, key: KeyEvent) -> Result<()> {
        // If showing quit confirmation, handle Y/N
        if self.show_quit_confirm {
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    self.should_quit = true;
                }
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                    self.show_quit_confirm = false;
                }
                _ => {}
            }
            return Ok(());
        }

        // Normal menu handling
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.show_quit_confirm = true;
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
                        self.show_quit_confirm = true;
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
        // El manejador de teclado siempre debe pasar por aquí, nunca salir directamente al menú
        // El Escape solo cambia de modo, no sale de la pantalla

        match self.practice_state.vim_mode {
            VimMode::Normal => self.handle_normal_mode(key),
            VimMode::Insert => self.handle_insert_mode(key),
            VimMode::Visual => self.handle_visual_mode(key),
            VimMode::Command => self.handle_command_mode(key),
        }
    }

    fn handle_normal_mode(&mut self, key: KeyEvent) -> Result<()> {
        // Check for pending operator/motion (like 'f' waiting for char)
        let pending = self.practice_state.key_buffer.clone();
        if !pending.is_empty() {
             let first_char = pending.chars().next().unwrap();
             if matches!(first_char, 'f' | 'F' | 't' | 'T') && pending.len() == 1 {
                 if let KeyCode::Char(target) = key.code {
                     let forward = matches!(first_char, 'f' | 't');
                     let inclusive = matches!(first_char, 'f' | 'F');
                     self.practice_state.vim_buffer.find_char_in_line(target, forward, inclusive);
                     self.practice_state.key_buffer.clear();
                     return Ok(());
                 } else if key.code == KeyCode::Esc {
                     self.practice_state.key_buffer.clear();
                     return Ok(());
                 }
             }
        }

        match key.code {
            KeyCode::Esc => {
                self.practice_state.key_buffer.clear();
            }

            // Undo / Redo
            KeyCode::Char('u') => {
                self.practice_state.vim_buffer.undo();
            }
            KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.practice_state.vim_buffer.redo();
            }

            // Insert Modes
            KeyCode::Char('i') => {
                self.practice_state.vim_mode = VimMode::Insert;
            }
            KeyCode::Char('a') => {
                self.practice_state.vim_buffer.move_cursor(MoveDirection::Right);
                self.practice_state.vim_mode = VimMode::Insert;
            }
            KeyCode::Char('I') => {
                self.practice_state.vim_buffer.move_to_non_blank_start();
                self.practice_state.vim_mode = VimMode::Insert;
            }
            KeyCode::Char('A') => {
                self.practice_state.vim_buffer.move_to_line_end();
                // Move one past end is handled by insert mode logic usually, 
                // but let's ensure we are at end.
                // Vim 'A' is append at EOL. In our buffer, col = len is allowed for insert.
                let row = self.practice_state.vim_buffer.cursor_row;
                self.practice_state.vim_buffer.cursor_col = self.practice_state.vim_buffer.line_len(row);
                self.practice_state.vim_mode = VimMode::Insert;
            }
            KeyCode::Char('o') => {
                self.practice_state.vim_buffer.save_history();
                let row = self.practice_state.vim_buffer.cursor_row;
                self.practice_state.vim_buffer.lines.insert(row + 1, String::new());
                self.practice_state.vim_buffer.cursor_row += 1;
                self.practice_state.vim_buffer.cursor_col = 0;
                self.practice_state.vim_mode = VimMode::Insert;
            }
            KeyCode::Char('O') => {
                self.practice_state.vim_buffer.save_history();
                let row = self.practice_state.vim_buffer.cursor_row;
                self.practice_state.vim_buffer.lines.insert(row, String::new());
                self.practice_state.vim_buffer.cursor_col = 0;
                self.practice_state.vim_mode = VimMode::Insert;
            }

            // Visual & Command
            KeyCode::Char('v') => {
                self.practice_state.vim_mode = VimMode::Visual;
                let pt = (self.practice_state.vim_buffer.cursor_row, self.practice_state.vim_buffer.cursor_col);
                self.practice_state.vim_buffer.selection_start = Some(pt);
            }
            KeyCode::Char(':') => {
                self.practice_state.vim_mode = VimMode::Command;
            }

            // Navigation
            KeyCode::Char('h') | KeyCode::Left => self.practice_state.vim_buffer.move_cursor(MoveDirection::Left),
            KeyCode::Char('j') | KeyCode::Down => self.practice_state.vim_buffer.move_cursor(MoveDirection::Down),
            KeyCode::Char('k') | KeyCode::Up => self.practice_state.vim_buffer.move_cursor(MoveDirection::Up),
            KeyCode::Char('l') | KeyCode::Right => self.practice_state.vim_buffer.move_cursor(MoveDirection::Right),
            
            KeyCode::Char('w') => self.practice_state.vim_buffer.move_word_forward(),
            KeyCode::Char('b') => self.practice_state.vim_buffer.move_word_backward(),
            KeyCode::Char('e') => self.practice_state.vim_buffer.move_word_end(),
            KeyCode::Char('0') => self.practice_state.vim_buffer.move_to_line_start(),
            KeyCode::Char('$') => self.practice_state.vim_buffer.move_to_line_end(),
            KeyCode::Char('^') => self.practice_state.vim_buffer.move_to_non_blank_start(),
            
            // Find char (start pending state)
            KeyCode::Char('f') | KeyCode::Char('F') | KeyCode::Char('t') | KeyCode::Char('T') => {
                if let KeyCode::Char(c) = key.code {
                    self.practice_state.key_buffer.push(c);
                }
            }

            // Editing
            KeyCode::Char('x') => {
                self.practice_state.vim_buffer.save_history();
                let row = self.practice_state.vim_buffer.cursor_row;
                let col = self.practice_state.vim_buffer.cursor_col;
                if col < self.practice_state.vim_buffer.line_len(row) {
                    self.practice_state.vim_buffer.lines[row].remove(col);
                }
            }
            
            // Pending commands (d, y, c, g)
            KeyCode::Char('d') | KeyCode::Char('y') | KeyCode::Char('c') | KeyCode::Char('g') => {
                 if let KeyCode::Char(c) = key.code {
                    self.practice_state.key_buffer.push(c);
                }
            }

            _ => {}
        }

        self.handle_command_completion();
        Ok(())
    }

    fn handle_insert_mode(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            // Exit insert mode
            KeyCode::Esc => {
                self.practice_state.vim_mode = VimMode::Normal;
                // Move cursor back one position
                if self.practice_state.vim_buffer.cursor_col > 0 {
                    self.practice_state.vim_buffer.cursor_col -= 1;
                }
            }

            // Insert characters
            KeyCode::Char(c) => {
                self.practice_state.vim_buffer.insert_char(c);
                // Move cursor after insertion
                self.practice_state.vim_buffer.cursor_col += 1;
            }

            // Backspace
            KeyCode::Backspace => {
                if self.practice_state.vim_buffer.cursor_col > 0 {
                    self.practice_state.vim_buffer.cursor_col -= 1;
                    let row = self.practice_state.vim_buffer.cursor_row;
                    let line = &mut self.practice_state.vim_buffer.lines[row];
                    if !line.is_empty() {
                        line.remove(self.practice_state.vim_buffer.cursor_col);
                    }
                }
            }

            // New line
            KeyCode::Enter => {
                let row = self.practice_state.vim_buffer.cursor_row;
                let col = self.practice_state.vim_buffer.cursor_col;
                let current_line = self.practice_state.vim_buffer.lines[row].clone();
                let (before, after) = current_line.split_at(col);
                self.practice_state.vim_buffer.lines[row] = before.to_string();
                self.practice_state.vim_buffer.lines.insert(row + 1, after.to_string());
                self.practice_state.vim_buffer.cursor_row += 1;
                self.practice_state.vim_buffer.cursor_col = 0;
            }

            // Movement in insert mode (Ctrl keys in real Vim, here we allow arrows)
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

            _ => {}
        }
        Ok(())
    }

    fn handle_visual_mode(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            // Exit visual mode
            KeyCode::Esc | KeyCode::Char('v') => {
                self.practice_state.vim_buffer.selection_start = None;
                self.practice_state.vim_mode = VimMode::Normal;
            }

            // Movement (extends selection)
            KeyCode::Char('h') | KeyCode::Left => self.practice_state.vim_buffer.move_cursor(MoveDirection::Left),
            KeyCode::Char('j') | KeyCode::Down => self.practice_state.vim_buffer.move_cursor(MoveDirection::Down),
            KeyCode::Char('k') | KeyCode::Up => self.practice_state.vim_buffer.move_cursor(MoveDirection::Up),
            KeyCode::Char('l') | KeyCode::Right => self.practice_state.vim_buffer.move_cursor(MoveDirection::Right),
            
            KeyCode::Char('w') => self.practice_state.vim_buffer.move_word_forward(),
            KeyCode::Char('b') => self.practice_state.vim_buffer.move_word_backward(),
            KeyCode::Char('e') => self.practice_state.vim_buffer.move_word_end(),
            KeyCode::Char('0') => self.practice_state.vim_buffer.move_to_line_start(),
            KeyCode::Char('$') => self.practice_state.vim_buffer.move_to_line_end(),
            KeyCode::Char('^') => self.practice_state.vim_buffer.move_to_non_blank_start(),

            // Visual edit commands
            KeyCode::Char('d') | KeyCode::Char('x') => {
                if let Some(start) = self.practice_state.vim_buffer.selection_start {
                    let end = (self.practice_state.vim_buffer.cursor_row, self.practice_state.vim_buffer.cursor_col);
                    self.practice_state.vim_buffer.delete_range(
                        Point { row: start.0, col: start.1 },
                        Point { row: end.0, col: end.1 }
                    );
                }
                self.practice_state.vim_buffer.selection_start = None;
                self.practice_state.vim_mode = VimMode::Normal;
            }
            KeyCode::Char('y') => {
                // Yank selection (simplified)
                if let Some(start) = self.practice_state.vim_buffer.selection_start {
                    let end = (self.practice_state.vim_buffer.cursor_row, self.practice_state.vim_buffer.cursor_col);
                     let text = self.practice_state.vim_buffer.get_range_text(
                        Point { row: start.0, col: start.1 },
                        Point { row: end.0, col: end.1 }
                    );
                    self.practice_state.key_buffer = format!("yanked:{}", text);
                }
                self.practice_state.vim_buffer.selection_start = None;
                self.practice_state.vim_mode = VimMode::Normal;
            }
            KeyCode::Char('c') => {
                if let Some(start) = self.practice_state.vim_buffer.selection_start {
                    let end = (self.practice_state.vim_buffer.cursor_row, self.practice_state.vim_buffer.cursor_col);
                    self.practice_state.vim_buffer.delete_range(
                        Point { row: start.0, col: start.1 },
                        Point { row: end.0, col: end.1 }
                    );
                }
                self.practice_state.vim_buffer.selection_start = None;
                self.practice_state.vim_mode = VimMode::Insert;
            }

            _ => {}
        }
        Ok(())
    }

    fn handle_command_mode(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            // Exit command mode
            KeyCode::Esc => {
                self.practice_state.vim_mode = VimMode::Normal;
                self.practice_state.key_buffer.clear();
            }
            KeyCode::Char('c') => {
                if self.practice_state.key_buffer.is_empty() {
                    self.practice_state.vim_mode = VimMode::Normal;
                } else {
                    self.practice_state.key_buffer.clear();
                }
            }

            // Execute command
            KeyCode::Enter => {
                self.execute_command();
                self.practice_state.vim_mode = VimMode::Normal;
                self.practice_state.key_buffer.clear();
            }

            // Build command
            KeyCode::Char(c) => {
                self.practice_state.key_buffer.push(c);
            }
            KeyCode::Backspace => {
                self.practice_state.key_buffer.pop();
            }

            _ => {}
        }
        Ok(())
    }

    fn handle_command_completion(&mut self) {
        let buf = self.practice_state.key_buffer.clone();
        if buf.len() >= 2 {
            match buf.as_str() {
                "dd" => {
                    // Delete line
                    let row = self.practice_state.vim_buffer.cursor_row;
                    self.practice_state.vim_buffer.lines.remove(row);
                    if self.practice_state.vim_buffer.lines.is_empty() {
                        self.practice_state.vim_buffer.lines.push(String::new());
                    }
                    self.practice_state.vim_buffer.cursor_row = self.practice_state.vim_buffer.cursor_row.min(self.practice_state.vim_buffer.lines.len() - 1);
                    self.practice_state.key_buffer = "d".to_string(); // Store deleted text
                }
                "yy" => {
                    // Yank line
                    let row = self.practice_state.vim_buffer.cursor_row;
                    let line = &self.practice_state.vim_buffer.lines[row];
                    self.practice_state.key_buffer = format!("yanked:{}", line);
                }
                "cc" => {
                    // Change line
                    let row = self.practice_state.vim_buffer.cursor_row;
                    self.practice_state.vim_buffer.lines[row].clear();
                    self.practice_state.vim_buffer.cursor_col = 0;
                    self.practice_state.vim_mode = VimMode::Insert;
                }
                "gg" => {
                    // Go to beginning
                    self.practice_state.vim_buffer.cursor_row = 0;
                    self.practice_state.vim_buffer.cursor_col = 0;
                }
                _ => {}
            }
        } else if buf.len() == 1 {
            // Single char commands reset the buffer unless followed by completion
            let c = buf.chars().next().unwrap();
            if !matches!(c, 'd' | 'y' | 'c' | 'g' | 'r') {
                self.practice_state.key_buffer.clear();
            }
        }
    }

    fn execute_command(&mut self) {
        let cmd = self.practice_state.key_buffer.clone();

        match cmd.as_str() {
            "w" | "write" => {
                // Save (placeholder - in real app would save to file)
            }
            "q" | "quit" => {
                // In practice mode, return to menu; otherwise quit app
                if matches!(
                    self.current_screen,
                    Screen::DailyDrill | Screen::FreePractice | Screen::GuidedLearning
                ) {
                    self.current_screen = Screen::MainMenu;
                    self.practice_state.reset();
                } else {
                    self.should_quit = true;
                }
            }
            "wq" | "x" => {
                // Save and quit
                self.should_quit = true;
            }
            "help" => {
                self.current_screen = Screen::Help;
            }
            "set nu" | "number" => {
                // Line numbers (would affect UI)
            }
            "set nonu" | "nonumber" => {
                // No line numbers
            }
            "d" => {
                // Just 'd' - delete character
                let row = self.practice_state.vim_buffer.cursor_row;
                let col = self.practice_state.vim_buffer.cursor_col;
                let line = &mut self.practice_state.vim_buffer.lines[row];
                if col < line.len() {
                    line.remove(col);
                }
            }
            "y" => {
                // Just 'y' - yank character
                let row = self.practice_state.vim_buffer.cursor_row;
                let col = self.practice_state.vim_buffer.cursor_col;
                let line = &self.practice_state.vim_buffer.lines[row];
                if col < line.len() {
                    let c = line.chars().nth(col).unwrap_or(' ');
                    self.practice_state.key_buffer = format!("yanked:{}", c);
                }
            }
            _ => {
                // Unknown command
            }
        }
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

    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyModifiers, KeyEventKind, KeyEventState};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    #[test]
    fn test_escape_normal_mode() {
        let mut app = App::new().unwrap();
        app.current_screen = Screen::DailyDrill; // Practice mode
        app.practice_state.vim_mode = VimMode::Normal;
        
        // Press Esc
        app.handle_key(key(KeyCode::Esc)).unwrap();
        
        // Should still be in Practice mode (DailyDrill)
        assert_eq!(app.current_screen, Screen::DailyDrill);
        // Should be in Normal mode
        assert_eq!(app.practice_state.vim_mode, VimMode::Normal);
    }

    #[test]
    fn test_visual_mode_no_typing() {
        let mut app = App::new().unwrap();
        app.current_screen = Screen::DailyDrill;
        app.practice_state.vim_mode = VimMode::Visual;
        app.practice_state.vim_buffer.lines = vec!["Hello".to_string()];
        
        // Press 'a' (invalid in Visual)
        app.handle_key(key(KeyCode::Char('a'))).unwrap();
        
        // Buffer should be unchanged
        assert_eq!(app.practice_state.vim_buffer.lines[0], "Hello");
        // Should still be in Visual mode
        assert_eq!(app.practice_state.vim_mode, VimMode::Visual);
    }
}
