use axum::extract::{Path, State};
use axum::http::header::LOCATION;
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};

use crate::dto::{
    AssignEntriesRequest, AssignResult, CollectionList, CollectionResponse,
    CreateCollectionRequest, UpdateCollectionRequest,
};
use crate::error::AppResult;
use crate::service::collections as svc;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/collections", get(list).post(create))
        .route(
            "/collections/{id}",
            get(get_one).patch(update).delete(delete),
        )
        .route("/collections/{id}/entries", post(assign).delete(unassign))
}

#[utoipa::path(
    get,
    path = "/api/v1/collections",
    tag = "collections",
    responses((status = 200, body = CollectionList))
)]
pub async fn list(State(state): State<AppState>) -> AppResult<Json<CollectionList>> {
    Ok(Json(svc::list(&state).await?))
}

#[utoipa::path(
    post,
    path = "/api/v1/collections",
    tag = "collections",
    request_body = CreateCollectionRequest,
    responses((status = 201, body = CollectionResponse), (status = 409, description = "Slug conflict"))
)]
pub async fn create(
    State(state): State<AppState>,
    Json(req): Json<CreateCollectionRequest>,
) -> AppResult<Response> {
    let collection = svc::create(&state, req).await?;
    let location = format!("/api/v1/collections/{}", collection.slug);
    let mut response = (StatusCode::CREATED, Json(collection)).into_response();
    if let Ok(value) = HeaderValue::from_str(&location) {
        response.headers_mut().insert(LOCATION, value);
    }
    Ok(response)
}

#[utoipa::path(
    get,
    path = "/api/v1/collections/{id}",
    tag = "collections",
    params(("id" = String, Path, description = "Collection id or slug")),
    responses((status = 200, body = CollectionResponse), (status = 404, description = "Not found"))
)]
pub async fn get_one(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<CollectionResponse>> {
    Ok(Json(svc::get(&state, &id).await?))
}

#[utoipa::path(
    patch,
    path = "/api/v1/collections/{id}",
    tag = "collections",
    params(("id" = String, Path, description = "Collection id or slug")),
    request_body = UpdateCollectionRequest,
    responses((status = 200, body = CollectionResponse), (status = 404, description = "Not found"))
)]
pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateCollectionRequest>,
) -> AppResult<Json<CollectionResponse>> {
    Ok(Json(svc::update(&state, &id, req).await?))
}

#[utoipa::path(
    delete,
    path = "/api/v1/collections/{id}",
    tag = "collections",
    params(("id" = String, Path, description = "Collection id or slug")),
    responses((status = 204), (status = 404, description = "Not found"))
)]
pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    svc::delete(&state, &id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/v1/collections/{id}/entries",
    tag = "collections",
    params(("id" = String, Path, description = "Collection id or slug")),
    request_body = AssignEntriesRequest,
    responses((status = 200, body = AssignResult))
)]
pub async fn assign(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<AssignEntriesRequest>,
) -> AppResult<Json<AssignResult>> {
    Ok(Json(svc::assign(&state, &id, req).await?))
}

#[utoipa::path(
    delete,
    path = "/api/v1/collections/{id}/entries",
    tag = "collections",
    params(("id" = String, Path, description = "Collection id or slug")),
    request_body = AssignEntriesRequest,
    responses((status = 200, body = AssignResult))
)]
pub async fn unassign(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<AssignEntriesRequest>,
) -> AppResult<Json<AssignResult>> {
    Ok(Json(svc::unassign(&state, &id, req).await?))
}
