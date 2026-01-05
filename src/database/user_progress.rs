use rusqlite::{Connection, Result, ToSql};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct UserStats {
    pub level: u32,
    pub xp: u64,
    pub commands_mastered: u32,
    pub commands_learning: u32,
    pub streak_days: u32,
    pub total_sessions: u64,
    pub total_practice_minutes: u64,
}

#[derive(Debug, Clone)]
pub struct CommandProgress {
    pub command_id: String,
    pub repetition: u32,
    pub interval_days: f64,
    pub ease_factor: f64,
    pub quality: u8,
    pub next_review: u64,
    pub mastered: bool,
}

#[derive(Debug)]
pub struct UserProgressDB {
    conn: Connection,
}

impl UserProgressDB {
    pub fn new() -> Result<Self> {
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("vimurai");

        // Create directory, ignore errors (may already exist)
        let _ = std::fs::create_dir_all(&data_dir);

        let db_path = data_dir.join("progress.db");
        let conn = Connection::open(db_path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS user_stats (
                id INTEGER PRIMARY KEY,
                level INTEGER NOT NULL DEFAULT 1,
                xp INTEGER NOT NULL DEFAULT 0,
                commands_mastered INTEGER NOT NULL DEFAULT 0,
                commands_learning INTEGER NOT NULL DEFAULT 0,
                streak_days INTEGER NOT NULL DEFAULT 0,
                total_sessions INTEGER NOT NULL DEFAULT 0,
                total_practice_minutes INTEGER NOT NULL DEFAULT 0,
                last_practice_date TEXT
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS command_progress (
                command_id TEXT PRIMARY KEY,
                repetition INTEGER NOT NULL DEFAULT 0,
                interval_days REAL NOT NULL DEFAULT 1.0,
                ease_factor REAL NOT NULL DEFAULT 2.5,
                quality INTEGER NOT NULL DEFAULT 0,
                next_review INTEGER NOT NULL DEFAULT 0,
                mastered INTEGER NOT NULL DEFAULT 0
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS achievements (
                id TEXT PRIMARY KEY,
                unlocked_at TEXT NOT NULL
            )",
            [],
        )?;

        Ok(Self { conn })
    }

    pub fn get_stats(&self) -> Result<UserStats> {
        let mut stmt = self.conn.prepare("SELECT * FROM user_stats ORDER BY id DESC LIMIT 1")?;
        let mut rows = stmt.query([])?;

        if let Some(row) = rows.next()? {
            Ok(UserStats {
                level: row.get(1)?,
                xp: row.get(2)?,
                commands_mastered: row.get(3)?,
                commands_learning: row.get(4)?,
                streak_days: row.get(5)?,
                total_sessions: row.get(6)?,
                total_practice_minutes: row.get(7)?,
            })
        } else {
            // Initialize stats
            self.conn.execute(
                "INSERT INTO user_stats (level, xp) VALUES (1, 0)",
                [],
            )?;
            Ok(UserStats {
                level: 1,
                xp: 0,
                commands_mastered: 0,
                commands_learning: 0,
                streak_days: 0,
                total_sessions: 0,
                total_practice_minutes: 0,
            })
        }
    }

    pub fn add_xp(&self, amount: u64) -> Result<u64> {
        let current_xp = self.get_stats()?.xp;
        let new_xp = current_xp + amount;

        self.conn.execute(
            "UPDATE user_stats SET xp = ? WHERE id = (SELECT id FROM user_stats ORDER BY id DESC LIMIT 1)",
            [new_xp as i64],
        )?;

        Ok(new_xp)
    }

    pub fn increment_session(&self) -> Result<()> {
        self.conn.execute(
            "UPDATE user_stats SET total_sessions = total_sessions + 1 WHERE id = (SELECT id FROM user_stats ORDER BY id DESC LIMIT 1)",
            [],
        )?;
        Ok(())
    }

    pub fn get_command_progress(&self, command_id: &str) -> Result<Option<CommandProgress>> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM command_progress WHERE command_id = ?"
        )?;
        let mut rows = stmt.query([command_id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(CommandProgress {
                command_id: row.get(0)?,
                repetition: row.get(1)?,
                interval_days: row.get(2)?,
                ease_factor: row.get(3)?,
                quality: row.get(4)?,
                next_review: row.get(5)?,
                mastered: row.get::<_, i64>(6)? != 0,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn update_command_progress(
        &self,
        command_id: &str,
        repetition: u32,
        interval_days: f64,
        ease_factor: f64,
        quality: u8,
        next_review: u64,
        mastered: bool,
    ) -> Result<()> {
        let params: &[&dyn ToSql] = &[
            &command_id,
            &(repetition as i64),
            &interval_days,
            &ease_factor,
            &(quality as i64),
            &(next_review as i64),
            &(if mastered { 1i64 } else { 0i64 }),
        ];
        self.conn.execute(
            "INSERT OR REPLACE INTO command_progress
             (command_id, repetition, interval_days, ease_factor, quality, next_review, mastered)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            params,
        )?;
        Ok(())
    }

    pub fn get_due_commands(&self) -> Result<Vec<String>> {
        let now = chrono::Utc::now().timestamp() as i64;
        let mut stmt = self.conn.prepare(
            "SELECT command_id FROM command_progress WHERE next_review <= ?"
        )?;
        let mut rows = stmt.query([now])?;
        let mut result = Vec::new();
        while let Some(row) = rows.next()? {
            result.push(row.get(0)?);
        }
        Ok(result)
    }

    pub fn unlock_achievement(&self, achievement_id: &str) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT OR IGNORE INTO achievements (id, unlocked_at) VALUES (?, ?)",
            [achievement_id, &now],
        )?;
        Ok(())
    }

    pub fn get_unlocked_achievements(&self) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare("SELECT id FROM achievements")?;
        let mut rows = stmt.query([])?;
        let mut result = Vec::new();
        while let Some(row) = rows.next()? {
            result.push(row.get(0)?);
        }
        Ok(result)
    }
}

impl Default for UserProgressDB {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self { conn: Connection::open(":memory:").unwrap() })
    }
}
