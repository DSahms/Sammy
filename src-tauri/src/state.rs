use parking_lot::Mutex;
use reqwest::Client;
use std::sync::Arc;

use crate::brain::config::BrainConfigFile;
use crate::vault::{SharedVault, UnlockedVault};

pub struct AppState {
    pub vault: SharedVault,
    pub http: Client,
    pub brains: Mutex<BrainConfigFile>,
    pub active_brain: Mutex<String>,
    pub active_session: Mutex<Option<String>>,
}

impl AppState {
    pub fn new() -> Self {
        let brains = BrainConfigFile::load_or_default();
        let active = brains
            .brains
            .iter()
            .find(|b| b.prefer)
            .map(|b| b.id.clone())
            .or_else(|| brains.brains.first().map(|b| b.id.clone()))
            .unwrap_or_else(|| "kobold".into());

        Self {
            vault: Arc::new(Mutex::new(None)),
            http: Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .expect("http client"),
            brains: Mutex::new(brains),
            active_brain: Mutex::new(active),
            active_session: Mutex::new(None),
        }
    }

    pub fn require_vault(&self) -> Result<parking_lot::MutexGuard<'_, Option<UnlockedVault>>, crate::error::SammyError> {
        let guard = self.vault.lock();
        if guard.is_none() {
            return Err(crate::error::SammyError::VaultLocked);
        }
        Ok(guard)
    }
}
