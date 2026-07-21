use chrono::Utc;
use rusqlite::params;
use uuid::Uuid;

use crate::brain::{self, ChatMessage};
use crate::error::{SammyError, SammyResult};
use crate::facts;
use crate::sources;
use crate::state::AppState;
use crate::vault::UnlockedVault;
use tauri::{AppHandle, Emitter};

pub fn ensure_session(vault: &UnlockedVault, session_id: &mut Option<String>) -> SammyResult<String> {
    if let Some(id) = session_id.clone() {
        return Ok(id);
    }
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    vault.db.conn.execute(
        "INSERT INTO sessions (id, title, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
        params![id, "Chat", now, now],
    )?;
    *session_id = Some(id.clone());
    Ok(id)
}

pub fn append_message(
    vault: &UnlockedVault,
    session_id: &str,
    role: &str,
    content: &str,
) -> SammyResult<()> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    vault.db.conn.execute(
        "INSERT INTO messages (id, session_id, role, content, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, session_id, role, content, now],
    )?;
    vault.db.conn.execute(
        "UPDATE sessions SET updated_at = ?1 WHERE id = ?2",
        params![now, session_id],
    )?;
    Ok(())
}

pub fn history(vault: &UnlockedVault, session_id: &str, limit: usize) -> SammyResult<Vec<ChatMessage>> {
    let mut stmt = vault.db.conn.prepare(
        "SELECT role, content FROM messages WHERE session_id = ?1 ORDER BY created_at ASC",
    )?;
    let rows = stmt.query_map(params![session_id], |row| {
        Ok(ChatMessage {
            role: row.get(0)?,
            content: row.get(1)?,
        })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    if out.len() > limit {
        out = out.split_off(out.len() - limit);
    }
    Ok(out)
}

pub async fn send(
    app: AppHandle,
    state: &AppState,
    user_text: String,
) -> SammyResult<String> {
    let user_text = user_text.trim().to_string();
    if user_text.is_empty() {
        return Err(SammyError::msg("Empty message"));
    }

    let (session_id, facts_block, sources_block, prior) = {
        let mut guard = state.require_vault()?;
        let vault = guard.as_mut().ok_or(SammyError::VaultLocked)?;
        let mut sid = state.active_session.lock().clone();
        let session_id = ensure_session(vault, &mut sid)?;
        *state.active_session.lock() = Some(session_id.clone());
        append_message(vault, &session_id, "user", &user_text)?;
        let facts_block = facts::facts_prompt_block(vault)?;
        let scope = sources::default_scope_ids(vault)?;
        let sources_block = if scope.is_empty() {
            String::new()
        } else {
            sources::build_source_context(vault, &user_text, Some(&scope))?
        };
        let prior = history(vault, &session_id, 24)?;
        (session_id, facts_block, sources_block, prior)
    };

    let brains = state.brains.lock().brains.clone();
    let preferred = state.active_brain.lock().clone();
    let brain = brain::resolve_brain(&state.http, &brains, &preferred).await?;
    let brain = brains
        .iter()
        .find(|b| b.id == brain.id)
        .cloned()
        .ok_or_else(|| SammyError::msg("Brain missing"))?;

    let mut messages = Vec::new();
    messages.push(ChatMessage {
        role: "system".into(),
        content: brain::trust_system_prompt(&facts_block, &sources_block),
    });
    messages.extend(prior);

    let app_chunks = app.clone();
    let reply = brain::generate_stream(&state.http, &brain, &messages, move |chunk| {
        let _ = app_chunks.emit("chat-chunk", chunk);
    })
    .await?;

    {
        let mut guard = state.require_vault()?;
        let vault = guard.as_mut().ok_or(SammyError::VaultLocked)?;
        append_message(vault, &session_id, "assistant", &reply)?;
    }

    let _ = app.emit("chat-done", &reply);
    Ok(reply)
}
