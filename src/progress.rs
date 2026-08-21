//! Persistent progress, settings and the SM-2 review scheduler.
//!
//! The database is deliberately isolated from the editor. Tests can use an
//! in-memory store, so exercising the curriculum never touches a real profile.

use chrono::{Datelike, Local};
use rusqlite::{Connection, OptionalExtension, params};
use std::{
    collections::{HashMap, HashSet},
    env, fmt, fs,
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const DAY_SECONDS: i64 = 86_400;
const SCHEMA_VERSION: i64 = 2;

#[derive(Debug)]
pub enum ProgressError {
    Io(std::io::Error),
    Database(rusqlite::Error),
    UnsupportedSchema(i64),
}

impl fmt::Display for ProgressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "error de archivos: {error}"),
            Self::Database(error) => write!(f, "error de progreso: {error}"),
            Self::UnsupportedSchema(version) => {
                write!(f, "la base de progreso usa una versión futura ({version})")
            }
        }
    }
}

impl std::error::Error for ProgressError {}

impl From<std::io::Error> for ProgressError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<rusqlite::Error> for ProgressError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Database(value)
    }
}

pub type Result<T> = std::result::Result<T, ProgressError>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReviewCard {
    pub repetitions: u32,
    pub interval_days: f64,
    pub ease_factor: f64,
    pub last_quality: u8,
    pub next_due: i64,
    pub attempts: u32,
    pub successes: u32,
}

impl Default for ReviewCard {
    fn default() -> Self {
        Self {
            repetitions: 0,
            interval_days: 0.0,
            ease_factor: 2.5,
            last_quality: 0,
            next_due: 0,
            attempts: 0,
            successes: 0,
        }
    }
}

