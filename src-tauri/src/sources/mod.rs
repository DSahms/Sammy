//! Document library (Cut 2) — always considered in Chat when present.

pub mod extract;
pub mod rag;

use chrono::Utc;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::error::{SammyError, SammyResult};
use crate::paths::files_dir;
use crate::vault::crypto::encrypt_bytes;
use crate::vault::UnlockedVault;

pub const INGEST_VERSION: u32 = 1;
pub const CHAR_BUDGET: usize = 48_000;
pub const CHUNK_SIZE: usize = 1_800;
pub const CHUNK_OVERLAP: usize = 200;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceDoc {
    pub id: String,
    pub filename: String,
    pub bytes: i64,
    pub chunks: i64,
    pub pinned: bool,
    pub added_at: String,
}

pub fn ensure_schema(vault: &UnlockedVault) -> SammyResult<()> {
    vault.db.conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS sources (
          id TEXT PRIMARY KEY,
          filename TEXT NOT NULL,
          stored_path TEXT NOT NULL,
          bytes INTEGER NOT NULL,
          chunks INTEGER NOT NULL,
          pinned INTEGER NOT NULL DEFAULT 1,
          ingest_version INTEGER NOT NULL DEFAULT 1,
          added_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS source_chunks (
          id TEXT PRIMARY KEY,
          doc_id TEXT NOT NULL,
          chunk_index INTEGER NOT NULL,
          text TEXT NOT NULL,
          FOREIGN KEY(doc_id) REFERENCES sources(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_chunks_doc ON source_chunks(doc_id, chunk_index);
        "#,
    )?;
    let ver = vault.db.get_meta("schema_version")?.unwrap_or_else(|| "1".into());
    if ver.parse::<u32>().unwrap_or(1) < 2 {
        vault.db.set_meta("schema_version", "2")?;
    }
    Ok(())
}

pub fn list(vault: &UnlockedVault) -> SammyResult<Vec<SourceDoc>> {
    ensure_schema(vault)?;
    let mut stmt = vault.db.conn.prepare(
        "SELECT id, filename, bytes, chunks, pinned, added_at FROM sources ORDER BY added_at DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(SourceDoc {
            id: row.get(0)?,
            filename: row.get(1)?,
            bytes: row.get(2)?,
            chunks: row.get(3)?,
            pinned: row.get::<_, i64>(4)? != 0,
            added_at: row.get(5)?,
        })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn set_pinned(vault: &UnlockedVault, id: &str, pinned: bool) -> SammyResult<()> {
    ensure_schema(vault)?;
    vault.db.conn.execute(
        "UPDATE sources SET pinned = ?1 WHERE id = ?2",
        params![if pinned { 1 } else { 0 }, id],
    )?;
    Ok(())
}

pub fn delete(vault: &UnlockedVault, id: &str) -> SammyResult<()> {
    ensure_schema(vault)?;
    let path: Option<String> = vault
        .db
        .conn
        .query_row(
            "SELECT stored_path FROM sources WHERE id = ?1",
            params![id],
            |r| r.get(0),
        )
        .ok();
    vault
        .db
        .conn
        .execute("DELETE FROM source_chunks WHERE doc_id = ?1", params![id])?;
    vault
        .db
        .conn
        .execute("DELETE FROM sources WHERE id = ?1", params![id])?;
    if let Some(p) = path {
        let _ = fs::remove_file(p);
    }
    Ok(())
}

pub fn ingest_path(vault: &UnlockedVault, path: &Path) -> SammyResult<SourceDoc> {
    ensure_schema(vault)?;
    let filename = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("document")
        .to_string();
    let raw = fs::read(path)?;
    let text = extract::extract_text(path, &raw)?;
    if text.trim().is_empty() {
        return Err(SammyError::msg("File had no readable text"));
    }
    let chunks = rag::chunk_text(&text, CHUNK_SIZE, CHUNK_OVERLAP);
    if chunks.is_empty() {
        return Err(SammyError::msg("File produced no chunks"));
    }

    let doc_id = Uuid::new_v4().to_string();
    let dest = encrypted_dest(&doc_id, &filename)?;
    let sealed = encrypt_bytes(&vault.master, &raw)?;
    fs::write(&dest, &sealed)?;

    let now = Utc::now().to_rfc3339();
    vault.db.conn.execute(
        "INSERT INTO sources (id, filename, stored_path, bytes, chunks, pinned, ingest_version, added_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, ?7)",
        params![
            doc_id,
            filename,
            dest.display().to_string(),
            raw.len() as i64,
            chunks.len() as i64,
            INGEST_VERSION as i64,
            now
        ],
    )?;

    for (i, chunk) in chunks.iter().enumerate() {
        let cid = format!("{doc_id}:{i}");
        vault.db.conn.execute(
            "INSERT INTO source_chunks (id, doc_id, chunk_index, text) VALUES (?1, ?2, ?3, ?4)",
            params![cid, doc_id, i as i64, chunk],
        )?;
    }

    // Also park a short fact pointer so "What I know" shows the library link.
    let fact_id = Uuid::new_v4().to_string();
    let preview: String = text.chars().take(280).collect();
    vault.db.conn.execute(
        "INSERT INTO facts (id, domain, label, value, source, created_at, updated_at)
         VALUES (?1, 'documents', ?2, ?3, 'sources', ?4, ?4)",
        params![
            fact_id,
            format!("Source: {filename}"),
            format!("Attached life/library document (id {doc_id}). Preview:\n{preview}…"),
            now
        ],
    )?;

    Ok(SourceDoc {
        id: doc_id,
        filename,
        bytes: raw.len() as i64,
        chunks: chunks.len() as i64,
        pinned: true,
        added_at: now,
    })
}

fn encrypted_dest(doc_id: &str, filename: &str) -> SammyResult<PathBuf> {
    let dir = files_dir().join(doc_id);
    fs::create_dir_all(&dir)?;
    let safe = filename.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
    Ok(dir.join(format!("{safe}.enc")))
}

pub fn build_source_context(
    vault: &UnlockedVault,
    question: &str,
    doc_ids: Option<&[String]>,
) -> SammyResult<String> {
    ensure_schema(vault)?;
    rag::build_context(vault, question, doc_ids, CHAR_BUDGET)
}

/// Docs included in Chat every turn.
/// If every file is pinned, or none are pinned, use the whole library.
/// If mixed (some pinned, some not), only pinned files — explicit focus.
pub fn default_scope_ids(vault: &UnlockedVault) -> SammyResult<Vec<String>> {
    let docs = list(vault)?;
    if docs.is_empty() {
        return Ok(vec![]);
    }
    let pinned: Vec<String> = docs.iter().filter(|d| d.pinned).map(|d| d.id.clone()).collect();
    let unpinned = docs.len() - pinned.len();
    if pinned.is_empty() || unpinned == 0 {
        Ok(docs.into_iter().map(|d| d.id).collect())
    } else {
        Ok(pinned)
    }
}
