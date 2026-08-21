use std::{
    collections::{HashMap, HashSet, VecDeque},
    env,
    time::{Duration, Instant},
};

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::{
    curriculum::{Belt, CommandInfo, Exercise, command_catalog, exercises},
    editor::{Editor, EditorEvent, EditorKey, Mode, ViewportCommand},
    progress::{
        ActivityDay, ExerciseRecord, PracticeReport, Profile, ProgressError, ProgressStore, Reward,
        ScoringPolicy, Settings, local_day, unix_now,
    },
    snippets::{SNIPPETS, snippet},
    terminal_appearance::{TerminalTheme, ThemeSource},
    ui::mascot::MascotState,
};

const TOAST_TICKS: u64 = 28;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    Boot,
    Home,
    Academy,
    Practice,
    Progress,
    Settings,
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PracticeKind {
    Daily,
    Guided,
    Free,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcademyPanel {
    Belts,
    Exercises,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct Toast {
    pub text: String,
    pub kind: ToastKind,
    expires_at: u64,
}

#[derive(Debug, Clone)]
pub struct CompletionCard {
    pub exercise_index: usize,
    pub reward: Reward,
    pub actions: u32,
    pub keystrokes: u32,
    pub mistakes: u32,
    pub hints: u32,
    pub elapsed: Duration,
    pub missing_skills: Vec<&'static str>,
    pub violations: Vec<&'static str>,
}

#[derive(Debug, Clone)]
pub struct SessionSummary {
    pub attempted: usize,
    pub completed: usize,
    pub xp: u32,
    pub actions: u32,
    pub elapsed: Duration,
}

#[derive(Debug, Clone)]
pub enum Overlay {
    Welcome,
    QuitConfirm,
    ResetProgressConfirm,
    Completion(CompletionCard),
    SessionSummary(SessionSummary),
}

#[derive(Debug, Clone, Copy, Default)]
pub struct AppOptions {
    pub skip_boot: bool,
    pub force_ascii: bool,
    pub no_animation: bool,
    pub terminal_theme: TerminalTheme,
    pub theme_source: ThemeSource,
}

#[derive(Debug, Clone, Copy)]
pub struct HomeItem {
    pub key: char,
    pub label: &'static str,
    pub description: &'static str,
}

pub const HOME_ITEMS: &[HomeItem] = &[
    HomeItem {
        key: 'd',
        label: "DAILY DRILL",
        description: "Repaso adaptativo de 3–10 minutos",
    },
    HomeItem {
        key: 'g',
        label: "ACADEMIA",
        description: "Campaña guiada por cinturones",
    },
    HomeItem {
        key: 'f',
        label: "SANDBOX",
        description: "Mini‑Vim libre sobre código real",
    },
    HomeItem {
        key: 'p',
        label: "PROGRESO",
        description: "XP, dominio, racha y actividad",
    },
    HomeItem {
        key: 's',
        label: "AJUSTES",
        description: "Pistas, dificultad y accesibilidad",
    },
    HomeItem {
        key: '?',
        label: "REFERENCIA",
        description: "Catálogo buscable de comandos",
    },
    HomeItem {
        key: 'q',
        label: "SALIR",
        description: "Cerrar el dojo con seguridad",
    },
];

pub struct PracticeSession {
    pub kind: PracticeKind,
    pub editor: Editor,
    pub exercise_index: Option<usize>,
    pub queue: Vec<usize>,
    pub queue_position: usize,
    pub started_at: Instant,
    pub exercise_started_at: Instant,
    pub semantic_actions: u32,
    pub keystrokes: u32,
    pub mistakes: u32,
    pub hints: u32,
    pub trace: VecDeque<String>,
    pub notations: Vec<String>,
    pub show_hint: bool,
    pub viewport_top: usize,
    pub snippet_index: usize,
    pub completed_in_run: usize,
    pub current_completed: bool,
    /// True when replaying a result that has already been persisted.
    pub scoring_locked: bool,
    pub xp_in_run: u32,
    pub actions_in_run: u32,
    pub keystrokes_in_run: u32,
    pub status: String,
    insert_action_counted: bool,
    paused_at: Option<Instant>,
}

impl PracticeSession {
    fn from_exercise(
        kind: PracticeKind,
        exercise_index: usize,
        queue: Vec<usize>,
        queue_position: usize,
        started_at: Instant,
    ) -> Self {
        let campaign = exercises();
        let exercise = &campaign[exercise_index];
        let lines = exercise
            .initial_lines
            .iter()
            .map(|line| (*line).to_owned())
            .collect();
        Self {
            kind,
            editor: Editor::new(lines, exercise.start),
            exercise_index: Some(exercise_index),
            queue,
            queue_position,
            started_at,
            exercise_started_at: Instant::now(),
            semantic_actions: 0,
            keystrokes: 0,
            mistakes: 0,
            hints: 0,
            trace: VecDeque::with_capacity(18),
            notations: Vec::new(),
            show_hint: false,
            viewport_top: 0,
            snippet_index: 0,
            completed_in_run: 0,
            current_completed: false,
            scoring_locked: false,
            xp_in_run: 0,
            actions_in_run: 0,
            keystrokes_in_run: 0,
            status: "Kage observa tu ruta…".to_owned(),
            insert_action_counted: false,
            paused_at: None,
        }
    }

    fn free(snippet_index: usize) -> Self {
        let source = snippet(snippet_index);
        Self {
            kind: PracticeKind::Free,
            editor: Editor::new(
                source.lines.iter().map(|line| (*line).to_owned()).collect(),
                crate::editor::Position::new(0, 0),
            ),
            exercise_index: None,
            queue: Vec::new(),
            queue_position: 0,
            started_at: Instant::now(),
            exercise_started_at: Instant::now(),
            semantic_actions: 0,
            keystrokes: 0,
            mistakes: 0,
            hints: 0,
            trace: VecDeque::with_capacity(18),
            notations: Vec::new(),
            show_hint: false,
            viewport_top: 0,
            snippet_index,
            completed_in_run: 0,
            current_completed: false,
            scoring_locked: false,
            xp_in_run: 0,
            actions_in_run: 0,
            keystrokes_in_run: 0,
            status: format!("SANDBOX // {} // sin puntuación", source.name),
            insert_action_counted: false,
            paused_at: None,
        }
    }

    fn push_trace(&mut self, value: String) {
        if value.is_empty() {
            return;
        }
        if self.trace.len() == 18 {
            self.trace.pop_front();
        }
        self.trace.push_back(value);
    }

    fn count_action(&mut self) {
        self.semantic_actions = self.semantic_actions.saturating_add(1);
        self.actions_in_run = self.actions_in_run.saturating_add(1);
    }

    fn count_keystroke(&mut self) {
        self.keystrokes = self.keystrokes.saturating_add(1);
        self.keystrokes_in_run = self.keystrokes_in_run.saturating_add(1);
    }

    fn pause_timing(&mut self) {
        self.paused_at.get_or_insert_with(Instant::now);
    }

    fn resume_timing(&mut self) {
        let Some(paused_at) = self.paused_at.take() else {
            return;
        };
        let pause = paused_at.elapsed();
        self.started_at += pause;
        self.exercise_started_at += pause;
    }
}

pub struct App {
    pub route: Route,
    pub previous_route: Route,
    pub overlay: Option<Overlay>,
    pub should_quit: bool,
    pub tick: u64,
    pub home_index: usize,
    pub academy_panel: AcademyPanel,
    pub belt_index: usize,
    pub exercise_index_in_belt: usize,
    pub settings_index: usize,
    pub help_scroll: usize,
    pub help_query: String,
    pub help_searching: bool,
    pub practice: Option<PracticeSession>,
    pub profile: Profile,
    pub settings: Settings,
    pub records: HashMap<String, ExerciseRecord>,
    pub activity: Vec<ActivityDay>,
    pub achievements: Vec<(String, i64)>,
    pub campaign: Vec<Exercise>,
    pub commands: Vec<CommandInfo>,
    pub mascot_state: MascotState,
    pub toast: Option<Toast>,
    pub startup_warning: Option<String>,
    pub shutdown_warning: Option<String>,
    pub force_ascii: bool,
    pub no_color: bool,
    pub terminal_theme: TerminalTheme,
    pub theme_source: ThemeSource,
    pub viewport_height: usize,
    bell_pending: bool,
    store: ProgressStore,
    mood_expires_at: u64,
    last_input_at: Instant,
}

impl App {
    pub fn new(options: AppOptions) -> Result<Self, ProgressError> {
        let (store, startup_warning) = match ProgressStore::open() {
            Ok(store) => (store, None),
            Err(error) => (
                ProgressStore::in_memory()?,
                Some(format!(
                    "No se pudo abrir el progreso persistente ({error}); esta sesión usará memoria."
                )),
            ),
        };
        Self::with_store(store, startup_warning, options)
    }

    pub fn in_memory(options: AppOptions) -> Result<Self, ProgressError> {
        Self::with_store(ProgressStore::in_memory()?, None, options)
    }

    fn with_store(
        store: ProgressStore,
        startup_warning: Option<String>,
        options: AppOptions,
    ) -> Result<Self, ProgressError> {
        let profile = store.profile()?;
        let mut settings = store.settings()?;
        if options.no_animation {
            settings.animations = false;
        }
        let records = store.exercise_records()?;
        let activity = store.recent_activity(35)?;
        let achievements = store.achievements()?;
        let first_run = profile.total_sessions == 0 && records.is_empty();
        let route = if options.skip_boot || !settings.animations {
            Route::Home
        } else {
            Route::Boot
        };
        Ok(Self {
            route,
            previous_route: Route::Home,
            overlay: if first_run && route == Route::Home {
                Some(Overlay::Welcome)
            } else {
                None
            },
            should_quit: false,
            tick: 0,
            home_index: 0,
            academy_panel: AcademyPanel::Belts,
            belt_index: 0,
            exercise_index_in_belt: 0,
            settings_index: 0,
            help_scroll: 0,
            help_query: String::new(),
            help_searching: false,
            practice: None,
            profile,
            settings,
            records,
            activity,
            achievements,
            campaign: exercises(),
            commands: command_catalog(),
            mascot_state: MascotState::Normal,
            toast: None,
            startup_warning,
            shutdown_warning: None,
            force_ascii: options.force_ascii,
            no_color: env::var_os("NO_COLOR").is_some(),
            terminal_theme: options.terminal_theme,
            theme_source: options.theme_source,
            viewport_height: 12,
            bell_pending: false,
            store,
            mood_expires_at: 0,
            last_input_at: Instant::now(),
        })
    }

    pub fn on_tick(&mut self) {
        self.tick = self.tick.wrapping_add(1);
        if self.route == Route::Boot && (self.tick >= 7 || !self.settings.animations) {
            self.route = Route::Home;
            if self.profile.total_sessions == 0 && self.records.is_empty() {
                self.overlay = Some(Overlay::Welcome);
            }
        }
        if self.mood_expires_at != 0 && self.tick >= self.mood_expires_at {
            self.mascot_state = MascotState::Normal;
            self.mood_expires_at = 0;
        }
        if self
            .toast
            .as_ref()
            .is_some_and(|toast| self.tick >= toast.expires_at)
        {
            self.toast = None;
        }
        if self.route == Route::Practice
            && self.last_input_at.elapsed() >= Duration::from_secs(9)
            && self.mascot_state == MascotState::Normal
        {
            self.mascot_state = MascotState::Thinking;
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if key.kind == KeyEventKind::Release {
            return;
        }
        self.last_input_at = Instant::now();
        if self.mascot_state == MascotState::Thinking {
            self.mascot_state = MascotState::Normal;
        }

        if self.overlay.is_some() {
            self.handle_overlay_key(key);
            return;
        }

        if key.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key.code, KeyCode::Char('q' | 'Q'))
        {
            self.overlay = Some(Overlay::QuitConfirm);
            return;
        }
        match key.code {
            KeyCode::F(1) => {
                if self.route == Route::Practice {
                    self.reveal_hint();
                } else {
                    self.open_help();
                }
                return;
            }
            KeyCode::F(2) if self.route == Route::Practice => {
                self.leave_practice();
                return;
            }
            KeyCode::F(3) => {
                if self.route == Route::Practice
                    && let Some(session) = self.practice.as_mut()
                {
                    session.pause_timing();
                }
                self.open_route(Route::Progress);
                return;
            }
            KeyCode::F(5) if self.route == Route::Practice => {
                self.retry_practice(true);
                return;
            }
            KeyCode::F(6) if self.route == Route::Practice => {
                self.cycle_snippet();
                return;
            }
            _ => {}
        }

        match self.route {
            Route::Boot => {
                self.route = Route::Home;
                if self.profile.total_sessions == 0 && self.records.is_empty() {
                    self.overlay = Some(Overlay::Welcome);
                }
            }
            Route::Home => self.handle_home_key(key),
            Route::Academy => self.handle_academy_key(key),
            Route::Practice => self.handle_practice_key(key),
            Route::Progress => self.handle_simple_back_key(key),
            Route::Settings => self.handle_settings_key(key),
            Route::Help => self.handle_help_key(key),
        }
    }

    /// Bracketed paste is intentionally scoreless. It is accepted only by the
    /// Sandbox so a copied reference solution cannot complete Academy or Daily.
    pub fn handle_paste(&mut self, text: &str) {
        let free_practice = self.route == Route::Practice
            && self
                .practice
                .as_ref()
                .is_some_and(|session| session.kind == PracticeKind::Free);
        if !free_practice {
            if self.route == Route::Practice {
                self.notify(
                    "Paste bloqueado en retos puntuados; usa el Sandbox".into(),
                    ToastKind::Warning,
                );
            }
            return;
        }

        for character in text.chars().take(10_000) {
            let key = match character {
                '\n' => KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
                '\r' => continue,
                character => KeyEvent::new(KeyCode::Char(character), KeyModifiers::NONE),
            };
            self.handle_key(key);
            if self.should_quit || self.route != Route::Practice || self.overlay.is_some() {
                break;
            }
        }
    }

    fn handle_overlay_key(&mut self, key: KeyEvent) {
        let Some(overlay) = self.overlay.clone() else {
            return;
        };
        match overlay {
            Overlay::Welcome => match key.code {
                KeyCode::Enter | KeyCode::Esc | KeyCode::Char(' ') => self.overlay = None,
                KeyCode::Char('g') => {
                    self.overlay = None;
                    self.open_route(Route::Academy);
                }
                _ => {}
            },
            Overlay::QuitConfirm => match key.code {
                KeyCode::Char('y' | 'Y') | KeyCode::Enter => self.confirm_quit(),
                KeyCode::Char('n' | 'N') | KeyCode::Esc => self.overlay = None,
                _ => {}
            },
            Overlay::ResetProgressConfirm => match key.code {
                KeyCode::Char('y' | 'Y') => {
                    self.overlay = None;
                    match self
                        .store
                        .reset_all()
                        .and_then(|()| self.refresh_progress())
                    {
                        Ok(()) => self.notify("Progreso reiniciado".into(), ToastKind::Success),
                        Err(error) => self.notify(error.to_string(), ToastKind::Error),
                    }
                }
                KeyCode::Char('n' | 'N') | KeyCode::Esc => self.overlay = None,
                _ => {}
            },
            Overlay::Completion(_) => match key.code {
                KeyCode::Enter | KeyCode::Char('n') => self.advance_after_completion(),
                KeyCode::Char('r') => {
                    self.overlay = None;
                    self.retry_practice(false);
                }
                KeyCode::Esc => {
                    self.overlay = None;
                    self.leave_practice();
                }
                _ => {}
            },
            Overlay::SessionSummary(_) => match key.code {
                KeyCode::Enter | KeyCode::Esc | KeyCode::Char('q') => {
                    self.overlay = None;
                    self.practice = None;
                    self.open_route(Route::Home);
                }
                KeyCode::Char('d') => {
                    self.overlay = None;
                    self.start_daily();
                }
                _ => {}
            },
        }
    }

    fn handle_home_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down | KeyCode::Tab => {
                self.home_index = (self.home_index + 1) % HOME_ITEMS.len();
            }
            KeyCode::Char('k') | KeyCode::Up | KeyCode::BackTab => {
                self.home_index = self
                    .home_index
                    .checked_sub(1)
                    .unwrap_or(HOME_ITEMS.len() - 1);
            }
            KeyCode::Enter => self.activate_home_item(self.home_index),
            KeyCode::Char(character) => {
                if let Some(index) = HOME_ITEMS.iter().position(|item| item.key == character) {
                    self.home_index = index;
                    self.activate_home_item(index);
                }
            }
            KeyCode::Esc => self.overlay = Some(Overlay::QuitConfirm),
            _ => {}
        }
    }

    fn activate_home_item(&mut self, index: usize) {
        match index {
            0 => self.start_daily(),
            1 => self.open_route(Route::Academy),
            2 => self.start_free(0),
            3 => self.open_route(Route::Progress),
            4 => self.open_route(Route::Settings),
            5 => self.open_help(),
            6 => self.overlay = Some(Overlay::QuitConfirm),
            _ => {}
        }
    }

    fn handle_academy_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => self.open_route(Route::Home),
            KeyCode::Left | KeyCode::Char('h') => self.academy_panel = AcademyPanel::Belts,
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Tab => {
                self.academy_panel = AcademyPanel::Exercises;
            }
            KeyCode::Char('j') | KeyCode::Down => match self.academy_panel {
                AcademyPanel::Belts => {
                    self.belt_index = (self.belt_index + 1).min(Belt::all().len() - 1);
                    self.exercise_index_in_belt = 0;
                }
                AcademyPanel::Exercises => {
                    let count = self.selected_belt_exercise_indices().len();
                    if count > 0 {
                        self.exercise_index_in_belt =
                            (self.exercise_index_in_belt + 1).min(count - 1);
                    }
                }
            },
            KeyCode::Char('k') | KeyCode::Up => match self.academy_panel {
                AcademyPanel::Belts => {
                    self.belt_index = self.belt_index.saturating_sub(1);
                    self.exercise_index_in_belt = 0;
                }
                AcademyPanel::Exercises => {
                    self.exercise_index_in_belt = self.exercise_index_in_belt.saturating_sub(1);
                }
            },
            KeyCode::Char('g') | KeyCode::Home => {
                if self.academy_panel == AcademyPanel::Belts {
                    self.belt_index = 0;
                } else {
                    self.exercise_index_in_belt = 0;
                }
            }
            KeyCode::Char('G') | KeyCode::End => {
                if self.academy_panel == AcademyPanel::Belts {
                    self.belt_index = Belt::all().len() - 1;
                    self.exercise_index_in_belt = 0;
                } else {
                    self.exercise_index_in_belt = self
                        .selected_belt_exercise_indices()
                        .len()
                        .saturating_sub(1);
                }
            }
            KeyCode::Enter => {
                if self.academy_panel == AcademyPanel::Belts {
                    self.academy_panel = AcademyPanel::Exercises;
                } else if !self.belt_unlocked(self.selected_belt()) {
                    self.notify(
                        "Completa el cinturón anterior para abrir este nodo".into(),
                        ToastKind::Warning,
                    );
                } else if let Some(index) = self.selected_exercise_index() {
                    self.start_guided(index);
                }
            }
            _ => {}
        }
    }

    fn handle_simple_back_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                let destination = if matches!(self.previous_route, Route::Boot | Route::Progress) {
                    Route::Home
                } else {
                    self.previous_route
                };
                self.open_route(destination);
            }
            KeyCode::Char('r') if self.route == Route::Progress => {
                if let Err(error) = self.refresh_progress() {
                    self.notify(error.to_string(), ToastKind::Error);
                }
            }
            _ => {}
        }
    }

    fn handle_settings_key(&mut self, key: KeyEvent) {
        const COUNT: usize = 8;
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => self.open_route(Route::Home),
            KeyCode::Char('j') | KeyCode::Down => {
                self.settings_index = (self.settings_index + 1).min(COUNT - 1);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.settings_index = self.settings_index.saturating_sub(1);
            }
            KeyCode::Left | KeyCode::Char('h') => self.adjust_setting(false),
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Enter | KeyCode::Char(' ') => {
                if self.settings_index == 6 {
                    self.overlay = Some(Overlay::ResetProgressConfirm);
                } else if self.settings_index == 7 {
                    self.open_route(Route::Home);
                } else {
                    self.adjust_setting(true);
                }
            }
            _ => {}
        }
    }

    fn adjust_setting(&mut self, forward: bool) {
        match self.settings_index {
            0 => self.settings.hints = !self.settings.hints,
            1 => self.settings.animations = !self.settings.animations,
            2 => self.settings.high_contrast = !self.settings.high_contrast,
            3 => self.settings.sound = !self.settings.sound,
            4 => {
                self.settings.drill_minutes = match (self.settings.drill_minutes, forward) {
                    (3, true) | (10, false) => 5,
                    (5, true) => 10,
                    (5, false) => 3,
                    (_, true) => 3,
                    (_, false) => 10,
                };
            }
            5 => {
                self.settings.difficulty = if forward {
                    (self.settings.difficulty + 1) % 3
                } else {
                    (self.settings.difficulty + 2) % 3
                };
            }
            _ => return,
        }
        match self.store.save_settings(&self.settings) {
            Ok(()) => self.notify("Ajuste guardado".into(), ToastKind::Success),
            Err(error) => self.notify(error.to_string(), ToastKind::Error),
        }
    }

    fn handle_help_key(&mut self, key: KeyEvent) {
        if self.help_searching {
            match key.code {
                KeyCode::Esc => {
                    self.help_searching = false;
                    self.help_query.clear();
                }
                KeyCode::Enter => self.help_searching = false,
                KeyCode::Backspace => {
                    self.help_query.pop();
                    self.help_scroll = 0;
                }
                KeyCode::Char(character) => {
                    self.help_query.push(character);
                    self.help_scroll = 0;
                }
                _ => {}
            }
            return;
        }
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                let destination = self.previous_route;
                self.open_route(if destination == Route::Help {
                    Route::Home
                } else {
                    destination
                });
            }
            KeyCode::Char('/') => self.help_searching = true,
            KeyCode::Char('j') | KeyCode::Down => {
                self.help_scroll = self
                    .help_scroll
                    .saturating_add(1)
                    .min(self.filtered_commands().len().saturating_sub(1));
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.help_scroll = self.help_scroll.saturating_sub(1);
            }
            KeyCode::Char('g') | KeyCode::Home => self.help_scroll = 0,
            _ => {}
        }
    }

    fn handle_practice_key(&mut self, key: KeyEvent) {
        let raw_label = key_label(key);
        let Some(editor_key) = map_editor_key(key) else {
            if matches!(
                key.code,
                KeyCode::Left | KeyCode::Right | KeyCode::Up | KeyCode::Down
            ) {
                if let Some(session) = self.practice.as_mut() {
                    session.count_keystroke();
                    session.count_action();
                    session.mistakes = session.mistakes.saturating_add(1);
                    session.push_trace("[flecha ✗]".into());
                    session.status = "Las flechas hacen ruido. Usa motions de Vim.".into();
                }
                self.set_mood(MascotState::Angry, 5);
            }
            return;
        };

        let (event, old_mode) = {
            let Some(session) = self.practice.as_mut() else {
                return;
            };
            let old_mode = session.editor.mode();
            session.push_trace(raw_label.clone());
            session.count_keystroke();
            (
                session.editor.handle_key(editor_key, self.viewport_height),
                old_mode,
            )
        };
        self.apply_editor_event(event, old_mode, raw_label);
        self.check_exercise_completion();
    }

    fn apply_editor_event(&mut self, event: EditorEvent, old_mode: Mode, raw_label: String) {
        let Some(session) = self.practice.as_mut() else {
            return;
        };
        match event {
            EditorEvent::Executed {
                notation,
                changed,
                moved,
            } => {
                if old_mode != Mode::Insert {
                    session.count_action();
                    if session.editor.mode() == Mode::Insert {
                        session.insert_action_counted = true;
                    }
                }
                if !notation.is_empty() && old_mode != Mode::Insert {
                    session.notations.push(notation.clone());
                    session.status = if changed {
                        format!("{notation} // buffer modificado")
                    } else if moved {
                        format!("{notation} // vector adquirido")
                    } else {
                        format!("{notation} // sin cambio")
                    };
                }
                ensure_cursor_visible(session, self.viewport_height);
            }
            EditorEvent::Pending { notation } => {
                session.status = format!("comando pendiente // {notation}_");
            }
            EditorEvent::Invalid { notation } => {
                session.count_action();
                session.mistakes = session.mistakes.saturating_add(1);
                session.status = format!("señal inválida // {notation}");
                self.set_mood(MascotState::Angry, 5);
            }
            EditorEvent::ModeChanged { from: _, to } => {
                if old_mode == Mode::Insert && to == Mode::Normal {
                    if !session.insert_action_counted {
                        session.count_action();
                    }
                    session.insert_action_counted = false;
                } else if to == Mode::Insert {
                    session.insert_action_counted = false;
                }
                session.notations.push(raw_label);
                session.status = format!("modo {}", mode_name(to));
            }
            EditorEvent::QuitRequested => {
                session.count_action();
                self.leave_practice();
            }
            EditorEvent::Viewport {
                command,
                lines,
                moved,
            } => {
                session.count_action();
                session.notations.push(viewport_notation(command).into());
                if moved {
                    ensure_cursor_visible(session, self.viewport_height);
                } else {
                    session.viewport_top = match command {
                        ViewportCommand::HalfPageDown | ViewportCommand::PageDown => {
                            session.viewport_top.saturating_add(lines)
                        }
                        ViewportCommand::HalfPageUp | ViewportCommand::PageUp => {
                            session.viewport_top.saturating_sub(lines)
                        }
                    };
                }
            }
        }
    }

    fn check_exercise_completion(&mut self) {
        let Some(session) = self.practice.as_ref() else {
            return;
        };
        let Some(exercise_index) = session.exercise_index else {
            return;
        };
        let exercise = &self.campaign[exercise_index];
        if !exercise.goal.is_met(
            session.editor.lines(),
            session.editor.cursor(),
            session.editor.mode(),
        ) {
            return;
        }

        let missing_skills = exercise
            .skills
            .iter()
            .copied()
            .filter(|skill| !skill_observed(skill, &session.notations))
            .collect::<Vec<_>>();
        let elapsed = session.exercise_started_at.elapsed();
        let actions = session.semantic_actions.max(1);
        let keystrokes = session.keystrokes;
        let violations = exercise
            .forbidden
            .iter()
            .copied()
            .filter(|skill| *skill != "flechas" && skill_observed(skill, &session.notations))
            .collect::<Vec<_>>();
        let mistakes = session
            .mistakes
            .saturating_add(u32::try_from(violations.len()).unwrap_or(u32::MAX));
        let hints = session.hints;
        let policy = scoring_policy(session.kind);
        let scoring_locked = session.scoring_locked;
        if !missing_skills.is_empty() {
            if let Some(session) = self.practice.as_mut() {
                session.status = format!(
                    "Meta alcanzada; falta demostrar: {}",
                    missing_skills.join(" · ")
                );
            }
            self.set_mood(MascotState::Thinking, 14);
            return;
        }
        if scoring_locked {
            self.set_mood(MascotState::Happy, 18);
            self.notify(
                "Repetición completada sin alterar XP ni repaso".into(),
                ToastKind::Success,
            );
            self.advance_after_completion();
            return;
        }
        let report = PracticeReport {
            exercise_id: exercise.id,
            skills: exercise.skills,
            success: true,
            semantic_actions: actions,
            mistakes,
            hints,
            elapsed,
            optimal_actions: u32::from(exercise.optimal_actions),
        };
        match self.store.record_result(&report, policy) {
            Ok(reward) => {
                if let Some(session) = self.practice.as_mut() {
                    session.completed_in_run = session.completed_in_run.saturating_add(1);
                    session.xp_in_run = session.xp_in_run.saturating_add(reward.xp);
                    session.current_completed = true;
                }
                let card = CompletionCard {
                    exercise_index,
                    reward,
                    actions,
                    keystrokes,
                    mistakes,
                    hints: report.hints,
                    elapsed,
                    missing_skills,
                    violations,
                };
                self.overlay = Some(Overlay::Completion(card));
                self.set_mood(MascotState::Happy, 18);
                if let Err(error) = self.refresh_progress() {
                    self.notify(error.to_string(), ToastKind::Error);
                }
            }
            Err(error) => self.notify(error.to_string(), ToastKind::Error),
        }
    }

    fn reveal_hint(&mut self) {
        let Some(session) = self.practice.as_mut() else {
            return;
        };
        if session.exercise_index.is_none() {
            session.status = "Sandbox libre: F6 cambia de snippet.".into();
            return;
        }
        if self.settings.hints {
            session.show_hint = true;
            session.hints = session.hints.saturating_add(1);
            session.status = "Pista desbloqueada; observa el objetivo otra vez.".into();
            self.set_mood(MascotState::Thinking, 14);
        } else {
            self.notify(
                "Las pistas están desactivadas en Ajustes".into(),
                ToastKind::Info,
            );
        }
    }

    fn start_daily(&mut self) {
        let queue = self.build_daily_queue();
        if let Some(&first) = queue.first() {
            self.practice = Some(PracticeSession::from_exercise(
                PracticeKind::Daily,
                first,
                queue,
                0,
                Instant::now(),
            ));
            self.open_route(Route::Practice);
        } else {
            self.notify(
                "No hay ejercicios disponibles todavía".into(),
                ToastKind::Warning,
            );
        }
    }

    fn start_guided(&mut self, exercise_index: usize) {
        self.practice = Some(PracticeSession::from_exercise(
            PracticeKind::Guided,
            exercise_index,
            vec![exercise_index],
            0,
            Instant::now(),
        ));
        self.open_route(Route::Practice);
    }

    fn start_free(&mut self, snippet_index: usize) {
        self.practice = Some(PracticeSession::free(snippet_index));
        self.open_route(Route::Practice);
    }

    fn retry_practice(&mut self, record_failure: bool) {
        let Some(old) = self.practice.take() else {
            return;
        };
        if record_failure
            && !old.scoring_locked
            && let Err(error) = self.record_incomplete_attempt(&old)
        {
            self.notify(error.to_string(), ToastKind::Error);
        }
        let scoring_locked = old.scoring_locked || old.current_completed;
        self.practice = match old.exercise_index {
            Some(index) => {
                let mut next = PracticeSession::from_exercise(
                    old.kind,
                    index,
                    old.queue,
                    old.queue_position,
                    old.started_at,
                );
                next.completed_in_run = old.completed_in_run;
                next.xp_in_run = old.xp_in_run;
                next.actions_in_run = old.actions_in_run;
                next.keystrokes_in_run = old.keystrokes_in_run;
                next.current_completed = scoring_locked;
                next.scoring_locked = scoring_locked;
                Some(next)
            }
            None => {
                let mut next = PracticeSession::free(old.snippet_index);
                next.started_at = old.started_at;
                next.actions_in_run = old.actions_in_run;
                next.keystrokes_in_run = old.keystrokes_in_run;
                Some(next)
            }
        };
        self.mascot_state = MascotState::Normal;
    }

    fn cycle_snippet(&mut self) {
        let Some(session) = self.practice.as_ref() else {
            return;
        };
        if session.kind != PracticeKind::Free {
            self.notify("F6 sólo cambia snippets en Sandbox".into(), ToastKind::Info);
            return;
        }
        let next = (session.snippet_index + 1) % SNIPPETS.len();
        let Some(old) = self.practice.take() else {
            return;
        };
        let mut replacement = PracticeSession::free(next);
        replacement.started_at = old.started_at;
        replacement.actions_in_run = old.actions_in_run;
        replacement.keystrokes_in_run = old.keystrokes_in_run;
        self.practice = Some(replacement);
    }

    fn advance_after_completion(&mut self) {
        self.overlay = None;
        let Some(old) = self.practice.take() else {
            return;
        };
        if old.kind != PracticeKind::Daily || old.queue_position + 1 >= old.queue.len() {
            let elapsed = old.started_at.elapsed();
            if let Err(error) = self.store.finish_session(elapsed) {
                self.notify(error.to_string(), ToastKind::Error);
            }
            let summary = SessionSummary {
                attempted: old.queue_position + 1,
                completed: old.completed_in_run,
                xp: old.xp_in_run,
                actions: old.actions_in_run,
                elapsed,
            };
            if old.kind == PracticeKind::Daily {
                self.route = Route::Home;
                self.overlay = Some(Overlay::SessionSummary(summary));
            } else {
                self.route = Route::Academy;
            }
            if let Err(error) = self.refresh_progress() {
                self.notify(error.to_string(), ToastKind::Error);
            }
            return;
        }
        let position = old.queue_position + 1;
        let index = old.queue[position];
        let mut next = PracticeSession::from_exercise(
            PracticeKind::Daily,
            index,
            old.queue,
            position,
            old.started_at,
        );
        next.completed_in_run = old.completed_in_run;
        next.xp_in_run = old.xp_in_run;
        next.actions_in_run = old.actions_in_run;
        next.keystrokes_in_run = old.keystrokes_in_run;
        self.practice = Some(next);
    }

    fn leave_practice(&mut self) {
        let Some(session) = self.practice.take() else {
            self.open_route(Route::Home);
            return;
        };
        if session.keystrokes > 0
            && session.exercise_index.is_some()
            && !session.current_completed
            && let Err(error) = self.record_incomplete_attempt(&session)
        {
            self.notify(error.to_string(), ToastKind::Error);
        }
        if session.keystrokes_in_run > 0
            && let Err(error) = self.store.finish_session(session.started_at.elapsed())
        {
            self.notify(error.to_string(), ToastKind::Error);
        }
        let destination = if session.kind == PracticeKind::Guided {
            Route::Academy
        } else {
            Route::Home
        };
        self.open_route(destination);
        if let Err(error) = self.refresh_progress() {
            self.notify(error.to_string(), ToastKind::Error);
        }
    }

    fn confirm_quit(&mut self) {
        self.overlay = None;
        let mut persistence_errors = Vec::new();
        if let Some(mut session) = self.practice.take() {
            session.resume_timing();
            if session.keystrokes > 0
                && session.exercise_index.is_some()
                && !session.current_completed
                && !session.scoring_locked
                && let Err(error) = self.record_incomplete_attempt(&session)
            {
                persistence_errors.push(error.to_string());
            }
            if session.keystrokes_in_run > 0
                && let Err(error) = self.store.finish_session(session.started_at.elapsed())
            {
                persistence_errors.push(error.to_string());
            }
        }
        if !persistence_errors.is_empty() {
            self.shutdown_warning = Some(persistence_errors.join("; "));
        }
        self.should_quit = true;
    }

    fn record_incomplete_attempt(
        &mut self,
        session: &PracticeSession,
    ) -> Result<(), ProgressError> {
        let Some(index) = session.exercise_index else {
            return Ok(());
        };
        let exercise = &self.campaign[index];
        let observed_skills = exercise
            .skills
            .iter()
            .copied()
            .filter(|skill| skill_observed(skill, &session.notations))
            .collect::<Vec<_>>();
        let forbidden_uses = exercise
            .forbidden
            .iter()
            .filter(|skill| **skill != "flechas" && skill_observed(skill, &session.notations))
            .count();
        let report = PracticeReport {
            exercise_id: exercise.id,
            skills: &observed_skills,
            success: false,
            semantic_actions: session.semantic_actions.max(1),
            mistakes: session
                .mistakes
                .saturating_add(1)
                .saturating_add(u32::try_from(forbidden_uses).unwrap_or(u32::MAX)),
            hints: session.hints,
            elapsed: session.exercise_started_at.elapsed(),
            optimal_actions: u32::from(exercise.optimal_actions),
        };
        self.store
            .record_result(&report, scoring_policy(session.kind))?;
        Ok(())
    }

    fn build_daily_queue(&mut self) -> Vec<usize> {
        let budget_seconds = u32::from(self.settings.drill_minutes).saturating_mul(60);
        let due = self
            .store
            .due_commands(unix_now())
            .unwrap_or_else(|error| {
                self.startup_warning = Some(error.to_string());
                Vec::new()
            })
            .into_iter()
            .collect::<Vec<_>>();
        let mut queue = Vec::new();
        let mut seen = HashSet::new();
        let mut planned_seconds = 0_u32;
        let unlocked = self
            .campaign
            .iter()
            .enumerate()
            .filter(|(_, exercise)| self.belt_unlocked(exercise.belt))
            .map(|(index, _)| index)
            .collect::<Vec<_>>();

        // Preserve the scheduler's due order. A single representative lesson
        // covers each due skill, avoiding a burst of near-duplicate missions.
        for skill in due {
            if planned_seconds >= budget_seconds {
                break;
            }
            if let Some(index) = unlocked.iter().copied().find(|index| {
                !seen.contains(index) && self.campaign[*index].skills.contains(&skill.as_str())
            }) && seen.insert(index)
            {
                queue.push(index);
                planned_seconds =
                    planned_seconds.saturating_add(u32::from(self.campaign[index].estimated_secs));
            }
        }

        // New unlocked material comes next, in curricular order.
        for &index in &unlocked {
            if planned_seconds >= budget_seconds {
                break;
            }
            if self
                .records
                .get(self.campaign[index].id)
                .is_none_or(|record| record.completions == 0)
                && seen.insert(index)
            {
                queue.push(index);
                planned_seconds =
                    planned_seconds.saturating_add(u32::from(self.campaign[index].estimated_secs));
            }
        }

        // General review rotates by day and session instead of always favoring
        // the first campaign nodes.
        let rotation = if unlocked.is_empty() {
            0
        } else {
            (local_day().unsigned_abs() as usize).wrapping_add(self.profile.total_sessions as usize)
                % unlocked.len()
        };
        for step in 0..unlocked.len() {
            if planned_seconds >= budget_seconds {
                break;
            }
            let index = unlocked[(rotation + step) % unlocked.len()];
            if seen.insert(index) {
                queue.push(index);
                planned_seconds =
                    planned_seconds.saturating_add(u32::from(self.campaign[index].estimated_secs));
            }
        }
        queue
    }

    pub fn selected_belt(&self) -> Belt {
        Belt::all()[self.belt_index.min(Belt::all().len() - 1)]
    }

    pub fn selected_belt_exercise_indices(&self) -> Vec<usize> {
        let belt = self.selected_belt();
        self.campaign
            .iter()
            .enumerate()
            .filter_map(|(index, exercise)| (exercise.belt == belt).then_some(index))
            .collect()
    }

    pub fn selected_exercise_index(&self) -> Option<usize> {
        self.selected_belt_exercise_indices()
            .get(self.exercise_index_in_belt)
            .copied()
    }

    pub fn belt_unlocked(&self, belt: Belt) -> bool {
        if belt == Belt::Survivor {
            return true;
        }
        let previous = Belt::all()[belt as usize - 1];
        let lessons = self
            .campaign
            .iter()
            .filter(|exercise| exercise.belt == previous)
            .collect::<Vec<_>>();
        !lessons.is_empty()
            && lessons.iter().all(|exercise| {
                self.records
                    .get(exercise.id)
                    .is_some_and(|record| record.completions > 0)
            })
    }

    pub fn belt_progress(&self, belt: Belt) -> (usize, usize) {
        let lessons = self
            .campaign
            .iter()
            .filter(|exercise| exercise.belt == belt)
            .collect::<Vec<_>>();
        let complete = lessons
            .iter()
            .filter(|exercise| {
                self.records
                    .get(exercise.id)
                    .is_some_and(|record| record.completions > 0)
            })
            .count();
        (complete, lessons.len())
    }

    pub fn current_exercise(&self) -> Option<&Exercise> {
        let index = self.practice.as_ref()?.exercise_index?;
        self.campaign.get(index)
    }

    pub fn filtered_commands(&self) -> Vec<&CommandInfo> {
        let query = self.help_query.to_lowercase();
        self.commands
            .iter()
            .filter(|command| {
                query.is_empty()
                    || command.keys.to_lowercase().contains(&query)
                    || command.name.to_lowercase().contains(&query)
                    || command.description.to_lowercase().contains(&query)
            })
            .collect()
    }

    pub fn set_viewport_height(&mut self, height: usize) {
        self.viewport_height = height.max(3);
        if let Some(session) = self.practice.as_mut() {
            ensure_cursor_visible(session, self.viewport_height);
        }
    }

    pub fn take_bell(&mut self) -> bool {
        std::mem::take(&mut self.bell_pending)
    }

    fn open_help(&mut self) {
        if !matches!(self.route, Route::Help | Route::Progress) {
            self.previous_route = self.route;
        }
        self.route = Route::Help;
        self.help_scroll = 0;
    }

    fn open_route(&mut self, route: Route) {
        if self.route != route && !matches!(self.route, Route::Help | Route::Progress) {
            self.previous_route = self.route;
        }
        self.route = route;
        if route == Route::Practice
            && let Some(session) = self.practice.as_mut()
        {
            session.resume_timing();
        }
        if route == Route::Progress
            && let Err(error) = self.refresh_progress()
        {
            self.notify(error.to_string(), ToastKind::Error);
        }
    }

    fn refresh_progress(&mut self) -> Result<(), ProgressError> {
        self.profile = self.store.profile()?;
        self.records = self.store.exercise_records()?;
        self.activity = self.store.recent_activity(35)?;
        self.achievements = self.store.achievements()?;
        Ok(())
    }

    fn notify(&mut self, text: String, kind: ToastKind) {
        self.toast = Some(Toast {
            text,
            kind,
            expires_at: self.tick.saturating_add(TOAST_TICKS),
        });
    }

    fn set_mood(&mut self, state: MascotState, ticks: u64) {
        if self.settings.sound && matches!(state, MascotState::Happy | MascotState::Angry) {
            self.bell_pending = true;
        }
        self.mascot_state = state;
        self.mood_expires_at = self.tick.saturating_add(ticks);
    }
}

