use std::sync::OnceLock;
use std::time::Instant;

use axum::extract::State;
use axum::Json;

use crate::dto::{HealthResponse, ReadyResponse};
use crate::error::{AppError, AppResult};
use crate::state::AppState;

fn started() -> &'static Instant {
    static START: OnceLock<Instant> = OnceLock::new();
    START.get_or_init(Instant::now)
}

#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses((status = 200, description = "Liveness", body = HealthResponse))
)]
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_s: started().elapsed().as_secs(),
    })
}

#[utoipa::path(
    get,
    path = "/ready",
    tag = "health",
    responses(
        (status = 200, description = "Ready", body = ReadyResponse),
        (status = 503, description = "Database unavailable")
    )
)]
pub async fn ready(State(state): State<AppState>) -> AppResult<Json<ReadyResponse>> {
    let row = sqlx::query!(
        "SELECT version FROM _sqlx_migrations WHERE success = true ORDER BY version DESC LIMIT 1"
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::Internal(anyhow::Error::new(e)))?;

    Ok(Json(ReadyResponse {
        status: "ready".to_string(),
        db: "ok".to_string(),
        migration: row
            .map(|r| r.version.to_string())
            .unwrap_or_else(|| "none".to_string()),
    }))
}
