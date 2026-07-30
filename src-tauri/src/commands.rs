use crate::{
    git_sync::GitSync,
    models::{CategoryInfo, Config, NewEntry, PasswordEntry},
    storage::Storage,
    wordlist,
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
    category: Option<String>,
    favorite: Option<bool>,
    state: State<AppData>,
) -> Result<Vec<PasswordEntry>, String> {
    let storage = state.storage.lock().map_err(|e| e.to_string())?;
    Ok(storage.list_entries(search.as_deref(), category.as_deref(), favorite))
}

#[tauri::command]
pub(crate) fn list_categories(state: State<AppData>) -> Result<Vec<CategoryInfo>, String> {
    let storage = state.storage.lock().map_err(|e| e.to_string())?;
    Ok(storage.list_categories())
}

#[tauri::command]
pub(crate) fn toggle_favorite(
    id: String,
    state: State<AppData>,
) -> Result<bool, String> {
    let mut storage = state.storage.lock().map_err(|e| e.to_string())?;
    let result = storage.toggle_favorite(&id);
    log::info!("Toggled favorite for entry {}", id);
    result
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
    log::info!("Added new entry");
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
    log::info!("Edited entry {}", id);
    Ok(true)
}

#[tauri::command]
pub(crate) fn delete_entry(id: String, state: State<AppData>) -> Result<bool, String> {
    let mut storage = state.storage.lock().map_err(|e| e.to_string())?;
    storage.delete_entry(&id)?;
    log::info!("Deleted entry {}", id);
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
    log::info!("Master password changed");
    Ok(true)
}

#[tauri::command]
pub(crate) fn init_password(password: String, state: State<AppData>) -> Result<bool, String> {
    let mut storage = state.storage.lock().map_err(|e| e.to_string())?;
    storage.init_salt(&password)?;
    log::info!("Password manager initialized");
    Ok(true)
}