fn ensure_cursor_visible(session: &mut PracticeSession, height: usize) {
    let cursor = session.editor.cursor().row;
    if cursor < session.viewport_top {
        session.viewport_top = cursor;
    } else if cursor >= session.viewport_top.saturating_add(height) {
        session.viewport_top = cursor.saturating_sub(height.saturating_sub(1));
    }
    let max_top = session.editor.lines().len().saturating_sub(height);
    session.viewport_top = session.viewport_top.min(max_top);
}

fn mode_name(mode: Mode) -> &'static str {
    match mode {
        Mode::Normal => "NORMAL",
        Mode::Insert => "INSERT",
        Mode::VisualChar => "VISUAL",
        Mode::VisualLine => "VISUAL LINE",
        Mode::Command => "COMMAND",
        Mode::Search => "SEARCH",
    }
}

fn viewport_notation(command: ViewportCommand) -> &'static str {
    match command {
        ViewportCommand::HalfPageDown => "<C-d>",
        ViewportCommand::HalfPageUp => "<C-u>",
        ViewportCommand::PageDown => "<C-f>",
        ViewportCommand::PageUp => "<C-b>",
    }
}

fn scoring_policy(kind: PracticeKind) -> ScoringPolicy {
    match kind {
        PracticeKind::Daily => ScoringPolicy::DailyDrill,
        PracticeKind::Guided => ScoringPolicy::GuidedLearning,
        PracticeKind::Free => ScoringPolicy::FreePractice,
    }
}

