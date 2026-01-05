use crate::engine::{mode::VimMode, vim_buffer::VimBuffer};
use crate::commands::command::Exercise;

/// Enum representing all possible screens in the app
#[derive(Clone, Debug, PartialEq)]
pub enum Screen {
    MainMenu,
    DailyDrill,
    GuidedLearning,
    FreePractice,
    Progress,
    Settings,
    Help,
}

/// State for the main menu screen
#[derive(Clone, Debug)]
pub struct MainMenuState {
    pub selected_index: usize,
    pub items: Vec<MenuItem>,
}

#[derive(Clone, Debug)]
pub struct MenuItem {
    pub title: String,
    pub description: String,
    pub shortcut: String,
}

impl Default for MainMenuState {
    fn default() -> Self {
        Self::new()
    }
}

impl MainMenuState {
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            items: vec![
                MenuItem {
                    title: "Daily Drill".to_string(),
                    description: "Practice commands due for review (3-5 min)".to_string(),
                    shortcut: "d".to_string(),
                },
                MenuItem {
                    title: "Guided Learning".to_string(),
                    description: "Learn Vim commands step by step".to_string(),
                    shortcut: "g".to_string(),
                },
                MenuItem {
                    title: "Free Practice".to_string(),
                    description: "Practice freely in a Vim buffer".to_string(),
                    shortcut: "f".to_string(),
                },
                MenuItem {
                    title: "Progress".to_string(),
                    description: "View your stats and achievements".to_string(),
                    shortcut: "p".to_string(),
                },
                MenuItem {
                    title: "Settings".to_string(),
                    description: "Configure app preferences".to_string(),
                    shortcut: "s".to_string(),
                },
                MenuItem {
                    title: "Help".to_string(),
                    description: "Show keyboard shortcuts".to_string(),
                    shortcut: "?".to_string(),
                },
                MenuItem {
                    title: "Quit".to_string(),
                    description: "Exit Vimurai".to_string(),
                    shortcut: "q".to_string(),
                },
            ],
        }
    }

    pub fn next(&mut self) {
        self.selected_index = (self.selected_index + 1) % self.items.len();
    }

    pub fn previous(&mut self) {
        self.selected_index = if self.selected_index == 0 {
            self.items.len() - 1
        } else {
            self.selected_index - 1
        };
    }

    pub fn current_item(&self) -> &MenuItem {
        &self.items[self.selected_index]
    }
}

/// State for practice screens
#[derive(Clone, Debug)]
pub struct PracticeState {
    pub vim_buffer: VimBuffer,
    pub vim_mode: VimMode,
    pub current_instruction: String,
    pub hint: String,
    pub start_time: std::time::Instant,
    pub expected_keys: Vec<String>,
    pub key_buffer: String,
    pub is_correct: Option<bool>,
    pub exercise_number: usize,
    pub total_exercises: usize,
    pub last_esc_time: Option<std::time::Instant>,
    pub current_exercise: Option<Exercise>,
}

impl Default for PracticeState {
    fn default() -> Self {
        Self::new()
    }
}

impl PracticeState {
    pub fn new() -> Self {
        Self {
            vim_buffer: VimBuffer::new(),
            vim_mode: VimMode::Normal,
            current_instruction: "".to_string(),
            hint: "".to_string(),
            start_time: std::time::Instant::now(),
            expected_keys: Vec::new(),
            key_buffer: String::new(),
            is_correct: None,
            exercise_number: 1,
            total_exercises: 10,
            last_esc_time: None,
            current_exercise: None,
        }
    }

    pub fn reset(&mut self) {
        self.vim_buffer = VimBuffer::new();
        self.vim_mode = VimMode::Normal;
        self.key_buffer.clear();
        self.is_correct = None;
        self.start_time = std::time::Instant::now();
        self.last_esc_time = None;
        self.current_exercise = None;
    }

    pub fn elapsed_time(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }
}

/// State for progress screen
#[derive(Clone, Debug)]
pub struct ProgressState {
    pub level: u32,
    pub xp: u64,
    pub xp_for_next_level: u64,
    pub commands_mastered: u32,
    pub commands_learning: u32,
    pub streak_days: u32,
    pub total_sessions: u64,
    pub achievements_unlocked: usize,
    pub recent_activity: Vec<ActivityItem>,
}

#[derive(Clone, Debug)]
pub struct ActivityItem {
    pub description: String,
    pub timestamp: String,
    pub xp_gained: u64,
}

impl Default for ProgressState {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressState {
    pub fn new() -> Self {
        Self {
            level: 1,
            xp: 0,
            xp_for_next_level: 100,
            commands_mastered: 0,
            commands_learning: 25,
            streak_days: 0,
            total_sessions: 0,
            achievements_unlocked: 0,
            recent_activity: Vec::new(),
        }
    }

    pub fn level_progress(&self) -> f64 {
        self.xp as f64 / self.xp_for_next_level as f64
    }
}

/// State for settings screen
#[derive(Clone, Debug)]
pub struct SettingsState {
    pub selected_index: usize,
    pub hints_enabled: bool,
    pub drill_duration: DrillDuration,
    pub difficulty: Difficulty,
    pub sound_enabled: bool,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum DrillDuration {
    Short = 3,  // 3 minutes
    Medium = 5, // 5 minutes
    Long = 10,  // 10 minutes
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Beginner,
    Intermediate,
    Advanced,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self::new()
    }
}

impl SettingsState {
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            hints_enabled: true,
            drill_duration: DrillDuration::Medium,
            difficulty: Difficulty::Beginner,
            sound_enabled: false,
        }
    }
}