/// Canonical SM-2 scheduling using qualities from 0 (blackout) to 5 (perfect).
#[must_use]
pub fn schedule_review(card: ReviewCard, quality: u8, now: i64) -> ReviewCard {
    let quality = quality.min(5);
    let q = f64::from(quality);
    let mut next = card;
    next.attempts = next.attempts.saturating_add(1);
    next.last_quality = quality;

    if quality < 3 {
        next.repetitions = 0;
        next.interval_days = 1.0;
    } else {
        next.successes = next.successes.saturating_add(1);
        next.repetitions = next.repetitions.saturating_add(1);
        next.interval_days = match next.repetitions {
            1 => 1.0,
            2 => 6.0,
            _ => (card.interval_days.max(1.0) * card.ease_factor).max(1.0),
        };
    }

    let adjustment = 0.1 - (5.0 - q) * (0.08 + (5.0 - q) * 0.02);
    next.ease_factor = (card.ease_factor + adjustment).max(1.3);
    let seconds = (next.interval_days * DAY_SECONDS as f64).round() as i64;
    next.next_due = now.saturating_add(seconds.max(DAY_SECONDS));
    next
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    pub hints: bool,
    pub animations: bool,
    pub high_contrast: bool,
    pub sound: bool,
    pub drill_minutes: u8,
    /// 0 = relaxed, 1 = balanced, 2 = strict.
    pub difficulty: u8,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            hints: true,
            animations: true,
            high_contrast: false,
            sound: false,
            drill_minutes: 5,
            difficulty: 1,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Profile {
    pub level: u32,
    pub xp: u64,
    pub streak_days: u32,
    pub best_streak: u32,
    pub total_sessions: u64,
    pub total_actions: u64,
    pub successful_actions: u64,
    pub total_practice_seconds: u64,
    pub commands_mastered: u32,
    pub commands_learning: u32,
}

impl Profile {
    #[must_use]
    pub fn accuracy_percent(&self) -> u16 {
        if self.total_actions == 0 {
            0
        } else {
            ((self.successful_actions.saturating_mul(100) / self.total_actions).min(100)) as u16
        }
    }

    #[must_use]
    pub fn xp_into_level(&self) -> u64 {
        self.xp % 250
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExerciseRecord {
    pub attempts: u32,
    pub completions: u32,
    pub stars: u8,
    pub best_actions: u32,
    pub best_millis: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivityDay {
    pub day: i64,
    pub sessions: u32,
    pub actions: u32,
    pub successes: u32,
    pub seconds: u32,
    pub xp: u32,
}

#[derive(Debug, Clone)]
pub struct PracticeReport<'a> {
    pub exercise_id: &'a str,
    pub skills: &'a [&'a str],
    pub success: bool,
    pub semantic_actions: u32,
    pub mistakes: u32,
    pub hints: u32,
    pub elapsed: Duration,
    pub optimal_actions: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reward {
    pub xp: u32,
    pub stars: u8,
    pub quality: u8,
    pub first_completion: bool,
}

/// Controls which persistent systems a practice result is allowed to affect.
///
/// Guided lessons keep their own completion history, but they must not make an
/// already-seen review card easier or harder. Free practice is deliberately
/// scoreless; its elapsed time can still be recorded by finishing the session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScoringPolicy {
    DailyDrill,
    GuidedLearning,
    FreePractice,
}

#[derive(Debug)]
pub struct ProgressStore {
    connection: Connection,
    path: Option<PathBuf>,
}

impl ProgressStore {
    pub fn open() -> Result<Self> {
        let directory = env::var_os("VIMURAI_DATA_DIR")
            .map(PathBuf::from)
            .or_else(|| dirs::data_local_dir().map(|path| path.join("vimurai")))
            .unwrap_or_else(|| PathBuf::from(".vimurai"));
        fs::create_dir_all(&directory)?;
        Self::open_path(directory.join("progress.db"))
    }

    pub fn open_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)?;
        }
        let connection = Connection::open(&path)?;
        let mut store = Self {
            connection,
            path: Some(path),
        };
        store.configure()?;
        store.migrate()?;
        Ok(store)
    }

    pub fn in_memory() -> Result<Self> {
        let connection = Connection::open_in_memory()?;
        let mut store = Self {
            connection,
            path: None,
        };
        store.configure()?;
        store.migrate()?;
        Ok(store)
    }

    fn configure(&mut self) -> Result<()> {
        self.connection.busy_timeout(Duration::from_secs(2))?;
        self.connection.pragma_update(None, "foreign_keys", "ON")?;
        if self.path.is_some() {
            let _: String =
                self.connection
                    .pragma_update_and_check(None, "journal_mode", "WAL", |row| row.get(0))?;
        }
        Ok(())
    }

    fn migrate(&mut self) -> Result<()> {
        let version: i64 = self
            .connection
            .pragma_query_value(None, "user_version", |row| row.get(0))?;
        if version > SCHEMA_VERSION {
            return Err(ProgressError::UnsupportedSchema(version));
        }

        let transaction = self.connection.transaction()?;
        let legacy_achievements = table_exists(&transaction, "achievements")?
            && achievement_timestamp_type(&transaction)?
                .is_some_and(|column_type| !column_type.to_ascii_uppercase().contains("INT"));
        if legacy_achievements {
            transaction
                .execute_batch("ALTER TABLE achievements RENAME TO achievements_legacy_v0;")?;
        }

        transaction.execute_batch(
            "CREATE TABLE IF NOT EXISTS profile (
                 id INTEGER PRIMARY KEY CHECK (id = 1),
                 xp INTEGER NOT NULL DEFAULT 0 CHECK (xp >= 0),
                 streak_days INTEGER NOT NULL DEFAULT 0 CHECK (streak_days >= 0),
                 best_streak INTEGER NOT NULL DEFAULT 0 CHECK (best_streak >= 0),
                 total_sessions INTEGER NOT NULL DEFAULT 0 CHECK (total_sessions >= 0),
                 total_actions INTEGER NOT NULL DEFAULT 0 CHECK (total_actions >= 0),
                 successful_actions INTEGER NOT NULL DEFAULT 0 CHECK (successful_actions >= 0),
                 total_practice_seconds INTEGER NOT NULL DEFAULT 0 CHECK (total_practice_seconds >= 0),
                 last_practice_day INTEGER
             );
             INSERT OR IGNORE INTO profile (id) VALUES (1);

             CREATE TABLE IF NOT EXISTS reviews (
                 command_id TEXT PRIMARY KEY,
                 repetitions INTEGER NOT NULL DEFAULT 0 CHECK (repetitions >= 0),
                 interval_days REAL NOT NULL DEFAULT 0 CHECK (interval_days >= 0),
                 ease_factor REAL NOT NULL DEFAULT 2.5 CHECK (ease_factor >= 1.3),
                 last_quality INTEGER NOT NULL DEFAULT 0 CHECK (last_quality BETWEEN 0 AND 5),
                 next_due INTEGER NOT NULL DEFAULT 0,
                 attempts INTEGER NOT NULL DEFAULT 0 CHECK (attempts >= 0),
                 successes INTEGER NOT NULL DEFAULT 0 CHECK (successes >= 0)
             );

             CREATE TABLE IF NOT EXISTS exercise_progress (
                 exercise_id TEXT PRIMARY KEY,
                 attempts INTEGER NOT NULL DEFAULT 0 CHECK (attempts >= 0),
                 completions INTEGER NOT NULL DEFAULT 0 CHECK (completions >= 0),
                 stars INTEGER NOT NULL DEFAULT 0 CHECK (stars BETWEEN 0 AND 3),
                 best_actions INTEGER NOT NULL DEFAULT 0 CHECK (best_actions >= 0),
                 best_millis INTEGER NOT NULL DEFAULT 0 CHECK (best_millis >= 0)
             );

             CREATE TABLE IF NOT EXISTS legacy_exercise_archive (
                 exercise_id TEXT PRIMARY KEY,
                 attempts INTEGER NOT NULL,
                 completions INTEGER NOT NULL,
                 stars INTEGER NOT NULL,
                 best_actions INTEGER NOT NULL,
                 best_millis INTEGER NOT NULL,
                 archived_at INTEGER NOT NULL
             );

             CREATE TABLE IF NOT EXISTS activity (
                 day INTEGER PRIMARY KEY,
                 sessions INTEGER NOT NULL DEFAULT 0,
                 actions INTEGER NOT NULL DEFAULT 0,
                 successes INTEGER NOT NULL DEFAULT 0,
                 seconds INTEGER NOT NULL DEFAULT 0,
                 xp INTEGER NOT NULL DEFAULT 0
             );

             CREATE TABLE IF NOT EXISTS achievements (
                 id TEXT PRIMARY KEY,
                 unlocked_at INTEGER NOT NULL
             );

             CREATE TABLE IF NOT EXISTS settings (
                 id INTEGER PRIMARY KEY CHECK (id = 1),
                 hints INTEGER NOT NULL DEFAULT 1 CHECK (hints IN (0, 1)),
                 animations INTEGER NOT NULL DEFAULT 1 CHECK (animations IN (0, 1)),
                 high_contrast INTEGER NOT NULL DEFAULT 0 CHECK (high_contrast IN (0, 1)),
                 sound INTEGER NOT NULL DEFAULT 0 CHECK (sound IN (0, 1)),
                 drill_minutes INTEGER NOT NULL DEFAULT 5 CHECK (drill_minutes IN (3, 5, 10)),
                 difficulty INTEGER NOT NULL DEFAULT 1 CHECK (difficulty BETWEEN 0 AND 2)
             );
             INSERT OR IGNORE INTO settings (id) VALUES (1);
            ",
        )?;

        migrate_legacy_profile(&transaction)?;
        migrate_legacy_exercises(&transaction)?;
        archive_and_map_legacy_exercises(&transaction)?;
        if legacy_achievements {
            transaction.execute_batch(
                "INSERT OR IGNORE INTO achievements (id, unlocked_at)
                 SELECT id,
                        COALESCE(
                            CAST(strftime('%s', unlocked_at) AS INTEGER),
                            CAST(unlocked_at AS INTEGER),
                            0
                        )
                 FROM achievements_legacy_v0;
                 DROP TABLE achievements_legacy_v0;",
            )?;
        }

        // The old application used these names. Drop them only after their
        // useful data has been copied successfully inside this transaction.
        transaction.execute_batch(
            "DROP TABLE IF EXISTS user_stats;
             DROP TABLE IF EXISTS command_progress;",
        )?;
        transaction.pragma_update(None, "user_version", SCHEMA_VERSION)?;
        transaction.commit()?;
        Ok(())
    }

    #[must_use]
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn profile(&self) -> Result<Profile> {
        let (mut profile, last_practice_day) = self.connection.query_row(
            "SELECT xp, streak_days, best_streak, total_sessions, total_actions,
                    successful_actions, total_practice_seconds, last_practice_day
             FROM profile WHERE id = 1",
            [],
            |row| {
                Ok((
                    Profile {
                        level: 1,
                        xp: row.get(0)?,
                        streak_days: row.get(1)?,
                        best_streak: row.get(2)?,
                        total_sessions: row.get(3)?,
                        total_actions: row.get(4)?,
                        successful_actions: row.get(5)?,
                        total_practice_seconds: row.get(6)?,
                        commands_mastered: 0,
                        commands_learning: 0,
                    },
                    row.get::<_, Option<i64>>(7)?,
                ))
            },
        )?;
        if last_practice_day.is_some_and(|day| day.saturating_add(1) < local_day()) {
            profile.streak_days = 0;
        }
        profile.level = 1 + (profile.xp / 250).min(9) as u32;
        profile.commands_mastered = self.connection.query_row(
            "SELECT COUNT(*) FROM reviews WHERE repetitions >= 3 AND last_quality >= 3",
            [],
            |row| row.get(0),
        )?;
        profile.commands_learning = self.connection.query_row(
            "SELECT COUNT(*) FROM reviews WHERE NOT (repetitions >= 3 AND last_quality >= 3)",
            [],
            |row| row.get(0),
        )?;
        Ok(profile)
    }

    pub fn settings(&self) -> Result<Settings> {
        self.connection
            .query_row(
                "SELECT hints, animations, high_contrast, sound, drill_minutes, difficulty
                 FROM settings WHERE id = 1",
                [],
                |row| {
                    Ok(Settings {
                        hints: row.get::<_, i64>(0)? != 0,
                        animations: row.get::<_, i64>(1)? != 0,
                        high_contrast: row.get::<_, i64>(2)? != 0,
                        sound: row.get::<_, i64>(3)? != 0,
                        drill_minutes: row.get(4)?,
                        difficulty: row.get(5)?,
                    })
                },
            )
            .map_err(Into::into)
    }

    pub fn save_settings(&self, settings: &Settings) -> Result<()> {
        self.connection.execute(
            "UPDATE settings SET hints = ?1, animations = ?2, high_contrast = ?3,
                    sound = ?4, drill_minutes = ?5, difficulty = ?6 WHERE id = 1",
            params![
                i64::from(settings.hints),
                i64::from(settings.animations),
                i64::from(settings.high_contrast),
                i64::from(settings.sound),
                settings.drill_minutes,
                settings.difficulty,
            ],
        )?;
        Ok(())
    }

    pub fn record_result(
        &mut self,
        report: &PracticeReport<'_>,
        policy: ScoringPolicy,
    ) -> Result<Reward> {
        let now = unix_now();
        let day = local_day();
        let difficulty: u8 = self.connection.query_row(
            "SELECT difficulty FROM settings WHERE id = 1",
            [],
            |row| row.get(0),
        )?;
        let actions = report.semantic_actions.max(1);
        let efficiency = if report.optimal_actions == 0 {
            1.0
        } else {
            f64::from(report.optimal_actions) / f64::from(actions)
        }
        .min(1.0);
        let quality = if !report.success {
            if report.mistakes > 2 { 1 } else { 2 }
        } else if report.mistakes == 0 && report.hints == 0 && efficiency >= 0.9 {
            5
        } else if report.mistakes <= 1 && report.hints <= 1 && efficiency >= 0.7 {
            4
        } else {
            3
        };
        let stars = if !report.success {
            0
        } else if quality == 5 {
            3
        } else if quality == 4 {
            2
        } else {
            1
        };

        if policy == ScoringPolicy::FreePractice {
            return Ok(Reward {
                xp: 0,
                stars: 0,
                quality,
                first_completion: false,
            });
        }

        let transaction = self.connection.transaction()?;
        let previous_completions: Option<u32> = transaction
            .query_row(
                "SELECT completions FROM exercise_progress WHERE exercise_id = ?1",
                [report.exercise_id],
                |row| row.get(0),
            )
            .optional()?;
        let first_completion = report.success && previous_completions.unwrap_or(0) == 0;

        let mut xp = match (policy, report.success, first_completion) {
            // Guided lessons teach and unlock a card once; repeating an
            // already-completed lesson must not become an XP farm.
            (ScoringPolicy::GuidedLearning, true, false) => 0,
            (_, true, _) => 15 + u32::from(stars) * 5,
            _ => 0,
        };
        if first_completion {
            xp += 35;
        }

        transaction.execute(
            "INSERT INTO exercise_progress
                 (exercise_id, attempts, completions, stars, best_actions, best_millis)
             VALUES (?1, 1, ?2, ?3, ?4, ?5)
             ON CONFLICT(exercise_id) DO UPDATE SET
                 attempts = attempts + 1,
                 completions = completions + excluded.completions,
                 stars = MAX(stars, excluded.stars),
                 best_actions = CASE
                     WHEN excluded.completions = 0 THEN best_actions
                     WHEN best_actions = 0 THEN excluded.best_actions
                     ELSE MIN(best_actions, excluded.best_actions)
                 END,
                 best_millis = CASE
                     WHEN excluded.completions = 0 THEN best_millis
                     WHEN best_millis = 0 THEN excluded.best_millis
                     ELSE MIN(best_millis, excluded.best_millis)
                 END",
            params![
                report.exercise_id,
                i64::from(report.success),
                stars,
                if report.success { actions } else { 0 },
                if report.success {
                    report.elapsed.as_millis().min(i64::MAX as u128) as i64
                } else {
                    0
                },
            ],
        )?;

        let mut seen_skills = HashSet::with_capacity(report.skills.len());
        for skill in report.skills {
            if !seen_skills.insert(*skill) {
                continue;
            }
            let current = load_review_from(&transaction, skill)?;
            let is_new = current.is_none();
            let should_schedule = match policy {
                ScoringPolicy::DailyDrill => true,
                ScoringPolicy::GuidedLearning => report.success && is_new,
                ScoringPolicy::FreePractice => false,
            };
            if !should_schedule {
                continue;
            }
            let mut next = schedule_review(current.unwrap_or_default(), quality, now);
            if quality >= 3 && next.repetitions >= 3 {
                let multiplier = match difficulty {
                    0 => 1.25,
                    2 => 0.75,
                    _ => 1.0,
                };
                next.interval_days = (next.interval_days * multiplier).round().max(1.0);
                next.next_due =
                    now.saturating_add((next.interval_days * DAY_SECONDS as f64).round() as i64);
            }
            transaction.execute(
                "INSERT INTO reviews
                     (command_id, repetitions, interval_days, ease_factor, last_quality,
                      next_due, attempts, successes)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                 ON CONFLICT(command_id) DO UPDATE SET
                     repetitions = excluded.repetitions,
                     interval_days = excluded.interval_days,
                     ease_factor = excluded.ease_factor,
                     last_quality = excluded.last_quality,
                     next_due = excluded.next_due,
                     attempts = excluded.attempts,
                     successes = excluded.successes",
                params![
                    skill,
                    next.repetitions,
                    next.interval_days,
                    next.ease_factor,
                    next.last_quality,
                    next.next_due,
                    next.attempts,
                    next.successes,
                ],
            )?;
        }

        let successful_actions = if report.success {
            actions.saturating_sub(report.mistakes.min(actions))
        } else {
            0
        };
        transaction.execute(
            "UPDATE profile SET xp = xp + ?1, total_actions = total_actions + ?2,
                    successful_actions = successful_actions + ?3 WHERE id = 1",
            params![xp, actions, successful_actions],
        )?;
        transaction.execute(
            "INSERT INTO activity (day, actions, successes, xp) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(day) DO UPDATE SET actions = actions + excluded.actions,
                 successes = successes + excluded.successes, xp = xp + excluded.xp",
            params![day, actions, successful_actions, xp],
        )?;

        if first_completion {
            unlock(&transaction, "first_steps", now)?;
        }
        if stars == 3 {
            unlock(&transaction, "perfect_form", now)?;
        }
        transaction.commit()?;

        Ok(Reward {
            xp,
            stars,
            quality,
            first_completion,
        })
    }

    pub fn finish_session(&mut self, elapsed: Duration) -> Result<()> {
        let day = local_day();
        let now = unix_now();
        let seconds = elapsed.as_secs().min(i64::MAX as u64) as i64;
        let transaction = self.connection.transaction()?;
        let (last_day, old_streak): (Option<i64>, u32) = transaction.query_row(
            "SELECT last_practice_day, streak_days FROM profile WHERE id = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;
        let streak = match last_day {
            Some(last) if last == day => old_streak.max(1),
            Some(last) if last + 1 == day => old_streak.saturating_add(1),
            _ => 1,
        };
        transaction.execute(
            "UPDATE profile SET streak_days = ?1, best_streak = MAX(best_streak, ?1),
                    total_sessions = total_sessions + 1,
                    total_practice_seconds = total_practice_seconds + ?2,
                    last_practice_day = ?3 WHERE id = 1",
            params![streak, seconds, day],
        )?;
        transaction.execute(
            "INSERT INTO activity (day, sessions, seconds) VALUES (?1, 1, ?2)
             ON CONFLICT(day) DO UPDATE SET sessions = sessions + 1,
                 seconds = seconds + excluded.seconds",
            params![day, seconds],
        )?;
        unlock(&transaction, "first_session", now)?;
        if streak >= 7 {
            unlock(&transaction, "week_warrior", now)?;
        }
        transaction.commit()?;
        Ok(())
    }

    pub fn review_card(&self, command_id: &str) -> Result<Option<ReviewCard>> {
        load_review_from(&self.connection, command_id)
    }

    pub fn due_commands(&self, now: i64) -> Result<Vec<String>> {
        let mut statement = self.connection.prepare(
            "SELECT command_id FROM reviews WHERE next_due <= ?1 ORDER BY next_due, command_id",
        )?;
        let rows = statement.query_map([now], |row| row.get(0))?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    pub fn exercise_records(&self) -> Result<HashMap<String, ExerciseRecord>> {
        let mut statement = self.connection.prepare(
            "SELECT exercise_id, attempts, completions, stars, best_actions, best_millis
             FROM exercise_progress",
        )?;
        let rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                ExerciseRecord {
                    attempts: row.get(1)?,
                    completions: row.get(2)?,
                    stars: row.get(3)?,
                    best_actions: row.get(4)?,
                    best_millis: row.get(5)?,
                },
            ))
        })?;
        Ok(rows.collect::<rusqlite::Result<HashMap<_, _>>>()?)
    }

    pub fn recent_activity(&self, days: u16) -> Result<Vec<ActivityDay>> {
        let today = local_day();
        let first = today - i64::from(days.saturating_sub(1));
        let mut statement = self.connection.prepare(
            "SELECT day, sessions, actions, successes, seconds, xp
             FROM activity WHERE day >= ?1 ORDER BY day",
        )?;
        let rows = statement.query_map([first], |row| {
            Ok(ActivityDay {
                day: row.get(0)?,
                sessions: row.get(1)?,
                actions: row.get(2)?,
                successes: row.get(3)?,
                seconds: row.get(4)?,
                xp: row.get(5)?,
            })
        })?;
        let stored = rows
            .collect::<rusqlite::Result<Vec<_>>>()?
            .into_iter()
            .map(|item| (item.day, item))
            .collect::<HashMap<_, _>>();
        Ok((first..=today)
            .map(|day| {
                stored.get(&day).cloned().unwrap_or(ActivityDay {
                    day,
                    sessions: 0,
                    actions: 0,
                    successes: 0,
                    seconds: 0,
                    xp: 0,
                })
            })
            .collect())
    }

    pub fn achievements(&self) -> Result<Vec<(String, i64)>> {
        let mut statement = self
            .connection
            .prepare("SELECT id, unlocked_at FROM achievements ORDER BY unlocked_at")?;
        let rows = statement.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    /// Removes only Vimurai's records after an explicit confirmation in the UI.
    pub fn reset_all(&mut self) -> Result<()> {
        let transaction = self.connection.transaction()?;
        transaction.execute_batch(
            "DELETE FROM reviews;
             DELETE FROM exercise_progress;
             DELETE FROM legacy_exercise_archive;
             DELETE FROM activity;
             DELETE FROM achievements;
             UPDATE profile SET xp = 0, streak_days = 0, best_streak = 0,
                 total_sessions = 0, total_actions = 0, successful_actions = 0,
                 total_practice_seconds = 0, last_practice_day = NULL WHERE id = 1;",
        )?;
        transaction.commit()?;
        Ok(())
    }
}

