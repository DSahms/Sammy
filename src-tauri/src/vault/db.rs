use rusqlite::{params, Connection};
use std::path::Path;

use crate::error::{SammyError, SammyResult};
use crate::vault::crypto::{sqlcipher_key, MasterKey};

pub struct VaultDb {
    pub conn: Connection,
}

impl VaultDb {
    pub fn open(path: &Path, master: &MasterKey) -> SammyResult<Self> {
        let conn = Connection::open(path)?;
        let key = sqlcipher_key(master);
        conn.pragma_update(None, "key", &key)?;
        // Verify key by reading sqlite_master
        conn.query_row("SELECT count(*) FROM sqlite_master", [], |r| r.get::<_, i64>(0))
            .map_err(|_| SammyError::BadPassphrase)?;
        Ok(Self { conn })
    }

    pub fn create(path: &Path, master: &MasterKey) -> SammyResult<Self> {
        if path.exists() {
            return Err(SammyError::VaultExists);
        }
        let conn = Connection::open(path)?;
        let key = sqlcipher_key(master);
        conn.pragma_update(None, "key", &key)?;
        conn.execute_batch(
            r#"
            CREATE TABLE meta (
              key TEXT PRIMARY KEY,
              value TEXT NOT NULL
            );
            CREATE TABLE facts (
              id TEXT PRIMARY KEY,
              domain TEXT NOT NULL,
              label TEXT NOT NULL,
              value TEXT NOT NULL,
              source TEXT NOT NULL DEFAULT 'intake',
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL
            );
            CREATE TABLE sessions (
              id TEXT PRIMARY KEY,
              title TEXT NOT NULL,
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL
            );
            CREATE TABLE messages (
              id TEXT PRIMARY KEY,
              session_id TEXT NOT NULL,
              role TEXT NOT NULL,
              content TEXT NOT NULL,
              created_at TEXT NOT NULL,
              FOREIGN KEY(session_id) REFERENCES sessions(id)
            );
            CREATE TABLE intake_progress (
              id INTEGER PRIMARY KEY CHECK (id = 1),
              path TEXT NOT NULL,
              step_index INTEGER NOT NULL DEFAULT 0,
              completed INTEGER NOT NULL DEFAULT 0,
              answers_json TEXT NOT NULL DEFAULT '{}'
            );
            INSERT INTO intake_progress (id, path, step_index, completed, answers_json)
              VALUES (1, 'none', 0, 0, '{}');
            INSERT INTO meta (key, value) VALUES ('schema_version', '1');
            "#,
        )?;
        Ok(Self { conn })
    }

    pub fn set_meta(&self, key: &str, value: &str) -> SammyResult<()> {
        self.conn.execute(
            "INSERT INTO meta(key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn get_meta(&self, key: &str) -> SammyResult<Option<String>> {
        let mut stmt = self.conn.prepare("SELECT value FROM meta WHERE key = ?1")?;
        let mut rows = stmt.query(params![key])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }
}
