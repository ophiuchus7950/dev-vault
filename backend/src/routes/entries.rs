use axum::extract::{Path, State};
use axum::http::header::{ETAG, IF_MATCH, LOCATION};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use axum_extra::extract::Query;
use serde::Deserialize;
use uuid::Uuid;

use crate::dto::{CreateEntryRequest, EntryPage, EntryResponse, ListQuery, UpdateEntryRequest};
use crate::error::AppResult;
use crate::service::entries as svc;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/entries", get(list).post(create))
        .route("/entries/{id}", get(get_one).patch(update).delete(delete))
        .route("/entries/{id}/restore", post(restore))
        .route("/entries/{id}/purge", axum::routing::delete(purge))
        .route("/trash", get(list_trash))
}

fn with_etag(entry: EntryResponse, status: StatusCode, location: Option<String>) -> Response {
    let etag = entry.etag();
    let mut response = (status, Json(entry)).into_response();
    if let Ok(value) = HeaderValue::from_str(&etag) {
        response.headers_mut().insert(ETAG, value);
    }
    if let Some(location) = location {
        if let Ok(value) = HeaderValue::from_str(&location) {
            response.headers_mut().insert(LOCATION, value);
        }
    }
    response
}

#[utoipa::path(
    get,
    path = "/api/v1/entries",
    tag = "entries",
    params(ListQuery),
    responses((status = 200, body = EntryPage))
)]
pub async fn list(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<EntryPage>> {
    Ok(Json(svc::list(&state, query).await?))
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub struct TrashQuery {
    pub limit: Option<i64>,
    pub cursor: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/v1/trash",
    tag = "entries",
    params(TrashQuery),
    responses((status = 200, body = EntryPage))
)]
pub async fn list_trash(
    State(state): State<AppState>,
    Query(query): Query<TrashQuery>,
) -> AppResult<Json<EntryPage>> {
    Ok(Json(
        svc::list_trash(&state, query.limit, query.cursor).await?,
    ))
}

#[utoipa::path(
    post,
    path = "/api/v1/entries",
    tag = "entries",
    request_body = CreateEntryRequest,
    responses(
        (status = 201, body = EntryResponse),
        (status = 422, description = "Validation failed")
    )
)]
pub async fn create(
    State(state): State<AppState>,
    Json(req): Json<CreateEntryRequest>,
) -> AppResult<Response> {
    let entry = svc::create(&state, req).await?;
    let location = format!("/api/v1/entries/{}", entry.id);
    Ok(with_etag(entry, StatusCode::CREATED, Some(location)))
}

#[utoipa::path(
    get,
    path = "/api/v1/entries/{id}",
    tag = "entries",
    params(("id" = Uuid, Path, description = "Entry id")),
    responses((status = 200, body = EntryResponse), (status = 404, description = "Not found"))
)]
pub async fn get_one(State(state): State<AppState>, Path(id): Path<Uuid>) -> AppResult<Response> {
    let entry = svc::get(&state, id).await?;
    Ok(with_etag(entry, StatusCode::OK, None))
}

#[utoipa::path(
    patch,
    path = "/api/v1/entries/{id}",
    tag = "entries",
    params(("id" = Uuid, Path, description = "Entry id")),
    request_body = UpdateEntryRequest,
    responses(
        (status = 200, body = EntryResponse),
        (status = 409, description = "Stale ETag"),
        (status = 428, description = "If-Match required")
    )
)]
pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(req): Json<UpdateEntryRequest>,
) -> AppResult<Response> {
    let if_match = headers
        .get(IF_MATCH)
        .and_then(|v| v.to_str().ok())
        .map(ToString::to_string);
    let entry = svc::update(&state, id, if_match, req).await?;
    Ok(with_etag(entry, StatusCode::OK, None))
}

#[utoipa::path(
    delete,
    path = "/api/v1/entries/{id}",
    tag = "entries",
    params(("id" = Uuid, Path, description = "Entry id")),
    responses((status = 204), (status = 404, description = "Not found"))
)]
pub async fn delete(State(state): State<AppState>, Path(id): Path<Uuid>) -> AppResult<StatusCode> {
    svc::soft_delete(&state, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/v1/entries/{id}/restore",
    tag = "entries",
    params(("id" = Uuid, Path, description = "Entry id")),
    responses((status = 200, body = EntryResponse), (status = 409, description = "Not trashed"))
)]
pub async fn restore(State(state): State<AppState>, Path(id): Path<Uuid>) -> AppResult<Response> {
    let entry = svc::restore(&state, id).await?;
    Ok(with_etag(entry, StatusCode::OK, None))
}

#[utoipa::path(
    delete,
    path = "/api/v1/entries/{id}/purge",
    tag = "entries",
    params(("id" = Uuid, Path, description = "Entry id")),
    responses((status = 204), (status = 409, description = "Not trashed"))
)]
pub async fn purge(State(state): State<AppState>, Path(id): Path<Uuid>) -> AppResult<StatusCode> {
    svc::purge(&state, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