fn table_exists(connection: &Connection, table: &str) -> rusqlite::Result<bool> {
    connection.query_row(
        "SELECT EXISTS(
             SELECT 1 FROM sqlite_schema WHERE type = 'table' AND name = ?1
         )",
        [table],
        |row| row.get(0),
    )
}

fn achievement_timestamp_type(connection: &Connection) -> rusqlite::Result<Option<String>> {
    connection
        .query_row(
            "SELECT type FROM pragma_table_info('achievements') WHERE name = 'unlocked_at'",
            [],
            |row| row.get(0),
        )
        .optional()
}

fn migrate_legacy_profile(connection: &Connection) -> rusqlite::Result<()> {
    if !table_exists(connection, "user_stats")? {
        return Ok(());
    }

    connection.execute_batch(
        "INSERT INTO profile (
             id, xp, streak_days, best_streak, total_sessions,
             total_practice_seconds, last_practice_day
         )
         SELECT
             1,
             MAX(CAST(xp AS INTEGER), 0),
             MAX(CAST(streak_days AS INTEGER), 0),
             MAX(CAST(streak_days AS INTEGER), 0),
             MAX(CAST(total_sessions AS INTEGER), 0),
             CASE
                 WHEN total_practice_minutes >= 153722867280912930
                     THEN 9223372036854775807
                 ELSE MAX(CAST(total_practice_minutes AS INTEGER), 0) * 60
             END,
             CASE
                 WHEN julianday(last_practice_date) IS NULL THEN NULL
                 ELSE CAST(julianday(date(last_practice_date)) - 1721424.5 AS INTEGER)
             END
         FROM user_stats
         ORDER BY id DESC
         LIMIT 1
         ON CONFLICT(id) DO UPDATE SET
             xp = MAX(profile.xp, excluded.xp),
             streak_days = MAX(profile.streak_days, excluded.streak_days),
             best_streak = MAX(profile.best_streak, excluded.best_streak),
             total_sessions = MAX(profile.total_sessions, excluded.total_sessions),
             total_practice_seconds = MAX(
                 profile.total_practice_seconds,
                 excluded.total_practice_seconds
             ),
             last_practice_day = CASE
                 WHEN profile.last_practice_day IS NULL THEN excluded.last_practice_day
                 WHEN excluded.last_practice_day IS NULL THEN profile.last_practice_day
                 ELSE MAX(profile.last_practice_day, excluded.last_practice_day)
             END;",
    )?;
    Ok(())
}

