use crate::storage::Storage;
use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;

#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<Mutex<Storage>>,
}

#[derive(Deserialize)]
pub struct DecryptRequest {
    pub site_url: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct DecryptResponse {
    pub entries: Vec<DecryptedEntry>,
}

#[derive(Serialize)]
pub struct DecryptedEntry {
    pub username: String,
    pub password: String,
    pub twofa_secret: Option<String>,
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
    })
}

async fn decrypt(
    State(state): State<AppState>,
    Json(req): Json<DecryptRequest>,
) -> Result<Json<DecryptResponse>, (StatusCode, String)> {
    let storage = state.storage.lock().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Lock error: {}", e),
        )
    })?;

    let domain = extract_domain(&req.site_url);
    let entries = storage.list_entries(None);

    use crate::models::EmailInfo;

    let mut matched = Vec::new();
    for entry in entries {
        if entry.site_url.contains(&domain) || domain.contains(&entry.site_url) {
            if let Ok(decrypted) = storage.get_entry(&entry.id, &req.password) {
                let fill_username = match decrypted.autofill_mode.as_str() {
                    "primary_email" => {
                        if let Some(ref raw) = decrypted.emails_raw {
                            if let Ok(infos) = serde_json::from_str::<Vec<EmailInfo>>(raw) {
                                if let Some(primary) = infos.iter().find(|e| e.is_primary) {
                                    primary.email.clone()
                                } else if let Some(first) = infos.first() {
                                    first.email.clone()
                                } else {
                                    String::new()
                                }
                            } else {
                                decrypted.username.clone()
                            }
                        } else {
                            decrypted.username.clone()
                        }
                    }
                    "phone" => decrypted.phone.unwrap_or_default(),
                    "none" => String::new(),
                    _ => decrypted.username.clone(),
                };
                matched.push(DecryptedEntry {
                    username: fill_username,
                    password: decrypted.password,
                    twofa_secret: decrypted.twofa_secret,
                });
            }
        }
    }

    Ok(Json(DecryptResponse { entries: matched }))
}

fn extract_domain(url: &str) -> String {
    let url = url.trim_start_matches("https://");
    let url = url.trim_start_matches("http://");
    let url = url.split('/').next().unwrap_or(url);
    let url = url.split(':').next().unwrap_or(url);
    url.to_lowercase()
}

pub async fn start_http_server(storage: Arc<Mutex<Storage>>) {
    let state = AppState { storage };
    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/decrypt", post(decrypt))
        .with_state(state);

    let listener = match TcpListener::bind("127.0.0.1:33445").await {
        Ok(l) => l,
        Err(e) => {
            log::error!("Failed to bind HTTP server: {}", e);
            return;
        }
    };

    log::info!("HTTP server listening on 127.0.0.1:33445");
    if let Err(e) = axum::serve(listener, app).await {
        log::error!("HTTP server error: {}", e);
    }
}
