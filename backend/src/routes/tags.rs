use axum::extract::{Path, State};
use axum::routing::{get, patch};
use axum::{Json, Router};
use axum_extra::extract::Query;
use serde::Deserialize;
use uuid::Uuid;

use crate::domain::Tag;
use crate::dto::{RenameTagRequest, TagList};
use crate::error::AppResult;
use crate::service::tags as svc;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/tags", get(list))
        .route("/tags/{id}", patch(rename).delete(delete))
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub struct TagQuery {
    pub q: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/v1/tags",
    tag = "tags",
    params(TagQuery),
    responses((status = 200, body = TagList))
)]
pub async fn list(
    State(state): State<AppState>,
    Query(query): Query<TagQuery>,
) -> AppResult<Json<TagList>> {
    Ok(Json(svc::list(&state, query.q).await?))
}

#[utoipa::path(
    patch,
    path = "/api/v1/tags/{id}",
    tag = "tags",
    params(("id" = Uuid, Path, description = "Tag id")),
    request_body = RenameTagRequest,
    responses((status = 200, body = Tag), (status = 409, description = "Name conflict"))
)]
pub async fn rename(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<RenameTagRequest>,
) -> AppResult<Json<Tag>> {
    Ok(Json(svc::rename(&state, id, req).await?))
}

#[utoipa::path(
    delete,
    path = "/api/v1/tags/{id}",
    tag = "tags",
    params(("id" = Uuid, Path, description = "Tag id")),
    responses((status = 204), (status = 404, description = "Not found"))
)]
pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<axum::http::StatusCode> {
    svc::delete(&state, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
