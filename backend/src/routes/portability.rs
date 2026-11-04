use axum::extract::{Path, State};
use axum::http::header::CONTENT_TYPE;
use axum::http::HeaderValue;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use axum_extra::extract::Query;
use serde::Deserialize;
use uuid::Uuid;

use crate::dto::{ExportDocument, ImportQuery, ImportSummary};
use crate::error::AppResult;
use crate::service::portability::{self as svc, ExportFormat};
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/export", get(export_vault))
        .route("/import", post(import))
        .route("/entries/{id}/export", get(export_entry))
}

fn text_response(content_type: String, body: Vec<u8>) -> Response {
    let mut response = body.into_response();
    if let Ok(value) = HeaderValue::from_str(&content_type) {
        response.headers_mut().insert(CONTENT_TYPE, value);
    }
    response
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ExportQuery {
    /// `json` (default), `md`, or `zip` (whole-vault only).
    pub format: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/v1/export",
    tag = "portability",
    params(ExportQuery),
    responses((status = 200, description = "Whole-vault export"))
)]
pub async fn export_vault(
    State(state): State<AppState>,
    Query(query): Query<ExportQuery>,
) -> AppResult<Response> {
    let format = ExportFormat::parse(query.format.as_deref().unwrap_or("json"))?;
    let (content_type, body) = svc::export_vault(&state, format).await?;
    Ok(text_response(content_type, body))
}

#[utoipa::path(
    get,
    path = "/api/v1/entries/{id}/export",
    tag = "portability",
    params(
        ("id" = Uuid, Path, description = "Entry id"),
        ExportQuery
    ),
    responses((status = 200, description = "Single-entry export"))
)]
pub async fn export_entry(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(query): Query<ExportQuery>,
) -> AppResult<Response> {
    let format = ExportFormat::parse(query.format.as_deref().unwrap_or("json"))?;
    let (content_type, body) = svc::export_entry(&state, id, format).await?;
    Ok(text_response(content_type, body))
}

#[utoipa::path(
    post,
    path = "/api/v1/import",
    tag = "portability",
    params(ImportQuery),
    request_body = ExportDocument,
    responses((status = 200, body = ImportSummary))
)]
pub async fn import(
    State(state): State<AppState>,
    Query(query): Query<ImportQuery>,
    Json(document): Json<ExportDocument>,
) -> AppResult<Json<ImportSummary>> {
    let policy = query
        .on_conflict
        .unwrap_or(crate::dto::ConflictPolicy::Skip);
    Ok(Json(svc::import(&state, document, policy).await?))
}
