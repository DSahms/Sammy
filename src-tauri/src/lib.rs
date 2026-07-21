mod brain;
mod chat;
mod commands;
mod error;
mod facts;
mod intake;
mod keychain;
mod paths;
mod sources;
mod state;
mod sync_schema;
mod vault;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = AppState::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::vault_status,
            commands::vault_create,
            commands::vault_unlock,
            commands::vault_lock,
            commands::list_brains,
            commands::set_active_brain,
            commands::get_active_brain,
            commands::set_brain_api_key,
            commands::brain_has_key,
            commands::chat_send,
            commands::chat_history,
            commands::intake_state,
            commands::intake_start,
            commands::intake_answer,
            commands::intake_skip_to_chat,
            commands::facts_list,
            commands::facts_upsert,
            commands::facts_delete,
            commands::facts_export,
            commands::sources_list,
            commands::sources_add,
            commands::sources_delete,
            commands::sources_set_pinned,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Sammy");
}
