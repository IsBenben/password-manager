use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailInfo {
    pub email: String,
    pub is_primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordEntry {
    pub id: String,
    pub site_url: String,
    pub username: String,
    pub password: String,
    pub emails_raw: Option<String>,
    pub phone: Option<String>,
    pub twofa_secret: Option<String>,
    pub note: String,
    pub created_at: u64,
    pub updated_at: u64,
    #[serde(default = "default_autofill_mode")]
    pub autofill_mode: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub favorite: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewEntry {
    pub site_url: String,
    pub username: String,
    pub password: String,
    pub emails_raw: Option<String>,
    pub phone: Option<String>,
    pub twofa_secret: Option<String>,
    pub note: String,
    #[serde(default = "default_autofill_mode")]
    pub autofill_mode: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub favorite: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryInfo {
    pub name: String,
    pub count: u32,
}

fn default_autofill_mode() -> String {
    "default".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub git_remote: String,
    #[serde(default)]
    pub font_family: String,
    #[serde(default = "default_session_timeout")]
    pub session_timeout_minutes: u32,
}

fn default_session_timeout() -> u32 {
    30
}

impl Default for Config {
    fn default() -> Self {
        Self {
            git_remote: String::new(),
            font_family: "system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif".to_string(),
            session_timeout_minutes: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataFile {
    pub version: u32,
    pub salt: String,
    pub entries: Vec<PasswordEntry>,
    pub config: Config,
}