#[tauri::command]
pub(crate) fn verify_password(password: String, state: State<AppData>) -> Result<bool, String> {
    let storage = state.storage.lock().map_err(|e| e.to_string())?;
    storage.verify_password(&password)?;
    log::info!("Password verified");
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
pub(crate) fn export_csv(path: String, password: String, state: State<AppData>) -> Result<String, String> {
    use zeroize::Zeroize;

    let storage = state.storage.lock().map_err(|e| e.to_string())?;
    let raw = storage.list_entries(None, None, None);
    let dest = PathBuf::from(&path);
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    let mut wtr = csv::Writer::from_path(&dest).map_err(|e| format!("Failed to create CSV: {}", e))?;
    wtr.write_record(["site_url", "username", "password", "category", "favorite", "note", "twofa_secret", "phone"])
        .map_err(|e| format!("CSV write error: {}", e))?;

    for entry in &raw {
        let decrypted = storage.get_entry(&entry.id, &password).unwrap_or_else(|_| entry.clone());
        let mut pwd = decrypted.password;
        let fav_str = if decrypted.favorite { "true" } else { "false" };
        wtr.write_record([
            decrypted.site_url.as_str(),
            decrypted.username.as_str(),
            pwd.as_str(),
            decrypted.category.as_str(),
            fav_str,
            decrypted.note.as_str(),
            decrypted.twofa_secret.as_deref().unwrap_or(""),
            decrypted.phone.as_deref().unwrap_or(""),
        ]).map_err(|e| format!("CSV write error: {}", e))?;
        pwd.zeroize();
    }

    wtr.flush().map_err(|e| format!("CSV flush error: {}", e))?;
    Ok(format!("Exported CSV to {}", path))
}

#[tauri::command]
pub(crate) fn import_csv(path: String, password: String, state: State<AppData>) -> Result<String, String> {
    let mut storage = state.storage.lock().map_err(|e| e.to_string())?;
    storage.verify_password(&password)?;

    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .from_path(&PathBuf::from(&path))
        .map_err(|e| format!("Failed to read CSV: {}", e))?;

    let headers = rdr.headers().map_err(|e| format!("CSV header error: {}", e))?;
    let is_bitwarden = headers.iter().any(|h| h == "login_uri" || h == "login_username");

    let mut count = 0u32;
    for result in rdr.records() {
        let record = result.map_err(|e| format!("CSV parse error: {}", e))?;

        let (site_url, username, field_password, category, favorite, note, twofa_secret, phone) = if is_bitwarden {
            let uri = record.get(6).unwrap_or("");
            let uname = record.get(7).unwrap_or("");
            let p = record.get(8).unwrap_or("");
            let cat = record.get(0).unwrap_or("");
            let fav = record.get(1).unwrap_or("") == "1";
            let notes = record.get(4).unwrap_or("");
            let totp = record.get(9).unwrap_or("");
            (uri, uname, p, cat, fav, notes, totp, "")
        } else {
            let uri = record.get(0).unwrap_or("");
            let uname = record.get(1).unwrap_or("");
            let p = record.get(2).unwrap_or("");
            let cat = record.get(3).unwrap_or("");
            let fav = record.get(4).unwrap_or("") == "true";
            let notes = record.get(5).unwrap_or("");
            let totp = record.get(6).unwrap_or("");
            let ph = record.get(7).unwrap_or("");
            (uri, uname, p, cat, fav, notes, totp, ph)
        };

        let entry = NewEntry {
            site_url: site_url.to_string(),
            username: username.to_string(),
            password: field_password.to_string(),
            emails_raw: None,
            phone: if phone.is_empty() { None } else { Some(phone.to_string()) },
            twofa_secret: if twofa_secret.is_empty() { None } else { Some(twofa_secret.to_string()) },
            note: note.to_string(),
            autofill_mode: "default".to_string(),
            category: category.to_string(),
            favorite,
        };
        storage.add_entry(entry, &password)?;
        count += 1;
    }

    Ok(format!("Imported {} entries from CSV", count))
}

#[tauri::command]
pub(crate) fn import_json(path: String, password: String, state: State<AppData>) -> Result<String, String> {
    let import_path = PathBuf::from(&path);
    let content = std::fs::read_to_string(&import_path)
        .map_err(|e| format!("Failed to read import file: {}", e))?;
    let mut storage = state.storage.lock().map_err(|e| e.to_string())?;
    storage.verify_password(&password)?;

    // Auto-backup before overwriting
    let data_path = storage.get_file_path().clone();
    let backup_path = data_path.with_extension("json.bak");
    if data_path.exists() {
        if let Ok(original) = std::fs::read_to_string(&data_path) {
            std::fs::write(&backup_path, &original).ok();
        }
    }

    std::fs::write(&data_path, &content)
        .map_err(|e| format!("Failed to write data file: {}", e))?;
    storage.reload()?;
    log::info!("Import completed from {}", path);
    Ok(format!("Imported from {}", path))
}

use rand::seq::SliceRandom;
use rand::Rng;

#[tauri::command]
pub(crate) fn generate_passphrase(
    word_count: Option<u32>,
    separator: Option<String>,
    capitalize: Option<bool>,
    append_number: Option<bool>,
) -> Result<String, String> {
    let count = word_count.unwrap_or(4).max(2).min(12) as usize;
    let sep = separator.unwrap_or_else(|| "-".to_string());
    let cap = capitalize.unwrap_or(false);
    let num = append_number.unwrap_or(false);

    let mut rng = rand::thread_rng();
    let mut words: Vec<String> = (0..count)
        .map(|_| {
            let w = wordlist::WORDS.choose(&mut rng).unwrap().to_string();
            if cap {
                let mut chars = w.chars();
                match chars.next() {
                    None => w,
                    Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
                }
            } else {
                w
            }
        })
        .collect();

    if num {
        let digits: u16 = rng.gen_range(10..99);
        words.push(digits.to_string());
    }

    Ok(words.join(&sep))
}

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