fn map_editor_key(key: KeyEvent) -> Option<EditorKey> {
    match key.code {
        KeyCode::Char(character) if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(EditorKey::Ctrl(character.to_ascii_lowercase()))
        }
        KeyCode::Char(character) => Some(EditorKey::Char(character)),
        KeyCode::Esc => Some(EditorKey::Esc),
        KeyCode::Enter => Some(EditorKey::Enter),
        KeyCode::Backspace => Some(EditorKey::Backspace),
        KeyCode::Delete => Some(EditorKey::Delete),
        KeyCode::Tab => Some(EditorKey::Tab),
        _ => None,
    }
}

fn key_label(key: KeyEvent) -> String {
    match key.code {
        KeyCode::Char(character) if key.modifiers.contains(KeyModifiers::CONTROL) => {
            format!("<C-{}>", character.to_ascii_lowercase())
        }
        KeyCode::Char(' ') => "<Space>".into(),
        KeyCode::Char(character) => character.to_string(),
        KeyCode::Esc => "<Esc>".into(),
        KeyCode::Enter => "<Enter>".into(),
        KeyCode::Backspace => "<BS>".into(),
        KeyCode::Delete => "<Del>".into(),
        KeyCode::Tab => "<Tab>".into(),
        KeyCode::Left => "<Left>".into(),
        KeyCode::Right => "<Right>".into(),
        KeyCode::Up => "<Up>".into(),
        KeyCode::Down => "<Down>".into(),
        _ => "<?>".into(),
    }
}

