pub mod crypto;
pub mod db;

use std::fs;
use std::sync::Arc;

use parking_lot::Mutex;

use crate::error::{SammyError, SammyResult};
use crate::paths::{db_path, ensure_dirs, salt_path, vault_dir};
use crate::vault::crypto::{derive_key, generate_salt, MasterKey};
use crate::vault::db::VaultDb;

pub struct UnlockedVault {
    pub master: MasterKey,
    pub db: VaultDb,
}

pub type SharedVault = Arc<Mutex<Option<UnlockedVault>>>;

pub fn vault_exists() -> bool {
    db_path().exists() && salt_path().exists()
}

pub fn create_vault(passphrase: &str) -> SammyResult<UnlockedVault> {
    if passphrase.len() < 8 {
        return Err(SammyError::msg("Passphrase must be at least 8 characters"));
    }
    ensure_dirs()?;
    if vault_exists() {
        return Err(SammyError::VaultExists);
    }
    let salt = generate_salt();
    fs::write(salt_path(), salt)?;
    let master = derive_key(passphrase, &salt)?;
    let db = VaultDb::create(&db_path(), &master)?;
    db.set_meta("created_with", "sammy-0.1")?;
    let unlocked = UnlockedVault { master, db };
    crate::sources::ensure_schema(&unlocked)?;
    Ok(unlocked)
}

pub fn unlock_vault(passphrase: &str) -> SammyResult<UnlockedVault> {
    ensure_dirs()?;
    if !vault_exists() {
        return Err(SammyError::VaultMissing);
    }
    let salt = fs::read(salt_path())?;
    let master = derive_key(passphrase, &salt)?;
    let db = VaultDb::open(&db_path(), &master)?;
    let unlocked = UnlockedVault { master, db };
    crate::sources::ensure_schema(&unlocked)?;
    Ok(unlocked)
}

pub fn data_dir_display() -> String {
    vault_dir().display().to_string()
}
