use crate::error::{SammyError, SammyResult};

const SERVICE: &str = "Sammy";

pub fn set_api_key(brain_id: &str, api_key: &str) -> SammyResult<()> {
    let entry = keyring::Entry::new(SERVICE, &format!("brain:{brain_id}"))
        .map_err(|e| SammyError::msg(format!("keychain: {e}")))?;
    if api_key.trim().is_empty() {
        let _ = entry.delete_credential();
        return Ok(());
    }
    entry
        .set_password(api_key.trim())
        .map_err(|e| SammyError::msg(format!("keychain set: {e}")))?;
    Ok(())
}

pub fn get_api_key(brain_id: &str) -> SammyResult<Option<String>> {
    let entry = keyring::Entry::new(SERVICE, &format!("brain:{brain_id}"))
        .map_err(|e| SammyError::msg(format!("keychain: {e}")))?;
    match entry.get_password() {
        Ok(p) if !p.is_empty() => Ok(Some(p)),
        Ok(_) => Ok(None),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(SammyError::msg(format!("keychain get: {e}"))),
    }
}

pub fn has_api_key(brain_id: &str) -> bool {
    matches!(get_api_key(brain_id), Ok(Some(_)))
}
