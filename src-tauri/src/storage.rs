use crate::crypto;
use crate::models::{CategoryInfo, Config, DataFile, EmailInfo, NewEntry, PasswordEntry};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use chrono::Utc;
use serde_json::Value;
use std::path::PathBuf;
use uuid::Uuid;

pub struct Storage {
    data: DataFile,
    file_path: PathBuf,
}

impl Storage {
    fn migrate_data(content: &str) -> DataFile {
        let mut data: DataFile = serde_json::from_str(content).unwrap_or_else(|_| DataFile {
            version: 1,
            salt: String::new(),
            entries: Vec::new(),
            config: Config::default(),
        });
        // Migrate old `email` field to `emails_raw`
        if let Ok(json) = serde_json::from_str::<Value>(content) {
            if let Some(entries) = json.get("entries").and_then(|v| v.as_array()) {
                for (i, entry_val) in entries.iter().enumerate() {
                    if entry_val.get("email").and_then(|v| v.as_str()).is_some()
                        && data.entries.get(i).map(|e| e.emails_raw.is_none()).unwrap_or(false)
                    {
                        let old_email = entry_val["email"].as_str().unwrap_or("");
                        if !old_email.is_empty() {
                            let info = EmailInfo {
                                email: old_email.to_string(),
                                is_primary: true,
                            };
                            if let Ok(json_str) = serde_json::to_string(&vec![info]) {
                                if let Some(e) = data.entries.get_mut(i) {
                                    e.emails_raw = Some(json_str);
                                }
                            }
                        }
                    }
                }
            }
        }
        data
    }

    pub fn new() -> Self {
        let file_path = Self::get_data_path();
        let data = if file_path.exists() {
            let content = std::fs::read_to_string(&file_path).unwrap_or_default();
            Self::migrate_data(&content)
        } else {
            DataFile {
                version: 1,
                salt: String::new(),
                entries: Vec::new(),
                config: Config::default(),
            }
        };
        Storage { data, file_path }
    }

    fn get_data_path() -> PathBuf {
        let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push(".password-manager");
        std::fs::create_dir_all(&path).ok();
        path.push("data.json");
        path
    }

    pub fn get_file_path(&self) -> &PathBuf {
        &self.file_path
    }

    pub fn save(&self) -> Result<(), String> {
        let content =
            serde_json::to_string_pretty(&self.data).map_err(|e| format!("Serialization error: {}", e))?;
        std::fs::write(&self.file_path, content).map_err(|e| format!("Write error: {}", e))?;
        #[cfg(unix)]
        {
            use std::fs::Permissions;
            use std::os::unix::fs::PermissionsExt;
            if let Err(e) = std::fs::set_permissions(&self.file_path, Permissions::from_mode(0o600)) {
                log::warn!("Failed to set file permissions: {}", e);
            }
        }
        Ok(())
    }

    pub fn init_salt(&mut self, password: &str) -> Result<(), String> {
        if !self.data.salt.is_empty() {
            return Err("Already initialized".into());
        }
        let salt = crypto::generate_salt();
        self.data.salt = BASE64.encode(&salt);
        self.save()?;
        let _ = crypto::derive_key(password, &salt);
        Ok(())
    }

    pub fn verify_password(&self, password: &str) -> Result<Vec<u8>, String> {
        if self.data.salt.is_empty() {
            return Err("Not initialized".into());
        }
        let salt = BASE64
            .decode(&self.data.salt)
            .map_err(|_| "Invalid salt".to_string())?;
        if self.data.entries.is_empty() {
            return Ok(salt);
        }
        // Scan all entries for any non-empty encrypted field to verify against
        for entry in &self.data.entries {
            let fields = [
                entry.password.as_str(),
                entry.emails_raw.as_deref().unwrap_or(""),
                entry.phone.as_deref().unwrap_or(""),
                entry.twofa_secret.as_deref().unwrap_or(""),
            ];
            let any_encrypted = fields.iter().any(|f| !f.is_empty());
            if any_encrypted {
                for field in fields {
                    if !field.is_empty() {
                        if crypto::decrypt_field(field, password).is_ok() {
                            return Ok(salt);
                        }
                        return Err("Incorrect password".into());
                    }
                }
            }
        }
        // No encrypted fields found at all — accept any password
        Ok(salt)
    }

