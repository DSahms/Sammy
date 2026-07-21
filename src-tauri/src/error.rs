use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SammyError {
    #[error("{0}")]
    Message(String),
    #[error("Vault is locked — unlock first")]
    VaultLocked,
    #[error("Vault already exists")]
    VaultExists,
    #[error("Vault not found — create one first")]
    VaultMissing,
    #[error("Wrong passphrase")]
    BadPassphrase,
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Http(#[from] reqwest::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl SammyError {
    pub fn msg(s: impl Into<String>) -> Self {
        Self::Message(s.into())
    }
}

impl Serialize for SammyError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type SammyResult<T> = Result<T, SammyError>;
