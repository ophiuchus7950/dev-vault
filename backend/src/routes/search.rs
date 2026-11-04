use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use axum_extra::extract::Query;

use crate::dto::{SearchPage, SearchQuery};
use crate::error::AppResult;
use crate::service::search as svc;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/search", get(search))
}

#[utoipa::path(
    get,
    path = "/api/v1/search",
    tag = "search",
    params(SearchQuery),
    responses((status = 200, body = SearchPage), (status = 422, description = "Invalid query"))
)]
pub async fn search(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> AppResult<Json<SearchPage>> {
    Ok(Json(svc::search(&state, query).await?))
}
