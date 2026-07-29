use crate::{
    git_sync::GitSync,
    models::{Config, NewEntry, PasswordEntry},
    storage::Storage,
};
use std::sync::{Arc, Mutex};
use tauri::State;
use totp_rs::{Algorithm, Secret, TOTP};

pub struct AppData {
    pub storage: Arc<Mutex<Storage>>,
}

#[tauri::command]
pub(crate) fn list_entries(
    search: Option<String>,
    state: State<AppData>,
) -> Result<Vec<PasswordEntry>, String> {
    let storage = state.storage.lock().map_err(|e| e.to_string())?;
    Ok(storage.list_entries(search.as_deref()))
}

#[tauri::command]
pub(crate) fn get_entry(
    id: String,
    password: String,
    state: State<AppData>,
) -> Result<PasswordEntry, String> {
    let storage = state.storage.lock().map_err(|e| e.to_string())?;
    storage.get_entry(&id, &password)
}

#[tauri::command]
pub(crate) fn add_entry(
    entry: NewEntry,
    password: String,
    state: State<AppData>,
) -> Result<bool, String> {
    let mut storage = state.storage.lock().map_err(|e| e.to_string())?;
    storage.add_entry(entry, &password)?;
    Ok(true)
}

#[tauri::command]
pub(crate) fn edit_entry(
    id: String,
    entry: NewEntry,
    password: String,
    state: State<AppData>,
) -> Result<bool, String> {
    let mut storage = state.storage.lock().map_err(|e| e.to_string())?;
    storage.edit_entry(&id, entry, &password)?;
    Ok(true)
}

#[tauri::command]
pub(crate) fn delete_entry(id: String, state: State<AppData>) -> Result<bool, String> {
    let mut storage = state.storage.lock().map_err(|e| e.to_string())?;
    storage.delete_entry(&id)?;
    Ok(true)
}

#[tauri::command]
pub(crate) fn change_master_password(
    old: String,
    new: String,
    state: State<AppData>,
) -> Result<bool, String> {
    let mut storage = state.storage.lock().map_err(|e| e.to_string())?;
    storage.change_master_password(&old, &new)?;
    Ok(true)
}

#[tauri::command]
pub(crate) fn init_password(password: String, state: State<AppData>) -> Result<bool, String> {
    let mut storage = state.storage.lock().map_err(|e| e.to_string())?;
    storage.init_salt(&password)?;
    Ok(true)
}

#[tauri::command]
pub(crate) fn verify_password(password: String, state: State<AppData>) -> Result<bool, String> {
    let storage = state.storage.lock().map_err(|e| e.to_string())?;
    storage.verify_password(&password)?;
    Ok(true)
}

#[tauri::command]
pub(crate) fn is_initialized(state: State<AppData>) -> Result<bool, String> {
    let storage = state.storage.lock().map_err(|e| e.to_string())?;
    Ok(storage.is_initialized())
}

#[tauri::command]
pub(crate) fn get_config(state: State<AppData>) -> Result<Config, String> {
    let storage = state.storage.lock().map_err(|e| e.to_string())?;
    Ok(storage.get_config())
}

#[tauri::command]
pub(crate) fn update_config(config: Config, state: State<AppData>) -> Result<bool, String> {
    let mut storage = state.storage.lock().map_err(|e| e.to_string())?;
    storage.update_config(config)?;
    Ok(true)
}

#[tauri::command]
pub(crate) fn git_push(
    message: Option<String>,
    state: State<AppData>,
) -> Result<String, String> {
    let storage = state.storage.lock().map_err(|e| e.to_string())?;
    let path = storage.get_file_path().parent().unwrap().to_path_buf();
    let remote = storage.get_config().git_remote;
    drop(storage);
    GitSync::commit_and_push(&path, &remote, message.as_deref())
}

#[tauri::command]
pub(crate) fn git_pull(state: State<AppData>) -> Result<String, String> {
    let storage = state.storage.lock().map_err(|e| e.to_string())?;
    let path = storage.get_file_path().parent().unwrap().to_path_buf();
    let remote = storage.get_config().git_remote;
    drop(storage);
    GitSync::pull(&path, &remote)
}

use std::path::PathBuf;

#[tauri::command]
pub(crate) fn export_json(path: String, state: State<AppData>) -> Result<String, String> {
    let storage = state.storage.lock().map_err(|e| e.to_string())?;
    let content = std::fs::read_to_string(&storage.get_file_path())
        .map_err(|e| format!("Failed to read data file: {}", e))?;
    let dest = PathBuf::from(&path);
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    std::fs::write(&dest, &content).map_err(|e| format!("Failed to write export file: {}", e))?;
    Ok(format!("Exported to {}", path))
}

#[tauri::command]
pub(crate) fn import_json(path: String, password: String, state: State<AppData>) -> Result<String, String> {
    let content = std::fs::read_to_string(&PathBuf::from(&path))
        .map_err(|e| format!("Failed to read import file: {}", e))?;
    let mut storage = state.storage.lock().map_err(|e| e.to_string())?;
    storage.verify_password(&password)?;
    std::fs::write(storage.get_file_path(), &content)
        .map_err(|e| format!("Failed to write data file: {}", e))?;
    storage.reload()?;
    Ok(format!("Imported from {}", path))
}

use rand::seq::SliceRandom;

#[tauri::command]
pub(crate) fn generate_password(
    length: u32,
    use_upper: bool,
    use_lower: bool,
    use_digits: bool,
    use_symbols: bool,
    exclude_confusing: bool,
) -> Result<String, String> {
    let mut chars = Vec::new();
    if use_upper {
        if exclude_confusing {
            chars.extend("ABCDEFGHJKLMNPQRSTUVWXYZ".chars());
        } else {
            chars.extend("ABCDEFGHIJKLMNOPQRSTUVWXYZ".chars());
        }
    }
    if use_lower {
        if exclude_confusing {
            chars.extend("abcdefghjkmnpqrstuvwxyz".chars());
        } else {
            chars.extend("abcdefghijklmnopqrstuvwxyz".chars());
        }
    }
    if use_digits {
        if exclude_confusing {
            chars.extend("23456789".chars());
        } else {
            chars.extend("0123456789".chars());
        }
    }
    if use_symbols {
        chars.extend("!@#$%^&*()_+-=[]{}|;:,.<>?".chars());
    }
    if chars.is_empty() {
        return Err("At least one character set must be selected".into());
    }
    let mut rng = rand::thread_rng();
    let pwd: String = (0..length)
        .map(|_| chars.choose(&mut rng).unwrap())
        .collect();
    Ok(pwd)
}

#[tauri::command]
pub(crate) fn generate_totp(
    secret: String,
    step_offset: Option<i64>,
) -> Result<String, String> {
    let totp = TOTP::new_unchecked(
        Algorithm::SHA1,
        6,
        1,
        30,
        Secret::Encoded(secret).to_bytes().map_err(|e| e.to_string())?,
        None,
        String::new(),
    );
    let offset = step_offset.unwrap_or(0);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let time = now + offset * 30;
    let code = totp.generate(time as u64);
    Ok(code)
}
