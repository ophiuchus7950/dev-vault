use axum::http::header;
use axum::http::{HeaderValue, Method};
use axum::routing::get;
use axum::Router;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::openapi::ApiDoc;
use crate::state::AppState;

fn build_cors(origins: &[String]) -> CorsLayer {
    let values: Vec<HeaderValue> = origins
        .iter()
        .filter_map(|origin| HeaderValue::from_str(origin).ok())
        .collect();

    let layer = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([header::CONTENT_TYPE, header::ACCEPT, header::IF_MATCH])
        .expose_headers([header::ETAG, header::LOCATION]);

    if values.is_empty() {
        layer
    } else {
        layer.allow_origin(values)
    }
}

pub fn build_router(state: AppState) -> Router {
    let cors = build_cors(&state.config.cors_allowed_origins);
    let body_limit = state.config.import_max_bytes;

    let api = Router::new()
        .merge(crate::routes::entries::routes())
        .merge(crate::routes::tags::routes())
        .merge(crate::routes::collections::routes())
        .merge(crate::routes::search::routes())
        .merge(crate::routes::portability::routes());

    Router::new()
        .route("/health", get(crate::routes::health::health))
        .route("/ready", get(crate::routes::health::ready))
        .nest("/api/v1", api)
        .with_state(state)
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(RequestBodyLimitLayer::new(body_limit))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
}
