use crossterm::{
    cursor::SetCursorStyle,
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

        // Restore cursor shape on exit
        std::io::stdout().execute(SetCursorStyle::DefaultUserShape)?;
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
        // En modo Normal, todos los caracteres son comandos o movimientos
        match key.code {
            // Double Esc to exit to menu
            KeyCode::Esc => {
                let now = std::time::Instant::now();
                if let Some(last_esc) = self.practice_state.last_esc_time {
                    // If less than 1 second since last Esc, exit to menu
                    if now.duration_since(last_esc).as_millis() < 1000 {
                        self.current_screen = Screen::MainMenu;
                        self.practice_state.reset();
                        return Ok(());
                    }
                }
                // Record this Esc press
                self.practice_state.last_esc_time = Some(now);
            }

            // Modo Insert
            KeyCode::Char('i') => {
                self.practice_state.vim_mode = VimMode::Insert;
            }
            KeyCode::Char('a') => {
                // Append: mover cursor una posición a la derecha si es posible
                let line_len = self.practice_state.vim_buffer.lines[self.practice_state.vim_buffer.cursor_row].len();
                if self.practice_state.vim_buffer.cursor_col < line_len {
                    self.practice_state.vim_buffer.cursor_col += 1;
                }
                self.practice_state.vim_mode = VimMode::Insert;
            }
            KeyCode::Char('I') => {
                // Insert at line start
                self.practice_state.vim_buffer.cursor_col = 0;
                self.practice_state.vim_mode = VimMode::Insert;
            }
            KeyCode::Char('A') => {
                // Append at line end
                let line_len = self.practice_state.vim_buffer.lines[self.practice_state.vim_buffer.cursor_row].len();
                self.practice_state.vim_buffer.cursor_col = line_len;
                self.practice_state.vim_mode = VimMode::Insert;
            }

            // New lines
            KeyCode::Char('o') => {
                // New line below
                let _current_line = self.practice_state.vim_buffer.lines[self.practice_state.vim_buffer.cursor_row].clone();
                self.practice_state.vim_buffer.lines.insert(self.practice_state.vim_buffer.cursor_row + 1, String::new());
                self.practice_state.vim_buffer.cursor_row += 1;
                self.practice_state.vim_buffer.cursor_col = 0;
                self.practice_state.vim_mode = VimMode::Insert;
            }
            KeyCode::Char('O') => {
                // New line above
                let row = self.practice_state.vim_buffer.cursor_row;
                self.practice_state.vim_buffer.lines.insert(row, String::new());
                self.practice_state.vim_buffer.cursor_col = 0;
                self.practice_state.vim_mode = VimMode::Insert;
            }

            // Visual mode
            KeyCode::Char('v') => {
                self.practice_state.vim_mode = VimMode::Visual;
            }

            // Command mode
            KeyCode::Char(':') => {
                self.practice_state.vim_mode = VimMode::Command;
            }

            // Movement - basic
            KeyCode::Char('h') => {
                self.practice_state.vim_buffer.move_cursor(MoveDirection::Left);
            }
            KeyCode::Char('j') => {
                self.practice_state.vim_buffer.move_cursor(MoveDirection::Down);
            }
            KeyCode::Char('k') => {
                self.practice_state.vim_buffer.move_cursor(MoveDirection::Up);
            }
            KeyCode::Char('l') => {
                self.practice_state.vim_buffer.move_cursor(MoveDirection::Right);
            }

            // Movement - word
            KeyCode::Char('w') => {
                for _ in 0.. {
                    self.practice_state.vim_buffer.move_cursor(MoveDirection::Right);
                    let line = &self.practice_state.vim_buffer.lines[self.practice_state.vim_buffer.cursor_row];
                    if self.practice_state.vim_buffer.cursor_col >= line.len() {
                        break;
                    }
                    if !line.chars().nth(self.practice_state.vim_buffer.cursor_col).unwrap_or(' ').is_whitespace() {
                        while self.practice_state.vim_buffer.cursor_col < line.len() {
                            let c = line.chars().nth(self.practice_state.vim_buffer.cursor_col).unwrap_or(' ');
                            if c.is_whitespace() {
                                break;
                            }
                            if self.practice_state.vim_buffer.cursor_col + 1 >= line.len() {
                                break;
                            }
                            self.practice_state.vim_buffer.cursor_col += 1;
                        }
                        break;
                    }
                }
            }
            KeyCode::Char('b') => {
                // Move backward word
                self.practice_state.vim_buffer.move_cursor(MoveDirection::Left);
                while self.practice_state.vim_buffer.cursor_col > 0 {
                    let line = &self.practice_state.vim_buffer.lines[self.practice_state.vim_buffer.cursor_row];
                    let c = line.chars().nth(self.practice_state.vim_buffer.cursor_col - 1).unwrap_or(' ');
                    if !c.is_whitespace() {
                        break;
                    }
                    self.practice_state.vim_buffer.cursor_col -= 1;
                }
                while self.practice_state.vim_buffer.cursor_col > 0 {
                    let line = &self.practice_state.vim_buffer.lines[self.practice_state.vim_buffer.cursor_row];
                    let c = line.chars().nth(self.practice_state.vim_buffer.cursor_col - 1).unwrap_or(' ');
                    if c.is_whitespace() {
                        break;
                    }
                    self.practice_state.vim_buffer.cursor_col -= 1;
                }
            }

            // Movement - line
            KeyCode::Char('0') => {
                self.practice_state.vim_buffer.cursor_col = 0;
            }
            KeyCode::Char('$') => {
                let line_len = self.practice_state.vim_buffer.lines[self.practice_state.vim_buffer.cursor_row].len();
                self.practice_state.vim_buffer.cursor_col = line_len.saturating_sub(1).max(0);
            }

            // Movement - file
            KeyCode::Char('g') => {
                // gg - file start (needs another g)
                // We'll implement as part of command buffer
                self.practice_state.key_buffer = "g".to_string();
            }
            KeyCode::Char('G') => {
                // File end
                self.practice_state.vim_buffer.cursor_row = self.practice_state.vim_buffer.lines.len().saturating_sub(1);
                let line_len = self.practice_state.vim_buffer.lines[self.practice_state.vim_buffer.cursor_row].len();
                self.practice_state.vim_buffer.cursor_col = line_len.saturating_sub(1).max(0);
            }

            // Edit commands
            KeyCode::Char('x') => {
                // Delete character under cursor
                let row = self.practice_state.vim_buffer.cursor_row;
                let col = self.practice_state.vim_buffer.cursor_col;
                let line = &mut self.practice_state.vim_buffer.lines[row];
                if col < line.len() {
                    line.remove(col);
                }
            }
            KeyCode::Char('X') => {
                // Delete character before cursor
                let row = self.practice_state.vim_buffer.cursor_row;
                if self.practice_state.vim_buffer.cursor_col > 0 {
                    self.practice_state.vim_buffer.cursor_col -= 1;
                    let line = &mut self.practice_state.vim_buffer.lines[row];
                    line.remove(self.practice_state.vim_buffer.cursor_col);
                }
            }
            KeyCode::Char('d') => {
                // dd - delete line (need another d)
                self.practice_state.key_buffer = "d".to_string();
            }
            KeyCode::Char('D') => {
                // Delete to end of line
                let row = self.practice_state.vim_buffer.cursor_row;
                let col = self.practice_state.vim_buffer.cursor_col;
                let line = &mut self.practice_state.vim_buffer.lines[row];
                let (_, after) = line.split_at(col);
                *line = after.to_string();
            }
            KeyCode::Char('y') => {
                // yy - yank line
                self.practice_state.key_buffer = "y".to_string();
            }
            KeyCode::Char('Y') => {
                // Yank to end of line
                let row = self.practice_state.vim_buffer.cursor_row;
                let col = self.practice_state.vim_buffer.cursor_col;
                let line = &self.practice_state.vim_buffer.lines[row];
                let (_, after) = line.split_at(col);
                // Store in register (simplified)
                self.practice_state.key_buffer = format!("yanked:{}", after);
            }
            KeyCode::Char('p') => {
                // Paste after
                let row = self.practice_state.vim_buffer.cursor_row;
                let yanked = &self.practice_state.key_buffer;
                if yanked.starts_with("yanked:") {
                    let text = yanked.strip_prefix("yanked:").unwrap_or("");
                    if !text.is_empty() {
                        self.practice_state.vim_buffer.lines[row].push_str(text);
                        self.practice_state.vim_buffer.cursor_col = self.practice_state.vim_buffer.lines[row].len().saturating_sub(1);
                    }
                } else if !yanked.is_empty() && yanked != "d" && yanked != "y" {
                    self.practice_state.vim_buffer.lines[row].push_str(yanked);
                }
            }
            KeyCode::Char('P') => {
                // Paste before
                let row = self.practice_state.vim_buffer.cursor_row;
                let yanked = &self.practice_state.key_buffer;
                if yanked.starts_with("yanked:") {
                    let text = yanked.strip_prefix("yanked:").unwrap_or("");
                    if !text.is_empty() {
                        let current = self.practice_state.vim_buffer.lines[row].clone();
                        let col = self.practice_state.vim_buffer.cursor_col;
                        let (before, after) = current.split_at(col);
                        self.practice_state.vim_buffer.lines[row] = format!("{}{}{}", before, text, after);
                        self.practice_state.vim_buffer.cursor_col = col + text.len();
                    }
                }
            }
            KeyCode::Char('u') => {
                // Undo (placeholder - would need full undo system)
                self.practice_state.key_buffer.clear();
            }
            KeyCode::Char('c') => {
                // Change
                self.practice_state.key_buffer = "c".to_string();
            }
            KeyCode::Char('r') => {
                // Replace character
                self.practice_state.key_buffer = "r".to_string();
            }

            // Arrow keys (also work in normal mode)
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

        // Handle command completion (dd, yy, etc.)
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
            KeyCode::Esc => {
                self.practice_state.vim_mode = VimMode::Normal;
            }
            KeyCode::Char('v') => {
                // Exit visual mode
                self.practice_state.vim_mode = VimMode::Normal;
            }

            // Movement (extends selection)
            KeyCode::Char('h') => {
                self.practice_state.vim_buffer.move_cursor(MoveDirection::Left);
            }
            KeyCode::Char('j') => {
                self.practice_state.vim_buffer.move_cursor(MoveDirection::Down);
            }
            KeyCode::Char('k') => {
                self.practice_state.vim_buffer.move_cursor(MoveDirection::Up);
            }
            KeyCode::Char('l') => {
                self.practice_state.vim_buffer.move_cursor(MoveDirection::Right);
            }
            KeyCode::Char('w') => {
                // Word forward
                for _ in 0..5 {
                    self.practice_state.vim_buffer.move_cursor(MoveDirection::Right);
                }
            }
            KeyCode::Char('b') => {
                for _ in 0..5 {
                    self.practice_state.vim_buffer.move_cursor(MoveDirection::Left);
                }
            }
            KeyCode::Char('0') => {
                self.practice_state.vim_buffer.cursor_col = 0;
            }
            KeyCode::Char('$') => {
                let line_len = self.practice_state.vim_buffer.lines[self.practice_state.vim_buffer.cursor_row].len();
                self.practice_state.vim_buffer.cursor_col = line_len.saturating_sub(1).max(0);
            }

            // Visual edit commands
            KeyCode::Char('d') => {
                // Delete selection (simplified: delete to end of line)
                let row = self.practice_state.vim_buffer.cursor_row;
                let col = self.practice_state.vim_buffer.cursor_col;
                let line = &mut self.practice_state.vim_buffer.lines[row];
                let (_, after) = line.split_at(col);
                *line = after.to_string();
                self.practice_state.vim_mode = VimMode::Normal;
            }
            KeyCode::Char('y') => {
                // Yank selection (simplified)
                let row = self.practice_state.vim_buffer.cursor_row;
                let col = self.practice_state.vim_buffer.cursor_col;
                let line = &self.practice_state.vim_buffer.lines[row];
                let (_, after) = line.split_at(col);
                self.practice_state.key_buffer = format!("yanked:{}", after);
                self.practice_state.vim_mode = VimMode::Normal;
            }
            KeyCode::Char('x') => {
                // Delete selection
                let row = self.practice_state.vim_buffer.cursor_row;
                let col = self.practice_state.vim_buffer.cursor_col;
                let line = &mut self.practice_state.vim_buffer.lines[row];
                if col < line.len() {
                    line.remove(col);
                }
                self.practice_state.vim_mode = VimMode::Normal;
            }
            KeyCode::Char('c') => {
                // Change selection (delete and enter insert)
                let row = self.practice_state.vim_buffer.cursor_row;
                let col = self.practice_state.vim_buffer.cursor_col;
                let line = &mut self.practice_state.vim_buffer.lines[row];
                if col < line.len() {
                    line.remove(col);
                }
                self.practice_state.vim_mode = VimMode::Insert;
            }
            KeyCode::Char('r') => {
                // Replace all selected (placeholder)
                self.practice_state.vim_mode = VimMode::Normal;
            }

            // Arrow keys
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