fn migrate_legacy_exercises(connection: &Connection) -> rusqlite::Result<()> {
    if !table_exists(connection, "command_progress")? {
        return Ok(());
    }

    // Despite its old name, command_progress was keyed with exercise IDs in
    // the legacy app. Preserve that history as exercise completion data; using
    // those IDs as review-card keys would poison the command-based drill queue.
    connection.execute_batch(
        "INSERT INTO exercise_progress (
             exercise_id, attempts, completions, stars, best_actions, best_millis
         )
         SELECT
             command_id,
             MAX(MAX(CAST(repetition AS INTEGER), 0), 1),
             MAX(MAX(CAST(repetition AS INTEGER), 0), 1),
             CASE
                 WHEN CAST(quality AS INTEGER) >= 4 THEN 3
                 WHEN CAST(quality AS INTEGER) >= 3 THEN 1
                 ELSE 0
             END,
             0,
             0
         FROM command_progress
         WHERE true
         ON CONFLICT(exercise_id) DO UPDATE SET
             attempts = MAX(exercise_progress.attempts, excluded.attempts),
             completions = MAX(exercise_progress.completions, excluded.completions),
             stars = MAX(exercise_progress.stars, excluded.stars);",
    )?;
    Ok(())
}

fn archive_and_map_legacy_exercises(connection: &Connection) -> rusqlite::Result<()> {
    const LEGACY_IDS: &str =
        "'N1','N2','N3','N4','I1','I2','I3','S1','S2','S3','L1','R1','R2','R3','R4'";
    connection.execute_batch(&format!(
        "INSERT INTO legacy_exercise_archive
             (exercise_id, attempts, completions, stars, best_actions, best_millis, archived_at)
         SELECT exercise_id, attempts, completions, stars, best_actions, best_millis,
                CAST(strftime('%s', 'now') AS INTEGER)
         FROM exercise_progress WHERE exercise_id IN ({LEGACY_IDS})
         ON CONFLICT(exercise_id) DO UPDATE SET
             attempts = MAX(attempts, excluded.attempts),
             completions = MAX(completions, excluded.completions),
             stars = MAX(stars, excluded.stars),
             best_actions = CASE
                 WHEN best_actions = 0 THEN excluded.best_actions
                 WHEN excluded.best_actions = 0 THEN best_actions
                 ELSE MIN(best_actions, excluded.best_actions)
             END,
             best_millis = CASE
                 WHEN best_millis = 0 THEN excluded.best_millis
                 WHEN excluded.best_millis = 0 THEN best_millis
                 ELSE MIN(best_millis, excluded.best_millis)
             END;

         INSERT INTO exercise_progress
             (exercise_id, attempts, completions, stars, best_actions, best_millis)
         SELECT 'SUR-01', n1.attempts + n2.attempts,
                MIN(n1.completions, n2.completions), MIN(n1.stars, n2.stars), 0, 0
         FROM exercise_progress AS n1, exercise_progress AS n2
         WHERE n1.exercise_id = 'N1' AND n2.exercise_id = 'N2'
               AND n1.completions > 0 AND n2.completions > 0
         ON CONFLICT(exercise_id) DO UPDATE SET
             attempts = MAX(attempts, excluded.attempts),
             completions = MAX(completions, excluded.completions),
             stars = MAX(stars, excluded.stars);

         INSERT INTO exercise_progress
             (exercise_id, attempts, completions, stars, best_actions, best_millis)
         SELECT 'SUR-05', i1.attempts + i2.attempts + i3.attempts,
                MIN(i1.completions, i2.completions, i3.completions),
                MIN(i1.stars, i2.stars, i3.stars), 0, 0
         FROM exercise_progress AS i1, exercise_progress AS i2, exercise_progress AS i3
         WHERE i1.exercise_id = 'I1' AND i2.exercise_id = 'I2' AND i3.exercise_id = 'I3'
               AND i1.completions > 0 AND i2.completions > 0 AND i3.completions > 0
         ON CONFLICT(exercise_id) DO UPDATE SET
             attempts = MAX(attempts, excluded.attempts),
             completions = MAX(completions, excluded.completions),
             stars = MAX(stars, excluded.stars);

         INSERT INTO exercise_progress
             (exercise_id, attempts, completions, stars, best_actions, best_millis)
         SELECT CASE exercise_id
                    WHEN 'N4' THEN 'SUR-04'
                    WHEN 'S1' THEN 'SNI-01'
                    WHEN 'S3' THEN 'SNI-02'
                    WHEN 'R1' THEN 'REF-01'
                END,
                attempts, completions, stars, 0, 0
         FROM exercise_progress
         WHERE exercise_id IN ('N4', 'S1', 'S3', 'R1') AND completions > 0
         ON CONFLICT(exercise_id) DO UPDATE SET
             attempts = MAX(attempts, excluded.attempts),
             completions = MAX(completions, excluded.completions),
             stars = MAX(stars, excluded.stars);

         DELETE FROM exercise_progress WHERE exercise_id IN ({LEGACY_IDS});"
    ))?;
    Ok(())
}

