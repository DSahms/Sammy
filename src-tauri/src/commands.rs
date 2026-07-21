use serde::Serialize;
use std::path::PathBuf;
use tauri::{AppHandle, State};

use crate::brain;
use crate::chat;
use crate::error::SammyResult;
use crate::facts::{self, Fact};
use crate::intake::{self, IntakeState};
use crate::keychain;
use crate::sources::{self, SourceDoc};
use crate::state::AppState;
use crate::vault::{self, UnlockedVault};

#[derive(Serialize)]
pub struct VaultStatus {
    pub exists: bool,
    pub unlocked: bool,
    pub path: String,
}

#[tauri::command]
pub fn vault_status(state: State<'_, AppState>) -> VaultStatus {
    VaultStatus {
        exists: vault::vault_exists(),
        unlocked: state.vault.lock().is_some(),
        path: vault::data_dir_display(),
    }
}

#[tauri::command]
pub fn vault_create(state: State<'_, AppState>, passphrase: String) -> SammyResult<VaultStatus> {
    let unlocked = vault::create_vault(&passphrase)?;
    *state.vault.lock() = Some(unlocked);
    Ok(vault_status(state))
}

#[tauri::command]
pub fn vault_unlock(state: State<'_, AppState>, passphrase: String) -> SammyResult<VaultStatus> {
    let unlocked = vault::unlock_vault(&passphrase)?;
    *state.vault.lock() = Some(unlocked);
    Ok(vault_status(state))
}

#[tauri::command]
pub fn vault_lock(state: State<'_, AppState>) -> SammyResult<VaultStatus> {
    *state.vault.lock() = None;
    *state.active_session.lock() = None;
    Ok(vault_status(state))
}

#[tauri::command]
pub fn list_brains(state: State<'_, AppState>) -> SammyResult<Vec<brain::config::BrainEntry>> {
    Ok(state.brains.lock().brains.clone())
}

#[tauri::command]
pub fn get_active_brain(state: State<'_, AppState>) -> String {
    state.active_brain.lock().clone()
}

#[tauri::command]
pub fn set_active_brain(state: State<'_, AppState>, id: String) -> SammyResult<String> {
    let brains = state.brains.lock();
    if brains.get(&id).is_none() {
        return Err(crate::error::SammyError::msg("Unknown brain id"));
    }
    drop(brains);
    *state.active_brain.lock() = id.clone();
    Ok(id)
}

#[tauri::command]
pub fn set_brain_api_key(id: String, api_key: String) -> SammyResult<()> {
    keychain::set_api_key(&id, &api_key)
}

#[tauri::command]
pub fn brain_has_key(id: String) -> bool {
    keychain::has_api_key(&id)
}

#[tauri::command]
pub async fn chat_send(
    app: AppHandle,
    state: State<'_, AppState>,
    text: String,
) -> SammyResult<String> {
    chat::send(app, state.inner(), text).await
}

#[tauri::command]
pub fn chat_history(state: State<'_, AppState>) -> SammyResult<Vec<brain::ChatMessage>> {
    let mut guard = state.require_vault()?;
    let vault = guard.as_mut().ok_or(crate::error::SammyError::VaultLocked)?;
    let sid = state.active_session.lock().clone();
    let Some(sid) = sid else {
        return Ok(vec![]);
    };
    chat::history(vault, &sid, 200)
}

#[tauri::command]
pub fn intake_state(state: State<'_, AppState>) -> SammyResult<IntakeState> {
    with_vault(&state, |v| intake::get_state(v))
}

#[tauri::command]
pub fn intake_start(state: State<'_, AppState>, path: String) -> SammyResult<IntakeState> {
    with_vault(&state, |v| intake::start(v, &path))
}

#[tauri::command]
pub fn intake_answer(state: State<'_, AppState>, text: String) -> SammyResult<IntakeState> {
    with_vault(&state, |v| intake::answer(v, &text))
}

#[tauri::command]
pub fn intake_skip_to_chat(state: State<'_, AppState>) -> SammyResult<IntakeState> {
    with_vault(&state, |v| intake::skip_to_chat(v))
}

#[tauri::command]
pub fn facts_list(state: State<'_, AppState>) -> SammyResult<Vec<Fact>> {
    with_vault(&state, |v| facts::list(v))
}

#[tauri::command]
pub fn facts_upsert(
    state: State<'_, AppState>,
    id: Option<String>,
    domain: String,
    label: String,
    value: String,
) -> SammyResult<Fact> {
    with_vault(&state, |v| facts::upsert(v, id, domain, label, value))
}

#[tauri::command]
pub fn facts_delete(state: State<'_, AppState>, id: String) -> SammyResult<()> {
    with_vault(&state, |v| facts::delete(v, &id))
}

#[tauri::command]
pub fn facts_export(state: State<'_, AppState>) -> SammyResult<String> {
    with_vault(&state, |v| facts::export_markdown(v))
}

#[tauri::command]
pub fn sources_list(state: State<'_, AppState>) -> SammyResult<Vec<SourceDoc>> {
    with_vault(&state, |v| sources::list(v))
}

#[tauri::command]
pub fn sources_add(state: State<'_, AppState>, path: String) -> SammyResult<SourceDoc> {
    with_vault(&state, |v| sources::ingest_path(v, &PathBuf::from(path)))
}

#[tauri::command]
pub fn sources_delete(state: State<'_, AppState>, id: String) -> SammyResult<()> {
    with_vault(&state, |v| sources::delete(v, &id))
}

#[tauri::command]
pub fn sources_set_pinned(
    state: State<'_, AppState>,
    id: String,
    pinned: bool,
) -> SammyResult<()> {
    with_vault(&state, |v| sources::set_pinned(v, &id, pinned))
}

fn with_vault<T>(
    state: &AppState,
    f: impl FnOnce(&UnlockedVault) -> SammyResult<T>,
) -> SammyResult<T> {
    let guard = state.require_vault()?;
    let vault = guard.as_ref().ok_or(crate::error::SammyError::VaultLocked)?;
    f(vault)
}