    pub fn list_entries(
        &self,
        search: Option<&str>,
        category: Option<&str>,
        favorite: Option<bool>,
    ) -> Vec<PasswordEntry> {
        self.data
            .entries
            .iter()
            .filter(|e| {
                if let Some(q) = search {
                    let q = q.to_lowercase();
                    if !e.site_url.to_lowercase().contains(&q)
                        && !e.username.to_lowercase().contains(&q)
                        && !e.note.to_lowercase().contains(&q)
                    {
                        return false;
                    }
                }
                if let Some(cat) = category {
                    let tags: Vec<&str> = e.category.split_whitespace().collect();
                    if cat.is_empty() {
                        if !tags.is_empty() {
                            return false;
                        }
                    } else if !tags.iter().any(|t| t.eq_ignore_ascii_case(cat)) {
                        return false;
                    }
                }
                if let Some(fav) = favorite {
                    if e.favorite != fav {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect()
    }

    pub fn list_categories(&self) -> Vec<CategoryInfo> {
        let mut map: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
        for entry in &self.data.entries {
            let tags: Vec<&str> = entry.category.split_whitespace().collect();
            if tags.is_empty() {
                *map.entry(String::new()).or_insert(0) += 1;
            } else {
                for tag in tags {
                    *map.entry(tag.to_string()).or_insert(0) += 1;
                }
            }
        }
        let mut result: Vec<CategoryInfo> = map
            .into_iter()
            .map(|(name, count)| CategoryInfo { name, count })
            .collect();
        result.sort_by(|a, b| {
            let a_is_empty = a.name.is_empty();
            let b_is_empty = b.name.is_empty();
            if a_is_empty != b_is_empty {
                b_is_empty.cmp(&a_is_empty)
            } else {
                b.count.cmp(&a.count)
            }
        });
        result
    }

    pub fn get_entry(&self, id: &str, password: &str) -> Result<PasswordEntry, String> {
        let _salt = self.get_salt_bytes()?;
        let entry = self
            .data
            .entries
            .iter()
            .find(|e| e.id == id)
            .ok_or_else(|| "Entry not found".to_string())?
            .clone();
        let decrypted = PasswordEntry {
            password: crypto::decrypt_field(&entry.password, password)
                .unwrap_or(entry.password.clone()),
            emails_raw: if let Some(ref e) = entry.emails_raw {
                if !e.is_empty() {
                    Some(crypto::decrypt_field(e, password).unwrap_or_else(|_| e.clone()))
                } else {
                    None
                }
            } else {
                None
            },
            phone: if let Some(ref p) = entry.phone {
                if !p.is_empty() {
                    Some(crypto::decrypt_field(p, password).unwrap_or_else(|_| p.clone()))
                } else {
                    None
                }
            } else {
                None
            },
            twofa_secret: if let Some(ref t) = entry.twofa_secret {
                if !t.is_empty() {
                    Some(crypto::decrypt_field(t, password).unwrap_or_else(|_| t.clone()))
                } else {
                    None
                }
            } else {
                None
            },
            ..entry
        };
        Ok(decrypted)
    }

    pub fn add_entry(&mut self, entry: NewEntry, password: &str) -> Result<(), String> {
        self.verify_password(password)?;
        if entry.site_url.trim().is_empty()
            && entry.username.trim().is_empty()
            && entry.emails_raw.as_deref().unwrap_or("").trim().is_empty()
            && entry.phone.as_deref().unwrap_or("").trim().is_empty()
        {
            return Err("At least one field (site_url, username, email, or phone) must be non-empty".into());
        }
        let salt = self.get_salt_bytes()?;
        let now = Utc::now().timestamp() as u64;
        let new_entry = PasswordEntry {
            id: Uuid::new_v4().to_string(),
            site_url: entry.site_url,
            username: entry.username,
            password: crypto::encrypt_field(&entry.password, password, &salt),
            emails_raw: entry
                .emails_raw
                .map(|e| crypto::encrypt_field(&e, password, &salt)),
            phone: entry
                .phone
                .map(|p| crypto::encrypt_field(&p, password, &salt)),
            twofa_secret: entry
                .twofa_secret
                .map(|t| crypto::encrypt_field(&t, password, &salt)),
            note: entry.note,
            autofill_mode: entry.autofill_mode,
            category: entry.category,
            favorite: entry.favorite,
            created_at: now,
            updated_at: now,
        };
        self.data.entries.push(new_entry);
        self.save()
    }

    pub fn edit_entry(
        &mut self,
        id: &str,
        entry: NewEntry,
        password: &str,
    ) -> Result<(), String> {
        self.verify_password(password)?;
        let salt = self.get_salt_bytes()?;
        let now = Utc::now().timestamp() as u64;
        let existing = self
            .data
            .entries
            .iter_mut()
            .find(|e| e.id == id)
            .ok_or_else(|| "Entry not found".to_string())?;
        existing.site_url = entry.site_url;
        existing.username = entry.username;
        existing.password = crypto::encrypt_field(&entry.password, password, &salt);
        existing.emails_raw = entry
            .emails_raw
            .map(|e| crypto::encrypt_field(&e, password, &salt));
        existing.phone = entry
            .phone
            .map(|p| crypto::encrypt_field(&p, password, &salt));
        existing.twofa_secret = entry
            .twofa_secret
            .map(|t| crypto::encrypt_field(&t, password, &salt));
        existing.note = entry.note;
        existing.autofill_mode = entry.autofill_mode;
        existing.category = entry.category;
        existing.favorite = entry.favorite;
        existing.updated_at = now;
        self.save()
    }

    pub fn delete_entry(&mut self, id: &str) -> Result<(), String> {
        let len = self.data.entries.len();
        self.data.entries.retain(|e| e.id != id);
        if self.data.entries.len() == len {
            return Err("Entry not found".into());
        }
        self.save()
    }

    pub fn change_master_password(&mut self, old: &str, new: &str) -> Result<(), String> {
        let _old_salt = self.get_salt_bytes()?;
        let new_salt = crypto::generate_salt();
        for entry in self.data.entries.iter_mut() {
            let old_password = crypto::decrypt_field(&entry.password, old)
                .map_err(|_| "Failed to decrypt entry with old password".to_string())?;
            entry.password = crypto::encrypt_field(&old_password, new, &new_salt);
            if let Some(ref e) = entry.emails_raw.clone() {
                if !e.is_empty() {
                    if let Ok(dec) = crypto::decrypt_field(e, old) {
                        entry.emails_raw = Some(crypto::encrypt_field(&dec, new, &new_salt));
                    }
                }
            }
            if let Some(ref p) = entry.phone.clone() {
                if !p.is_empty() {
                    if let Ok(dec) = crypto::decrypt_field(p, old) {
                        entry.phone = Some(crypto::encrypt_field(&dec, new, &new_salt));
                    }
                }
            }
            if let Some(ref t) = entry.twofa_secret.clone() {
                if !t.is_empty() {
                    if let Ok(dec) = crypto::decrypt_field(t, old) {
                        entry.twofa_secret = Some(crypto::encrypt_field(&dec, new, &new_salt));
                    }
                }
            }
        }
        self.data.salt = BASE64.encode(&new_salt);
        self.save()
    }

    pub fn get_config(&self) -> Config {
        self.data.config.clone()
    }

    pub fn update_config(&mut self, config: Config) -> Result<(), String> {
        self.data.config = config;
        self.save()
    }

    pub fn toggle_favorite(&mut self, id: &str) -> Result<bool, String> {
        let now = Utc::now().timestamp() as u64;
        let fav = self
            .data
            .entries
            .iter_mut()
            .find(|e| e.id == id)
            .ok_or_else(|| "Entry not found".to_string())
            .map(|e| {
                e.favorite = !e.favorite;
                e.updated_at = now;
                e.favorite
            })?;
        self.save()?;
        Ok(fav)
    }

    pub fn get_salt_bytes(&self) -> Result<Vec<u8>, String> {
        BASE64
            .decode(&self.data.salt)
            .map_err(|_| "Invalid salt".to_string())
    }

    pub fn is_initialized(&self) -> bool {
        !self.data.salt.is_empty()
    }

    pub fn reload(&mut self) -> Result<(), String> {
        if self.file_path.exists() {
            let content =
                std::fs::read_to_string(&self.file_path).map_err(|e| format!("Read error: {}", e))?;
            self.data = Self::migrate_data(&content);
        }
        Ok(())
    }
}