fn load_review_from(connection: &Connection, command_id: &str) -> Result<Option<ReviewCard>> {
    Ok(connection
        .query_row(
            "SELECT repetitions, interval_days, ease_factor, last_quality, next_due,
                    attempts, successes FROM reviews WHERE command_id = ?1",
            [command_id],
            |row| {
                Ok(ReviewCard {
                    repetitions: row.get(0)?,
                    interval_days: row.get(1)?,
                    ease_factor: row.get(2)?,
                    last_quality: row.get(3)?,
                    next_due: row.get(4)?,
                    attempts: row.get(5)?,
                    successes: row.get(6)?,
                })
            },
        )
        .optional()?)
}

fn unlock(connection: &Connection, id: &str, now: i64) -> rusqlite::Result<()> {
    connection.execute(
        "INSERT OR IGNORE INTO achievements (id, unlocked_at) VALUES (?1, ?2)",
        params![id, now],
    )?;
    Ok(())
}

#[must_use]
pub fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .min(i64::MAX as u64) as i64
}

#[must_use]
pub fn local_day() -> i64 {
    i64::from(Local::now().date_naive().num_days_from_ce())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temporary_database_path(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        env::temp_dir().join(format!("vimurai-{label}-{}-{nonce}.db", std::process::id()))
    }

    #[test]
    fn sm2_uses_the_canonical_first_intervals() {
        let first = schedule_review(ReviewCard::default(), 5, 1_000);
        assert_eq!(first.repetitions, 1);
        assert_eq!(first.interval_days, 1.0);
        assert!(first.ease_factor > 2.5);

        let second = schedule_review(first, 5, first.next_due);
        assert_eq!(second.repetitions, 2);
        assert_eq!(second.interval_days, 6.0);
        assert!(second.next_due > first.next_due);
    }

    #[test]
    fn sm2_lapse_resets_repetitions_and_penalizes_ease() {
        let learned = ReviewCard {
            repetitions: 4,
            interval_days: 20.0,
            ..ReviewCard::default()
        };
        let lapsed = schedule_review(learned, 1, 10);
        assert_eq!(lapsed.repetitions, 0);
        assert_eq!(lapsed.interval_days, 1.0);
        assert!(lapsed.ease_factor < learned.ease_factor);
    }

    #[test]
    fn in_memory_store_is_hermetic_and_records_progress() {
        let mut store = ProgressStore::in_memory().expect("in-memory database");
        let report = PracticeReport {
            exercise_id: "N1",
            skills: &["j"],
            success: true,
            semantic_actions: 2,
            mistakes: 0,
            hints: 0,
            elapsed: Duration::from_secs(4),
            optimal_actions: 2,
        };
        let reward = store
            .record_result(&report, ScoringPolicy::DailyDrill)
            .expect("record result");
        assert_eq!(reward.stars, 3);
        assert!(reward.first_completion);
        assert_eq!(store.review_card("j").unwrap().unwrap().repetitions, 1);
        assert_eq!(store.exercise_records().unwrap()["N1"].completions, 1);
        assert!(store.profile().unwrap().xp > 0);
        assert!(store.path().is_none());
    }

    #[test]
    fn settings_round_trip_and_reset_are_complete() {
        let mut store = ProgressStore::in_memory().unwrap();
        let settings = Settings {
            hints: false,
            animations: false,
            high_contrast: true,
            sound: true,
            drill_minutes: 10,
            difficulty: 2,
        };
        store.save_settings(&settings).unwrap();
        assert_eq!(store.settings().unwrap(), settings);

        store
            .record_result(
                &PracticeReport {
                    exercise_id: "N1",
                    skills: &["h"],
                    success: true,
                    semantic_actions: 1,
                    mistakes: 0,
                    hints: 0,
                    elapsed: Duration::from_secs(1),
                    optimal_actions: 1,
                },
                ScoringPolicy::DailyDrill,
            )
            .unwrap();
        store.reset_all().unwrap();
        assert_eq!(store.profile().unwrap().xp, 0);
        assert!(store.exercise_records().unwrap().is_empty());
        assert_eq!(store.settings().unwrap(), settings);
    }

    #[test]
    fn migrates_the_legacy_database_without_losing_progress() {
        let path = temporary_database_path("legacy-migration");
        let legacy = Connection::open(&path).unwrap();
        legacy
            .execute_batch(
                "CREATE TABLE user_stats (
                     id INTEGER PRIMARY KEY,
                     level INTEGER NOT NULL DEFAULT 1,
                     xp INTEGER NOT NULL DEFAULT 0,
                     commands_mastered INTEGER NOT NULL DEFAULT 0,
                     commands_learning INTEGER NOT NULL DEFAULT 0,
                     streak_days INTEGER NOT NULL DEFAULT 0,
                     total_sessions INTEGER NOT NULL DEFAULT 0,
                     total_practice_minutes INTEGER NOT NULL DEFAULT 0,
                     last_practice_date TEXT
                 );
                 CREATE TABLE command_progress (
                     command_id TEXT PRIMARY KEY,
                     repetition INTEGER NOT NULL DEFAULT 0,
                     interval_days REAL NOT NULL DEFAULT 1.0,
                     ease_factor REAL NOT NULL DEFAULT 2.5,
                     quality INTEGER NOT NULL DEFAULT 0,
                     next_review INTEGER NOT NULL DEFAULT 0,
                     mastered INTEGER NOT NULL DEFAULT 0
                 );
                 CREATE TABLE achievements (
                     id TEXT PRIMARY KEY,
                     unlocked_at TEXT NOT NULL
                 );
                 INSERT INTO user_stats VALUES
                     (1, 4, 2130, 1, 2, 3, 7, 5, '2026-05-30');
                 INSERT INTO command_progress VALUES
                     ('N1', 4, 20.0, 2.5, 4, 1780143637, 1),
                     ('N2', 4, 20.0, 2.5, 4, 1780143637, 1);
                 INSERT INTO achievements VALUES
                     ('mastered:N1', '2026-05-30T12:20:37.792958953+00:00');",
            )
            .unwrap();
        drop(legacy);

        let store = ProgressStore::open_path(&path).unwrap();
        let profile = store.profile().unwrap();
        assert_eq!(profile.xp, 2130);
        assert_eq!(
            profile.streak_days, 0,
            "una racha vencida no se muestra activa"
        );
        assert_eq!(profile.best_streak, 3);
        assert_eq!(profile.total_sessions, 7);
        assert_eq!(profile.total_practice_seconds, 300);

        let records = store.exercise_records().unwrap();
        assert!(!records.contains_key("N1"));
        assert!(!records.contains_key("N2"));
        let exercise = &records["SUR-01"];
        assert_eq!(exercise.attempts, 8);
        assert_eq!(exercise.completions, 4);
        assert_eq!(exercise.stars, 3);
        let archived: i64 = store
            .connection
            .query_row("SELECT COUNT(*) FROM legacy_exercise_archive", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(archived, 2);
        assert!(store.due_commands(i64::MAX).unwrap().is_empty());
        assert_eq!(
            store.achievements().unwrap(),
            vec![("mastered:N1".to_string(), 1_780_143_637)]
        );
        assert!(!table_exists(&store.connection, "user_stats").unwrap());
        assert!(!table_exists(&store.connection, "command_progress").unwrap());
        let version: i64 = store
            .connection
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(version, SCHEMA_VERSION);

        drop(store);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn scoring_policy_keeps_guided_and_free_practice_out_of_sm2() {
        let mut store = ProgressStore::in_memory().unwrap();
        let drill = PracticeReport {
            exercise_id: "N1",
            skills: &["j", "j"],
            success: true,
            semantic_actions: 2,
            mistakes: 0,
            hints: 0,
            elapsed: Duration::from_secs(2),
            optimal_actions: 2,
        };
        store
            .record_result(&drill, ScoringPolicy::DailyDrill)
            .unwrap();
        let learned = store.review_card("j").unwrap().unwrap();
        assert_eq!(learned.attempts, 1, "duplicate skills count only once");

        let guided_failure = PracticeReport {
            exercise_id: "N2",
            skills: &["j", "k"],
            success: false,
            semantic_actions: 3,
            mistakes: 2,
            hints: 1,
            elapsed: Duration::from_secs(3),
            optimal_actions: 1,
        };
        store
            .record_result(&guided_failure, ScoringPolicy::GuidedLearning)
            .unwrap();
        assert_eq!(store.review_card("j").unwrap(), Some(learned));
        assert!(store.review_card("k").unwrap().is_none());
        assert_eq!(store.exercise_records().unwrap()["N2"].best_actions, 0);

        let guided_success = PracticeReport {
            success: true,
            mistakes: 0,
            hints: 0,
            ..guided_failure
        };
        let first_guided_reward = store
            .record_result(&guided_success, ScoringPolicy::GuidedLearning)
            .unwrap();
        assert!(first_guided_reward.xp > 0);
        assert_eq!(store.review_card("j").unwrap(), Some(learned));
        let guided_card = store.review_card("k").unwrap().unwrap();
        assert_eq!(guided_card.repetitions, 1);
        let repeat_reward = store
            .record_result(&guided_success, ScoringPolicy::GuidedLearning)
            .unwrap();
        assert_eq!(repeat_reward.xp, 0);
        assert_eq!(store.review_card("k").unwrap(), Some(guided_card));

        let before_free = store.profile().unwrap();
        let free = PracticeReport {
            exercise_id: "sandbox",
            skills: &["l"],
            ..guided_success
        };
        let reward = store
            .record_result(&free, ScoringPolicy::FreePractice)
            .unwrap();
        assert_eq!(reward.xp, 0);
        assert_eq!(store.profile().unwrap(), before_free);
        assert!(!store.exercise_records().unwrap().contains_key("sandbox"));
        assert!(store.review_card("l").unwrap().is_none());
    }

    #[test]
    fn difficulty_changes_mature_intervals_and_accuracy_counts_mistakes() {
        fn mature_interval(difficulty: u8) -> f64 {
            let mut store = ProgressStore::in_memory().unwrap();
            let mut settings = store.settings().unwrap();
            settings.difficulty = difficulty;
            store.save_settings(&settings).unwrap();
            let report = PracticeReport {
                exercise_id: "accuracy",
                skills: &["w"],
                success: true,
                semantic_actions: 5,
                mistakes: 2,
                hints: 0,
                elapsed: Duration::from_secs(3),
                optimal_actions: 3,
            };
            for _ in 0..3 {
                store
                    .record_result(&report, ScoringPolicy::DailyDrill)
                    .unwrap();
            }
            let profile = store.profile().unwrap();
            assert_eq!(profile.total_actions, 15);
            assert_eq!(profile.successful_actions, 9);
            store.review_card("w").unwrap().unwrap().interval_days
        }

        let relaxed = mature_interval(0);
        let balanced = mature_interval(1);
        let strict = mature_interval(2);
        assert!(relaxed > balanced, "{relaxed} > {balanced}");
        assert!(balanced > strict, "{balanced} > {strict}");
    }
}
