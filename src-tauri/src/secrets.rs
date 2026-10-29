use keyring::Entry;

const SERVICE: &str = "app.uptimewidget";
const ACCOUNT: &str = "uptimerobot-api-key";

fn entry() -> keyring::Result<Entry> {
    Entry::new(SERVICE, ACCOUNT)
}

pub fn load_key() -> Option<String> {
    match entry().and_then(|e| e.get_password()) {
        Ok(key) if !key.is_empty() => Some(key),
        Ok(_) | Err(keyring::Error::NoEntry) => None,
        Err(e) => {
            log::warn!("failed to read API key from keychain: {e}");
            None
        }
    }
}

pub fn store_key(key: &str) -> Result<(), String> {
    entry()
        .and_then(|e| e.set_password(key))
        .map_err(|e| format!("failed to store API key: {e}"))
}

pub fn delete_key() {
    match entry().and_then(|e| e.delete_credential()) {
        Ok(()) | Err(keyring::Error::NoEntry) => {}
        Err(e) => log::warn!("failed to delete API key: {e}"),
    }
}