fn skill_observed(skill: &str, notations: &[String]) -> bool {
    notations.iter().any(|notation| {
        let raw = notation.as_str();
        match skill {
            "move-left" => raw.ends_with('h'),
            "move-down" => raw.ends_with('j'),
            "move-up" => raw.ends_with('k'),
            "move-right" => raw.ends_with('l'),
            "word-forward" => raw.ends_with('w') || raw.ends_with('W'),
            "word-backward" => raw.ends_with('b') || raw.ends_with('B'),
            "word-end" => raw.ends_with('e') || raw.ends_with('E'),
            "insert-before" => raw == "i",
            "append-after" => raw == "a",
            "insert-line-start" => raw == "I",
            "append-line-end" => raw == "A",
            "open-below" => raw == "o",
            "open-above" => raw == "O",
            "escape" => raw == "<Esc>",
            "delete-char" => raw.ends_with('x'),
            "replace-char" => raw.starts_with('r'),
            "find-forward" => raw.starts_with('f'),
            "find-backward" => raw.starts_with('F'),
            "till-forward" => raw.starts_with('t') || raw.contains("dt") || raw.contains("ct"),
            "till-backward" => raw.starts_with('T') || raw.contains("dT") || raw.contains("cT"),
            "repeat-find" => raw.ends_with(';'),
            "reverse-find" => raw.ends_with(','),
            "line-start" => raw.ends_with('0'),
            "first-nonblank" => raw.ends_with('^'),
            "line-end" => raw.ends_with('$'),
            "count" => raw.chars().any(|character| character.is_ascii_digit()),
            "delete-op" => raw.starts_with('d') || raw == "vd" || raw == "Vd",
            "yank-op" | "yank-line" => raw.starts_with('y') || raw == "vy" || raw == "Vy",
            "change-op" => raw.starts_with('c') || raw == "vc" || raw == "Vc",
            "delete-line" => raw.ends_with("dd"),
            "change-line" => raw.ends_with("cc"),
            "paste-after" => raw.ends_with('p'),
            "paste-before" => raw.ends_with('P'),
            "unnamed-register" => {
                raw.starts_with('y') || raw.starts_with('d') || matches!(raw, "p" | "P")
            }
            "visual-char" => raw == "v",
            "visual-line" => raw == "V",
            "search-forward" => raw.starts_with('/'),
            "search-backward" => raw.starts_with('?'),
            "next-match" => raw.ends_with('n'),
            "previous-match" => raw.ends_with('N'),
            "file-start" => raw.ends_with("gg"),
            "file-end" | "goto-line" => raw.ends_with('G'),
            "match-pair" => raw.ends_with('%'),
            "paragraph-next" => raw.ends_with('}'),
            "paragraph-prev" => raw.ends_with('{'),
            "half-page-down" => raw == "<C-d>",
            "half-page-up" => raw == "<C-u>",
            "repeat-change" => raw == ".",
            "inner-word" => raw.contains("iw"),
            "around-word" => raw.contains("aw"),
            "inner-quotes" => raw.contains("i\""),
            "around-quotes" => raw.contains("a\""),
            "inner-parens" => raw.contains("i("),
            "around-parens" => raw.contains("a("),
            "inner-braces" => raw.contains("i{"),
            "around-braces" => raw.contains("a{"),
            "named-register" => raw.starts_with('"'),
            _ => false,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(character: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(character), KeyModifiers::NONE)
    }

    fn solution_keys(notation: &str) -> Vec<KeyEvent> {
        let mut result = Vec::new();
        let mut chars = notation.chars().peekable();
        while let Some(character) = chars.next() {
            if character != '<' {
                result.push(key(character));
                continue;
            }
            let mut token = String::from("<");
            for next in chars.by_ref() {
                token.push(next);
                if next == '>' {
                    break;
                }
            }
            let event = match token.as_str() {
                "<Esc>" => KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
                "<Enter>" => KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
                "<Tab>" => KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE),
                "<BS>" => KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE),
                "<Del>" => KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE),
                "<C-d>" => KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL),
                "<C-u>" => KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL),
                "<C-f>" => KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL),
                "<C-b>" => KeyEvent::new(KeyCode::Char('b'), KeyModifiers::CONTROL),
                _ => panic!("token de prueba desconocido: {token}"),
            };
            result.push(event);
        }
        result
    }

    fn test_app() -> App {
        let mut app = App::in_memory(AppOptions {
            skip_boot: true,
            no_animation: true,
            ..AppOptions::default()
        })
        .unwrap();
        app.overlay = None;
        app
    }

    #[test]
    fn home_navigation_wraps_and_routes() {
        let mut app = App::in_memory(AppOptions {
            skip_boot: true,
            ..AppOptions::default()
        })
        .unwrap();
        app.overlay = None;
        app.handle_key(key('k'));
        assert_eq!(app.home_index, HOME_ITEMS.len() - 1);
        app.handle_key(key('g'));
        assert_eq!(app.route, Route::Academy);
    }

    #[test]
    fn daily_queue_is_finite_and_starts_with_new_content() {
        let mut app = App::in_memory(AppOptions {
            skip_boot: true,
            ..AppOptions::default()
        })
        .unwrap();
        let queue = app.build_daily_queue();
        assert!(!queue.is_empty());
        assert!(queue.len() <= 8);
        assert_eq!(app.campaign[queue[0]].belt, Belt::Survivor);
    }

    #[test]
    fn arrows_are_a_mistake_inside_practice() {
        let mut app = test_app();
        app.start_guided(0);
        app.handle_key(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE));
        assert_eq!(app.practice.as_ref().unwrap().mistakes, 1);
    }

    #[test]
    fn skill_detection_understands_counts_and_compounds() {
        let notation = vec!["d3w".to_owned(), "ci\"".to_owned(), "<C-d>".to_owned()];
        assert!(skill_observed("count", &notation));
        assert!(skill_observed("word-forward", &notation));
        assert!(skill_observed("inner-quotes", &notation));
        assert!(skill_observed("half-page-down", &notation));
    }

    #[test]
    fn every_solution_passes_through_the_learning_pipeline() {
        let mut app = test_app();
        for index in 0..app.campaign.len() {
            let id = app.campaign[index].id;
            let title = app.campaign[index].title;
            let solution = app.campaign[index].solution.to_owned();
            app.overlay = None;
            app.practice = None;
            app.start_guided(index);
            let events = solution_keys(&solution);
            let expected_keystrokes = u32::try_from(events.len()).unwrap();
            for event in events {
                app.handle_key(event);
            }
            let Some(Overlay::Completion(card)) = app.overlay.as_ref() else {
                panic!(
                    "{id} ({title}) no fue aceptado; estado={:?}",
                    app.practice.as_ref().map(|session| &session.status)
                );
            };
            assert_eq!(card.actions, u32::from(app.campaign[index].optimal_actions));
            assert_eq!(card.keystrokes, expected_keystrokes);
        }
    }

    #[test]
    fn reaching_the_target_early_does_not_skip_required_motions() {
        let mut app = test_app();
        let index = app
            .campaign
            .iter()
            .position(|exercise| exercise.id == "ARC-05")
            .unwrap();
        app.start_guided(index);

        app.handle_key(key('G'));
        assert!(app.overlay.is_none());
        assert!(
            app.practice
                .as_ref()
                .unwrap()
                .status
                .contains("falta demostrar")
        );

        app.handle_key(KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL));
        app.handle_key(KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL));
        assert!(matches!(app.overlay, Some(Overlay::Completion(_))));
    }

    #[test]
    fn replaying_a_completed_daily_does_not_score_twice() {
        let mut app = test_app();
        app.start_daily();
        let exercise_index = app.practice.as_ref().unwrap().exercise_index.unwrap();
        let exercise_id = app.campaign[exercise_index].id.to_owned();
        let skill = app.campaign[exercise_index].skills[0];
        let solution = app.campaign[exercise_index].solution.to_owned();
        for event in solution_keys(&solution) {
            app.handle_key(event);
        }
        let xp = app.profile.xp;
        let record = app.records[&exercise_id].clone();
        let card = app.store.review_card(skill).unwrap();

        app.handle_key(key('r'));
        for event in solution_keys(&solution) {
            app.handle_key(event);
        }

        assert_eq!(app.profile.xp, xp);
        assert_eq!(app.records[&exercise_id], record);
        assert_eq!(app.store.review_card(skill).unwrap(), card);
    }

    #[test]
    fn progress_can_pause_and_resume_a_practice_session() {
        let mut app = test_app();
        app.start_free(0);
        app.handle_key(KeyEvent::new(KeyCode::F(3), KeyModifiers::NONE));
        assert_eq!(app.route, Route::Progress);
        assert!(app.practice.is_some());
        app.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert_eq!(app.route, Route::Practice);
        assert!(app.practice.is_some());
    }

    #[test]
    fn confirmed_quit_closes_an_active_session() {
        let mut app = test_app();
        app.start_daily();
        let exercise_id = app.current_exercise().unwrap().id.to_owned();
        app.handle_key(key('h'));
        app.handle_key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL));
        app.handle_key(key('y'));

        assert!(app.should_quit);
        assert!(app.practice.is_none());
        assert_eq!(app.store.profile().unwrap().total_sessions, 1);
        assert_eq!(
            app.store.exercise_records().unwrap()[&exercise_id].attempts,
            1
        );
    }

    #[test]
    fn paste_is_blocked_in_scored_modes_and_available_in_sandbox() {
        let mut app = test_app();
        app.start_guided(0);
        let before = app.practice.as_ref().unwrap().editor.lines_as_strings();
        app.handle_paste("jjlllll");
        assert_eq!(
            app.practice.as_ref().unwrap().editor.lines_as_strings(),
            before
        );
        assert!(app.toast.as_ref().unwrap().text.contains("Paste bloqueado"));

        app.start_free(0);
        app.handle_key(key('i'));
        app.handle_paste("λ猫");
        assert!(app.practice.as_ref().unwrap().editor.lines_as_strings()[0].starts_with("λ猫"));
    }

    #[test]
    fn daily_queue_uses_the_requested_time_budget() {
        fn planned_seconds(minutes: u8) -> u32 {
            let mut app = test_app();
            app.settings.drill_minutes = minutes;
            for exercise in &app.campaign {
                app.records.insert(
                    exercise.id.to_owned(),
                    ExerciseRecord {
                        completions: 1,
                        ..ExerciseRecord::default()
                    },
                );
            }
            let queue = app.build_daily_queue();
            assert_eq!(
                queue.iter().copied().collect::<HashSet<_>>().len(),
                queue.len()
            );
            queue
                .into_iter()
                .map(|index| u32::from(app.campaign[index].estimated_secs))
                .sum()
        }

        let short = planned_seconds(3);
        let medium = planned_seconds(5);
        let long = planned_seconds(10);
        assert!((180..270).contains(&short));
        assert!((300..390).contains(&medium));
        assert!((600..690).contains(&long));
    }

    #[test]
    fn skipping_boot_still_shows_first_run_onboarding() {
        let mut app = App::in_memory(AppOptions::default()).unwrap();
        assert_eq!(app.route, Route::Boot);
        app.handle_key(key(' '));
        assert_eq!(app.route, Route::Home);
        assert!(matches!(app.overlay, Some(Overlay::Welcome)));
    }

    #[test]
    fn transient_screens_do_not_create_a_navigation_cycle() {
        let mut app = test_app();
        app.open_route(Route::Academy);
        app.handle_key(KeyEvent::new(KeyCode::F(3), KeyModifiers::NONE));
        app.handle_key(KeyEvent::new(KeyCode::F(1), KeyModifiers::NONE));
        assert_eq!(app.route, Route::Help);
        app.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert_eq!(app.route, Route::Academy);
    }
}
