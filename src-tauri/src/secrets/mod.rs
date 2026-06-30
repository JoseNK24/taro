use keyring::Entry;
use thiserror::Error;

const SERVICE: &str = "taro";

#[derive(Debug, Error)]
pub enum SecretError {
    #[error("Keychain error: {0}")]
    Keyring(#[from] keyring::Error),
    #[error("Secret not found: {0}")]
    NotFound(String),
}

pub type SecretResult<T> = Result<T, SecretError>;

fn account_name(integration_id: &str, secret_key: &str) -> String {
    format!("{integration_id}:{secret_key}")
}

fn app_account_name(key: &str) -> String {
    format!("app:{key}")
}

pub fn set_app_secret(key: &str, value: &str) -> SecretResult<()> {
    let entry = Entry::new(SERVICE, &app_account_name(key))?;
    entry.set_password(value)?;
    Ok(())
}

pub fn get_app_secret(key: &str) -> SecretResult<String> {
    let entry = Entry::new(SERVICE, &app_account_name(key))?;
    entry.get_password().map_err(|e| match e {
        keyring::Error::NoEntry => SecretError::NotFound(format!("app:{key}")),
        other => SecretError::Keyring(other),
    })
}

pub fn delete_app_secret(key: &str) -> SecretResult<()> {
    let entry = Entry::new(SERVICE, &app_account_name(key))?;
    entry.delete_credential()?;
    Ok(())
}

pub fn set_secret(integration_id: &str, secret_key: &str, value: &str) -> SecretResult<()> {
    let entry = Entry::new(SERVICE, &account_name(integration_id, secret_key))?;
    entry.set_password(value)?;
    Ok(())
}

pub fn get_secret(integration_id: &str, secret_key: &str) -> SecretResult<String> {
    let entry = Entry::new(SERVICE, &account_name(integration_id, secret_key))?;
    entry.get_password().map_err(|e| match e {
        keyring::Error::NoEntry => SecretError::NotFound(format!(
            "{integration_id}:{secret_key}"
        )),
        other => SecretError::Keyring(other),
    })
}

pub fn has_secret(integration_id: &str, secret_key: &str) -> bool {
    get_secret(integration_id, secret_key).is_ok()
}

pub fn delete_secret(integration_id: &str, secret_key: &str) -> SecretResult<()> {
    let entry = Entry::new(SERVICE, &account_name(integration_id, secret_key))?;
    entry.delete_credential()?;
    Ok(())
}

pub fn resolve_env(
    integration_id: &str,
    secret_defs: &[crate::models::SecretDef],
) -> SecretResult<std::collections::HashMap<String, String>> {
    let mut env = std::collections::HashMap::new();
    for def in secret_defs {
        if def.required {
            let value = get_secret(integration_id, &def.key)?;
            env.insert(def.key.clone(), value);
        } else if has_secret(integration_id, &def.key) {
            if let Ok(value) = get_secret(integration_id, &def.key) {
                env.insert(def.key.clone(), value);
            }
        }
    }
    Ok(env)
}

pub fn missing_required_secrets(
    integration_id: &str,
    secret_defs: &[crate::models::SecretDef],
) -> Vec<String> {
    secret_defs
        .iter()
        .filter(|d| d.required && !has_secret(integration_id, &d.key))
        .map(|d| d.key.clone())
        .collect()
}

pub fn resolve_env_keys(
    integration_id: &str,
    keys: &[String],
) -> std::collections::HashMap<String, String> {
    let mut env = std::collections::HashMap::new();
    for key in keys {
        if let Ok(value) = get_secret(integration_id, key) {
            env.insert(key.clone(), value);
        }
    }
    env
}
