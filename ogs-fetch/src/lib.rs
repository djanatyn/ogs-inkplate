pub mod compression;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub game_id: u64,
    pub black_player: String,
    pub white_player: String,
    pub result: String,
    pub date: String,
    pub downloaded: bool,
}

#[derive(Debug, Deserialize)]
pub struct ApiResponse {
    pub count: usize,
    pub next: Option<String>,
    pub previous: Option<String>,
    pub results: Vec<ApiGameResult>,
}

#[derive(Debug, Deserialize)]
pub struct ApiGameResult {
    pub id: u64,
    pub players: ApiPlayers,
    pub outcome: String,
    pub ended: String,
    pub handicap: u32,
}

#[derive(Debug, Deserialize)]
pub struct ApiPlayers {
    pub black: ApiPlayer,
    pub white: ApiPlayer,
}

#[derive(Debug, Deserialize)]
pub struct ApiPlayer {
    pub username: String,
}

pub fn is_valid_game_outcome(outcome: &str) -> bool {
    // Only include games that completed normally (by points)
    // Skip: Resignation, Timeout, Disconnection, Cancelled, etc.
    !outcome.contains("Resignation")
        && !outcome.contains("Timeout")
        && !outcome.contains("Disconnect")
        && !outcome.contains("Cancelled")
        && !outcome.contains("Annulled")
        // Valid outcomes are point-based scores like "6.5 points", "W+24.5", etc.
        && (outcome.contains("points") || outcome.contains("+"))
}

pub struct Database {
    conn: rusqlite::Connection,
}

impl Database {
    pub fn new(path: &str) -> Result<Self, rusqlite::Error> {
        let conn = rusqlite::Connection::open(path)?;
        Ok(Database { conn })
    }

    pub fn init_schema(&self) -> Result<(), rusqlite::Error> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS games (
                game_id INTEGER PRIMARY KEY,
                black_player TEXT NOT NULL,
                white_player TEXT NOT NULL,
                result TEXT NOT NULL,
                date TEXT NOT NULL,
                downloaded INTEGER DEFAULT 0,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS fetch_state (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            );
            ",
        )?;

        // Initialize fetch_state with defaults if not present
        self.conn.execute(
            "INSERT OR IGNORE INTO fetch_state (key, value) VALUES ('last_page', '0')",
            [],
        )?;

        Ok(())
    }

    pub fn get_last_page(&self) -> Result<u32, rusqlite::Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT value FROM fetch_state WHERE key = 'last_page'")?;
        let page = stmt
            .query_row([], |row| {
                let val: String = row.get(0)?;
                Ok(val.parse::<u32>().unwrap_or(0))
            })
            .unwrap_or(0);
        Ok(page)
    }

    pub fn set_last_page(&self, page: u32) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "UPDATE fetch_state SET value = ?, updated_at = CURRENT_TIMESTAMP WHERE key = 'last_page'",
            [page.to_string()],
        )?;
        Ok(())
    }

    pub fn insert_game(&self, game: &Game) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "INSERT OR IGNORE INTO games (game_id, black_player, white_player, result, date, downloaded)
             VALUES (?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                game.game_id,
                &game.black_player,
                &game.white_player,
                &game.result,
                &game.date,
                game.downloaded as u8
            ],
        )?;
        Ok(())
    }

    pub fn get_undownloaded_games(&self) -> Result<Vec<u64>, rusqlite::Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT game_id FROM games WHERE downloaded = 0")?;
        let game_ids = stmt
            .query_map([], |row| row.get(0))?
            .collect::<Result<Vec<u64>, _>>()?;
        Ok(game_ids)
    }

    pub fn mark_game_downloaded(&self, game_id: u64) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "UPDATE games SET downloaded = 1 WHERE game_id = ?",
            [game_id],
        )?;
        Ok(())
    }

    pub fn count_games(&self) -> Result<u64, rusqlite::Error> {
        let mut stmt = self.conn.prepare("SELECT COUNT(*) FROM games")?;
        let count = stmt.query_row([], |row| row.get(0))?;
        Ok(count)
    }

    pub fn count_downloaded_games(&self) -> Result<u64, rusqlite::Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT COUNT(*) FROM games WHERE downloaded = 1")?;
        let count = stmt.query_row([], |row| row.get(0))?;
        Ok(count)
    }
}
