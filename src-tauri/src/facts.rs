use chrono::Utc;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{SammyError, SammyResult};
use crate::paths::data_root;
use crate::vault::UnlockedVault;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fact {
    pub id: String,
    pub domain: String,
    pub label: String,
    pub value: String,
    pub source: String,
    pub created_at: String,
    pub updated_at: String,
}

pub fn list(vault: &UnlockedVault) -> SammyResult<Vec<Fact>> {
    let mut stmt = vault.db.conn.prepare(
        "SELECT id, domain, label, value, source, created_at, updated_at FROM facts ORDER BY updated_at DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Fact {
            id: row.get(0)?,
            domain: row.get(1)?,
            label: row.get(2)?,
            value: row.get(3)?,
            source: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn upsert(
    vault: &UnlockedVault,
    id: Option<String>,
    domain: String,
    label: String,
    value: String,
) -> SammyResult<Fact> {
    let now = Utc::now().to_rfc3339();
    let id = id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let existing: Option<String> = vault
        .db
        .conn
        .query_row("SELECT id FROM facts WHERE id = ?1", params![id], |r| r.get(0))
        .ok();
    if existing.is_some() {
        vault.db.conn.execute(
            "UPDATE facts SET domain = ?1, label = ?2, value = ?3, updated_at = ?4 WHERE id = ?5",
            params![domain, label, value, now, id],
        )?;
    } else {
        vault.db.conn.execute(
            "INSERT INTO facts (id, domain, label, value, source, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, 'manual', ?5, ?5)",
            params![id, domain, label, value, now],
        )?;
    }
    list(vault)?
        .into_iter()
        .find(|f| f.id == id)
        .ok_or_else(|| SammyError::msg("Fact missing after upsert"))
}

pub fn delete(vault: &UnlockedVault, id: &str) -> SammyResult<()> {
    vault
        .db
        .conn
        .execute("DELETE FROM facts WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn facts_prompt_block(vault: &UnlockedVault) -> SammyResult<String> {
    let facts = list(vault)?;
    if facts.is_empty() {
        return Ok("(none yet)".into());
    }
    let mut lines = Vec::new();
    for f in facts {
        lines.push(format!("- [{}] {}: {}", f.domain, f.label, f.value));
    }
    Ok(lines.join("\n"))
}

pub fn export_markdown(vault: &UnlockedVault) -> SammyResult<String> {
    let facts = list(vault)?;
    let mut md = String::from("# What I know about you\n\n");
    md.push_str(&format!("Exported: {}\n\n", Utc::now().to_rfc3339()));
    if facts.is_empty() {
        md.push_str("_Nothing stored yet._\n");
    } else {
        for f in facts {
            md.push_str(&format!(
                "## {}\n**Domain:** {}\n**Source:** {}\n\n{}\n\n",
                f.label, f.domain, f.source, f.value
            ));
        }
    }
    let path = data_root().join(format!(
        "what-i-know-{}.md",
        Utc::now().format("%Y%m%d-%H%M%S")
    ));
    std::fs::write(&path, &md)?;
    Ok(path.display().to_string())
}
