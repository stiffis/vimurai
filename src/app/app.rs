use crossterm::{
    cursor::{Hide, Show, SetCursorStyle},
    event::{self, DisableBracketedPaste, EnableBracketedPaste, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};

use ratatui::{prelude::CrosstermBackend, Terminal};

use crate::commands::command_db::CommandDatabase;
use crate::commands::command::Exercise;
use crate::database::UserProgressDB;
use crate::spaced_rep::{SM2Algorithm, Quality, SM2Item};
use crate::engine::mode::{VimMode, Operator};
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
    pub guided_learning_state: GuidedLearningState,
    pub progress_state: ProgressState,
    pub settings_state: SettingsState,
    pub show_quit_confirm: bool,
    pub command_db: CommandDatabase,
    pub progress_db: UserProgressDB,
}

impl App {
    pub fn new() -> Result<Self> {
        Ok(Self {
            should_quit: false,
            current_screen: Screen::MainMenu,
            main_menu_state: MainMenuState::new(),
            practice_state: PracticeState::new(),
            guided_learning_state: GuidedLearningState::new(),
            progress_state: ProgressState::new(),
            settings_state: SettingsState::new(),
            show_quit_confirm: false,
            command_db: CommandDatabase::new(),
            progress_db: UserProgressDB::new()?,
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

    pub fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        // Solo manejar keys en Practice screens
        if matches!(
            self.current_screen,
            Screen::DailyDrill | Screen::FreePractice
        ) {
            self.handle_practice_key(key)?;
        } else {
            // Menu screens
            match self.current_screen {
                Screen::MainMenu => self.handle_main_menu_key(key)?,
                Screen::GuidedLearning => self.handle_guided_learning_key(key)?,
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
                        self.practice_state.mode = PracticeMode::Daily;
                        self.start_daily_drill();
                    }
                    "Guided Learning" => {
                        self.current_screen = Screen::GuidedLearning;
                        self.practice_state.reset();
                        self.guided_learning_state.levels = self.command_db.get_levels();
                    }
                    "Free Practice" => {
                        self.current_screen = Screen::FreePractice;
                        self.practice_state.reset();
                        self.practice_state.mode = PracticeMode::Free;
                    }
                    "Progress" => {
                        self.refresh_progress_ui();
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
                self.practice_state.mode = PracticeMode::Daily;
                self.start_daily_drill();
            }
            KeyCode::Char('g') => {
                self.current_screen = Screen::GuidedLearning;
                self.practice_state.reset();
                self.guided_learning_state.levels = self.command_db.get_levels();
            }
            KeyCode::Char('f') => {
                self.current_screen = Screen::FreePractice;
                self.practice_state.reset();
                self.practice_state.mode = PracticeMode::Free;
            }
            KeyCode::Char('p') => {
                self.refresh_progress_ui();
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

    fn handle_guided_learning_key(&mut self, key: KeyEvent) -> Result<()> {
        if key.code == KeyCode::Esc || key.code == KeyCode::Char('q') {
            self.current_screen = Screen::MainMenu;
            return Ok(());
        }

        match key.code {
            KeyCode::Left | KeyCode::Char('h') => {
                self.guided_learning_state.active_panel = GuidedLearningPanel::Levels;
            }
            KeyCode::Right | KeyCode::Char('l') => {
                if !self.guided_learning_state.levels.is_empty() {
                    self.guided_learning_state.active_panel = GuidedLearningPanel::Exercises;
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                match self.guided_learning_state.active_panel {
                    GuidedLearningPanel::Levels => {
                        if self.guided_learning_state.selected_level_index > 0 {
                            self.guided_learning_state.selected_level_index -= 1;
                            self.guided_learning_state.selected_exercise_index = 0;
                        }
                    }
                    GuidedLearningPanel::Exercises => {
                        if self.guided_learning_state.selected_exercise_index > 0 {
                            self.guided_learning_state.selected_exercise_index -= 1;
                        }
                    }
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                match self.guided_learning_state.active_panel {
                    GuidedLearningPanel::Levels => {
                        if self.guided_learning_state.selected_level_index + 1 < self.guided_learning_state.levels.len() {
                            self.guided_learning_state.selected_level_index += 1;
                            self.guided_learning_state.selected_exercise_index = 0;
                        }
                    }
                    GuidedLearningPanel::Exercises => {
                        if let Some((_, exercises)) = self.guided_learning_state.levels.get(self.guided_learning_state.selected_level_index) {
                            if self.guided_learning_state.selected_exercise_index + 1 < exercises.len() {
                                self.guided_learning_state.selected_exercise_index += 1;
                            }
                        }
                    }
                }
            }
            KeyCode::Enter => {
                if self.guided_learning_state.active_panel == GuidedLearningPanel::Exercises {
                    if let Some((_, exercises)) = self.guided_learning_state.levels.get(self.guided_learning_state.selected_level_index) {
                        if let Some(exercise) = exercises.get(self.guided_learning_state.selected_exercise_index) {
                            self.current_screen = Screen::DailyDrill; 
                            self.practice_state.reset();
                            self.practice_state.mode = PracticeMode::Guided;
                            self.load_exercise(exercise.clone());
                        }
                    }
                } else {
                    self.guided_learning_state.active_panel = GuidedLearningPanel::Exercises;
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn handle_practice_key(&mut self, key: KeyEvent) -> Result<()> {
        // El manejador de teclado siempre debe pasar por aquí, nunca salir directamente al menú
        // El Escape solo cambia de modo, no sale de la pantalla

        match self.practice_state.vim_mode {
            VimMode::Normal => self.handle_normal_mode(key),
            VimMode::Insert => self.handle_insert_mode(key),
            VimMode::Visual => self.handle_visual_mode(key),
            VimMode::Command => self.handle_command_mode(key),
            VimMode::OperatorPending(op) => self.handle_operator_pending_mode(key, op),
        }
    }

    pub fn handle_normal_mode(&mut self, key: KeyEvent) -> Result<()> {
        // If level completed, Enter moves to next
        if self.practice_state.is_correct == Some(true) {
            if key.code == KeyCode::Enter {
                // Load next exercise logic
                if let Some(current) = &self.practice_state.current_exercise {
                    // Find current index and load next
                    let all = self.command_db.get_all_exercises();
                    if let Some(pos) = all.iter().position(|e| e.id == current.id) {
                        if pos + 1 < all.len() {
                            self.load_exercise(all[pos + 1].clone());
                            self.practice_state.exercise_number += 1;
                            return Ok(());
                        } else {
                            self.practice_state.current_instruction = "ALL LEVELS COMPLETED!".to_string();
                             return Ok(());
                        }
                    }
                }
            }
        }

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
                     self.check_exercise_completion();
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

            // Operators (Switch to Pending Mode)
            KeyCode::Char('d') => {
                self.practice_state.vim_mode = VimMode::OperatorPending(Operator::Delete);
            }
            KeyCode::Char('y') => {
                self.practice_state.vim_mode = VimMode::OperatorPending(Operator::Yank);
            }
            KeyCode::Char('c') => {
                self.practice_state.vim_mode = VimMode::OperatorPending(Operator::Change);
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
            KeyCode::Char('x') | KeyCode::Delete => {
                self.practice_state.vim_buffer.save_history();
                let row = self.practice_state.vim_buffer.cursor_row;
                let col = self.practice_state.vim_buffer.cursor_col;
                if col < self.practice_state.vim_buffer.line_len(row) {
                    self.practice_state.vim_buffer.lines[row].remove(col);
                }
            }
            
            // Pending commands (g) - gg is handled by handle_command_completion
            KeyCode::Char('g') => {
                 if let KeyCode::Char(c) = key.code {
                    self.practice_state.key_buffer.push(c);
                }
            }

            _ => {}
        }

        self.handle_command_completion();
        self.check_exercise_completion();
        Ok(())
    }

    pub fn handle_insert_mode(&mut self, key: KeyEvent) -> Result<()> {
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
                self.practice_state.vim_buffer.save_history();
                if self.practice_state.vim_buffer.cursor_col > 0 {
                    self.practice_state.vim_buffer.cursor_col -= 1;
                    let row = self.practice_state.vim_buffer.cursor_row;
                    let line = &mut self.practice_state.vim_buffer.lines[row];
                    // Safety check index
                    if self.practice_state.vim_buffer.cursor_col < line.len() {
                        line.remove(self.practice_state.vim_buffer.cursor_col);
                    }
                } else if self.practice_state.vim_buffer.cursor_row > 0 {
                    // Join with previous line
                    let row = self.practice_state.vim_buffer.cursor_row;
                    let current_line = self.practice_state.vim_buffer.lines.remove(row);
                    let prev_row = row - 1;
                    let new_col = self.practice_state.vim_buffer.lines[prev_row].len();
                    self.practice_state.vim_buffer.lines[prev_row].push_str(&current_line);
                    self.practice_state.vim_buffer.cursor_row = prev_row;
                    self.practice_state.vim_buffer.cursor_col = new_col;
                }
            }
            
            // Delete Key
            KeyCode::Delete => {
                self.practice_state.vim_buffer.save_history();
                let row = self.practice_state.vim_buffer.cursor_row;
                let col = self.practice_state.vim_buffer.cursor_col;
                let line_len = self.practice_state.vim_buffer.lines[row].len();
                
                if col < line_len {
                    self.practice_state.vim_buffer.lines[row].remove(col);
                } else if row + 1 < self.practice_state.vim_buffer.lines.len() {
                    // Join with next line
                    let next_line = self.practice_state.vim_buffer.lines.remove(row + 1);
                    self.practice_state.vim_buffer.lines[row].push_str(&next_line);
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

    pub fn handle_visual_mode(&mut self, key: KeyEvent) -> Result<()> {
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

    pub fn handle_command_mode(&mut self, key: KeyEvent) -> Result<()> {
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

    pub fn handle_operator_pending_mode(&mut self, key: KeyEvent, op: Operator) -> Result<()> {
        if key.code == KeyCode::Esc {
            self.practice_state.vim_mode = VimMode::Normal;
            self.practice_state.key_buffer.clear();
            return Ok(());
        }

        // 1. Check for pending char search (e.g., user pressed 'd', then 'f', waiting for char)
        let pending_motion = self.practice_state.key_buffer.clone();
        if !pending_motion.is_empty() {
             let first_char = pending_motion.chars().next().unwrap();
             if matches!(first_char, 'f' | 'F' | 't' | 'T') && pending_motion.len() == 1 {
                 if let KeyCode::Char(target) = key.code {
                     let forward = matches!(first_char, 'f' | 't');
                     let inclusive = matches!(first_char, 'f' | 'F');
                     
                     // Simulate find
                     let mut end = self.practice_state.vim_buffer.simulate_motion(|b| {
                         b.find_char_in_line(target, forward, inclusive);
                     });
                     
                     // Adjust for inclusive motions (f, F)
                     if inclusive {
                         end.col += 1;
                     }
                     
                     self.execute_operator(op, end);
                     self.practice_state.key_buffer.clear();
                     return Ok(());
                 }
             }
        }

        // 2. Handle double operator (dd, yy, cc)
        match (op, key.code) {
            (Operator::Delete, KeyCode::Char('d')) => {
                // dd logic
                self.practice_state.vim_buffer.save_history();
                let row = self.practice_state.vim_buffer.cursor_row;
                if row < self.practice_state.vim_buffer.lines.len() {
                    self.practice_state.vim_buffer.lines.remove(row);
                    if self.practice_state.vim_buffer.lines.is_empty() {
                        self.practice_state.vim_buffer.lines.push(String::new());
                    }
                    let len = self.practice_state.vim_buffer.lines.len();
                    if self.practice_state.vim_buffer.cursor_row >= len {
                        self.practice_state.vim_buffer.cursor_row = len.saturating_sub(1);
                    }
                }
                self.practice_state.vim_mode = VimMode::Normal;
                self.check_exercise_completion();
                return Ok(());
            }
            (Operator::Yank, KeyCode::Char('y')) => {
                // yy logic
                let row = self.practice_state.vim_buffer.cursor_row;
                if let Some(line) = self.practice_state.vim_buffer.lines.get(row) {
                    self.practice_state.key_buffer = format!("yanked:{}", line);
                }
                self.practice_state.vim_mode = VimMode::Normal;
                self.check_exercise_completion();
                return Ok(());
            }
            (Operator::Change, KeyCode::Char('c')) => {
                // cc logic
                self.practice_state.vim_buffer.save_history();
                let row = self.practice_state.vim_buffer.cursor_row;
                if row < self.practice_state.vim_buffer.lines.len() {
                    self.practice_state.vim_buffer.lines[row].clear();
                    self.practice_state.vim_buffer.cursor_col = 0;
                }
                self.practice_state.vim_mode = VimMode::Insert;
                self.check_exercise_completion();
                return Ok(());
            }
            _ => {}
        }

        // 3. Handle start of pending motion (f, t...)
        match key.code {
            KeyCode::Char('f') | KeyCode::Char('F') | KeyCode::Char('t') | KeyCode::Char('T') => {
                if let KeyCode::Char(c) = key.code {
                    self.practice_state.key_buffer.push(c);
                    return Ok(());
                }
            }
            _ => {}
        }

        // 4. Handle simple motions
        let motion_result = match key.code {
            KeyCode::Char('w') => {
                if op == Operator::Change {
                    Some((self.practice_state.vim_buffer.simulate_motion(|b| b.move_word_end()), true))
                } else {
                    Some((self.practice_state.vim_buffer.simulate_motion(|b| b.move_word_forward()), false))
                }
            }
            KeyCode::Char('b') => Some((self.practice_state.vim_buffer.simulate_motion(|b| b.move_word_backward()), false)),
            KeyCode::Char('e') => Some((self.practice_state.vim_buffer.simulate_motion(|b| b.move_word_end()), true)),
            KeyCode::Char('$') => Some((self.practice_state.vim_buffer.simulate_motion(|b| b.move_to_line_end()), true)),
            KeyCode::Char('0') => Some((self.practice_state.vim_buffer.simulate_motion(|b| b.move_to_line_start()), false)),
            KeyCode::Char('^') => Some((self.practice_state.vim_buffer.simulate_motion(|b| b.move_to_non_blank_start()), false)),
            KeyCode::Char('h') | KeyCode::Left => Some((self.practice_state.vim_buffer.simulate_motion(|b| b.move_cursor(MoveDirection::Left)), false)),
            KeyCode::Char('l') | KeyCode::Right => Some((self.practice_state.vim_buffer.simulate_motion(|b| b.move_cursor(MoveDirection::Right)), false)),
            KeyCode::Char('j') | KeyCode::Down => Some((self.practice_state.vim_buffer.simulate_motion(|b| b.move_cursor(MoveDirection::Down)), true)),
            KeyCode::Char('k') | KeyCode::Up => Some((self.practice_state.vim_buffer.simulate_motion(|b| b.move_cursor(MoveDirection::Up)), true)),
            _ => None,
        };

        if let Some((mut end, is_inclusive)) = motion_result {
            if is_inclusive {
                end.col += 1;
            }
            self.execute_operator(op, end);
        } else {
            // Cancel if unknown key and not pending
            if self.practice_state.key_buffer.is_empty() {
                self.practice_state.vim_mode = VimMode::Normal;
            }
        }

        Ok(())
    }

    fn execute_operator(&mut self, op: Operator, end: Point) {
        let start = Point { 
            row: self.practice_state.vim_buffer.cursor_row, 
            col: self.practice_state.vim_buffer.cursor_col 
        };

        match op {
            Operator::Delete => {
                self.practice_state.vim_buffer.delete_range(start, end);
                self.practice_state.vim_mode = VimMode::Normal;
            }
            Operator::Yank => {
                let text = self.practice_state.vim_buffer.get_range_text(start, end);
                self.practice_state.key_buffer = format!("yanked:{}", text);
                self.practice_state.vim_mode = VimMode::Normal;
            }
            Operator::Change => {
                self.practice_state.vim_buffer.delete_range(start, end);
                self.practice_state.vim_mode = VimMode::Insert;
            }
        }
        self.check_exercise_completion();
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
            // IMPORTANT: keep alive operator pending chars if logic depends on them, 
            // but in operator pending mode, this buffer logic is separate. 
            // However, Normal mode still uses this for dd/yy/cc/gg.
            // Also f/t pending in normal mode.
            if !matches!(c, 'd' | 'y' | 'c' | 'g' | 'r' | 'f' | 'F' | 't' | 'T') {
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
                    if self.practice_state.mode == PracticeMode::Guided {
                        self.current_screen = Screen::GuidedLearning;
                    } else {
                        self.current_screen = Screen::MainMenu;
                    }
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
        // 1. Try to get due exercises
        let due_ids = self.progress_db.get_due_commands().unwrap_or_default();
        let all_exercises = self.command_db.get_all_exercises();

        if let Some(due_id) = due_ids.first() {
            if let Some(ex) = all_exercises.iter().find(|e| &e.id == due_id) {
                self.load_exercise(ex.clone());
                return;
            }
        }

        // 2. If nothing due, find the first exercise with 0 progress
        for ex in all_exercises {
            let progress = self.progress_db.get_command_progress(&ex.id).unwrap_or(None);
            if progress.is_none() {
                self.load_exercise(ex.clone());
                return;
            }
        }

        // 3. Fallback to first if everything is done but not due
        if let Some(first) = all_exercises.first() {
            self.load_exercise((*first).clone());
        } else {
            self.practice_state.current_instruction = "No exercises available!".to_string();
        }
    }

    fn load_exercise(&mut self, exercise: Exercise) {
        self.practice_state.vim_buffer.lines = exercise.initial_lines.clone();
        self.practice_state.vim_buffer.cursor_row = exercise.initial_cursor.0;
        self.practice_state.vim_buffer.cursor_col = exercise.initial_cursor.1;
        self.practice_state.current_instruction = format!("{}: {}", exercise.title, exercise.description);
        self.practice_state.hint = format!("Context: {} | Hint: {}", exercise.context, exercise.hint);
        self.practice_state.current_exercise = Some(exercise);
        self.practice_state.is_correct = None;
    }

    pub fn check_exercise_completion(&mut self) {
        if self.practice_state.is_correct == Some(true) {
            return; // Already completed
        }

        if let Some(exercise) = &self.practice_state.current_exercise {
            let mut correct = true;

            // Check content if expected
            if let Some(expected_lines) = &exercise.expected_lines {
                if self.practice_state.vim_buffer.lines != *expected_lines {
                    correct = false;
                }
            }

            // Check cursor if expected
            if let Some(expected_cursor) = exercise.expected_cursor {
                if self.practice_state.vim_buffer.cursor_row != expected_cursor.0 
                   || self.practice_state.vim_buffer.cursor_col != expected_cursor.1 {
                    correct = false;
                }
            }

            if correct {
                self.practice_state.is_correct = Some(true);
                self.practice_state.current_instruction = "SUCCESS! Press Enter for next level.".to_string();
                
                // Record result in DB
                let id = exercise.id.clone();
                self.record_exercise_result(id, Quality::Perfect);
                self.refresh_progress_ui();
            }
        }
    }

    fn refresh_progress_ui(&mut self) {
        if let Ok(stats) = self.progress_db.get_stats() {
            self.progress_state.level = stats.level;
            self.progress_state.xp = stats.xp;
            self.progress_state.commands_mastered = stats.commands_mastered;
            self.progress_state.commands_learning = stats.commands_learning;
            self.progress_state.streak_days = stats.streak_days;
            self.progress_state.total_sessions = stats.total_sessions;
        }
    }

    fn record_exercise_result(&mut self, exercise_id: String, quality: Quality) {
        let now = chrono::Utc::now().timestamp() as u64;
        
        // 1. Get current progress from DB
        let current_progress = self.progress_db.get_command_progress(&exercise_id).unwrap_or(None);
        
        // 2. Convert to SM2Item
        let sm2_item = match current_progress {
            Some(p) => SM2Item {
                repetition: p.repetition,
                interval: p.interval_days,
                ease_factor: p.ease_factor,
                quality: p.quality,
                next_review: p.next_review,
            },
            None => SM2Item::default(),
        };
        
        // 3. Calculate next review
        let updated = SM2Algorithm::calculate_next_review(sm2_item, quality, now);
        
        // 4. Save to DB
        let _ = self.progress_db.update_command_progress(
            &exercise_id,
            updated.repetition,
            updated.interval,
            updated.ease_factor,
            updated.quality,
            updated.next_review,
            updated.repetition > 5 // Heuristic for mastered
        );
        
        // 5. Add XP
        let _ = self.progress_db.add_xp(10);
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
