use crate::storage::Storage;
use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::net::TcpListener;

#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<Mutex<Storage>>,
    pub fail_count: Arc<AtomicU32>,
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
    let fail_count = state.fail_count.load(Ordering::Relaxed);
    if fail_count >= 10 {
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            "Too many failed attempts. Try again later.".to_string(),
        ));
    }
    if fail_count > 0 {
        tokio::time::sleep(Duration::from_secs(fail_count as u64)).await;
    }

    let storage = state.storage.lock().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Lock error: {}", e),
        )
    })?;

    let password_valid = storage.verify_password(&req.password).is_ok();

    if !password_valid {
        state.fail_count.fetch_add(1, Ordering::Relaxed);
        return Err((StatusCode::UNAUTHORIZED, "Wrong password".to_string()));
    }

    let domain = extract_domain(&req.site_url);
    let entries = storage.list_entries(None, None, None);

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

    state.fail_count.store(0, Ordering::Relaxed);
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
    let state = AppState {
        storage,
        fail_count: Arc::new(AtomicU32::new(0)),
    };
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
