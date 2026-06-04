// AetherEcho — OS Keyring Integration
// Stores the Groq API key securely in Windows Credential Manager.
// Never writes the key to disk in plaintext.

use anyhow::{Context, Result};
use keyring::Entry;

const SERVICE_NAME: &str = "aether-echo";
const API_KEY_ACCOUNT: &str = "groq-api-key";

/// Save the Groq API key to the OS keyring (Windows Credential Manager).
pub fn save_api_key(api_key: &str) -> Result<()> {
    let entry = Entry::new(SERVICE_NAME, API_KEY_ACCOUNT)
        .context("Failed to create keyring entry")?;
    entry
        .set_password(api_key)
        .context("Failed to save API key to keyring")?;
    log::info!("API key saved to OS keyring");
    Ok(())
}

/// Retrieve the Groq API key from the OS keyring.
/// Returns None if no key has been stored yet.
pub fn load_api_key() -> Result<Option<String>> {
    let entry = Entry::new(SERVICE_NAME, API_KEY_ACCOUNT)
        .context("Failed to create keyring entry")?;
    match entry.get_password() {
        Ok(key) => Ok(Some(key)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(anyhow::anyhow!("Keyring error: {}", e)),
    }
}

/// Delete the stored API key from the OS keyring.
pub fn delete_api_key() -> Result<()> {
    let entry = Entry::new(SERVICE_NAME, API_KEY_ACCOUNT)
        .context("Failed to create keyring entry")?;
    match entry.delete_credential() {
        Ok(_) => {
            log::info!("API key deleted from OS keyring");
            Ok(())
        }
        Err(keyring::Error::NoEntry) => Ok(()), // Already gone — OK
        Err(e) => Err(anyhow::anyhow!("Keyring delete error: {}", e)),
    }
}

/// Check if an API key exists in the keyring.
pub fn has_api_key() -> bool {
    load_api_key().ok().flatten().is_some()
}
