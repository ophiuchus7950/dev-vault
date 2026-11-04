use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use axum::response::Response;
use axum::Router;
use devvault::config::{Config, LogFormat};
use devvault::router::build_router;
use devvault::state::AppState;
use http_body_util::BodyExt;
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;

fn config() -> Config {
    Config {
        database_url: String::new(),
        bind_addr: "127.0.0.1:0".to_string(),
        log_format: LogFormat::Pretty,
        cors_allowed_origins: vec!["http://localhost:5173".to_string()],
        max_page_size: 100,
        default_page_size: 20,
        search_min_trgm_score: 0.3,
        import_max_bytes: 16_777_216,
    }
}

fn app(pool: PgPool) -> Router {
    build_router(AppState::new(pool, config()))
}

async fn body_json(response: Response) -> Value {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

async fn send(
    app: &Router,
    method: &str,
    uri: &str,
    body: Option<Value>,
    etag: Option<&str>,
) -> Response {
    let mut builder = Request::builder().method(method).uri(uri);
    if body.is_some() {
        builder = builder.header(header::CONTENT_TYPE, "application/json");
    }
    if let Some(tag) = etag {
        builder = builder.header(header::IF_MATCH, tag);
    }
    let request = builder
        .body(body.map_or_else(Body::empty, |b| Body::from(b.to_string())))
        .unwrap();
    app.clone().oneshot(request).await.unwrap()
}

async fn create_snippet(app: &Router, title: &str, tags: &[&str]) -> Value {
    let response = send(
        app,
        "POST",
        "/api/v1/entries",
        Some(json!({
            "kind": "snippet",
            "title": title,
            "body": format!("// {title}"),
            "language": "rust",
            "tags": tags,
        })),
        None,
    )
    .await;
    assert_eq!(response.status(), StatusCode::CREATED);
    body_json(response).await
}

#[sqlx::test]
async fn create_read_delete_roundtrip(pool: PgPool) {
    let app = app(pool);

    let entry = create_snippet(&app, "Postgres upsert", &["postgres", "sql"]).await;
    let id = entry["id"].as_str().unwrap().to_string();
    assert_eq!(entry["kind"], "snippet");
    assert_eq!(entry["language"], "rust");

    let fetched = send(&app, "GET", &format!("/api/v1/entries/{id}"), None, None).await;
    assert_eq!(fetched.status(), StatusCode::OK);
    let fetched = body_json(fetched).await;
    assert_eq!(fetched["title"], "Postgres upsert");
    assert_eq!(fetched["tags"], json!(["postgres", "sql"]));

    let deleted = send(&app, "DELETE", &format!("/api/v1/entries/{id}"), None, None).await;
    assert_eq!(deleted.status(), StatusCode::NO_CONTENT);

    let gone = send(&app, "GET", &format!("/api/v1/entries/{id}"), None, None).await;
    assert_eq!(gone.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn rejects_link_without_url(pool: PgPool) {
    let app = app(pool);

    let response = send(
        &app,
        "POST",
        "/api/v1/entries",
        Some(json!({ "kind": "link", "title": "Missing url" })),
        None,
    )
    .await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let body = body_json(response).await;
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
    assert_eq!(body["error"]["details"][0]["field"], "url");
}

#[sqlx::test]
async fn update_requires_if_match_and_detects_stale(pool: PgPool) {
    let app = app(pool);
    let entry = create_snippet(&app, "Original", &[]).await;
    let id = entry["id"].as_str().unwrap().to_string();

    // no If-Match -> 428
    let missing = send(
        &app,
        "PATCH",
        &format!("/api/v1/entries/{id}"),
        Some(json!({ "title": "Renamed" })),
        None,
    )
    .await;
    assert_eq!(missing.status(), StatusCode::PRECONDITION_REQUIRED);

    // stale If-Match -> 409
    let stale = send(
        &app,
        "PATCH",
        &format!("/api/v1/entries/{id}"),
        Some(json!({ "title": "Renamed" })),
        Some("\"deadbeef\""),
    )
    .await;
    assert_eq!(stale.status(), StatusCode::CONFLICT);

    // correct etag -> 200
    let current = send(&app, "GET", &format!("/api/v1/entries/{id}"), None, None).await;
    let etag = current
        .headers()
        .get(header::ETAG)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    assert_eq!(current.status(), StatusCode::OK);

    let updated = send(
        &app,
        "PATCH",
        &format!("/api/v1/entries/{id}"),
        Some(json!({ "title": "Renamed" })),
        Some(&etag),
    )
    .await;
    assert_eq!(updated.status(), StatusCode::OK);
    assert_eq!(body_json(updated).await["title"], "Renamed");
}

#[sqlx::test]
async fn trash_restore_and_purge(pool: PgPool) {
    let app = app(pool);
    let entry = create_snippet(&app, "Trash me", &[]).await;
    let id = entry["id"].as_str().unwrap().to_string();

    send(&app, "DELETE", &format!("/api/v1/entries/{id}"), None, None).await;

    let trash = send(&app, "GET", "/api/v1/trash", None, None).await;
    let trash = body_json(trash).await;
    assert_eq!(trash["data"].as_array().unwrap().len(), 1);

    // purge while trashed -> ok
    let purged = send(
        &app,
        "DELETE",
        &format!("/api/v1/entries/{id}/purge"),
        None,
        None,
    )
    .await;
    assert_eq!(purged.status(), StatusCode::NO_CONTENT);

    // purging again -> 404
    let again = send(
        &app,
        "DELETE",
        &format!("/api/v1/entries/{id}/purge"),
        None,
        None,
    )
    .await;
    assert_eq!(again.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn restore_returns_conflict_when_not_trashed(pool: PgPool) {
    let app = app(pool);
    let entry = create_snippet(&app, "Alive", &[]).await;
    let id = entry["id"].as_str().unwrap().to_string();

    let response = send(
        &app,
        "POST",
        &format!("/api/v1/entries/{id}/restore"),
        None,
        None,
    )
    .await;
    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[sqlx::test]
async fn tag_filter_requires_all_tags(pool: PgPool) {
    let app = app(pool);
    create_snippet(&app, "Both tags", &["postgres", "sql"]).await;
    create_snippet(&app, "One tag", &["postgres"]).await;

    let response = send(
        &app,
        "GET",
        "/api/v1/entries?tag=postgres&tag=sql",
        None,
        None,
    )
    .await;
    let body = body_json(response).await;
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["title"], "Both tags");
}

#[sqlx::test]
async fn search_matches_across_case_and_typos(pool: PgPool) {
    let app = app(pool);
    create_snippet(&app, "Debounce with useEffect", &["react"]).await;
    create_snippet(&app, "Postgres upsert", &["postgres"]).await;

    let response = send(&app, "GET", "/api/v1/search?q=useEfect", None, None).await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_json(response).await;
    let data = body["data"].as_array().unwrap();
    assert!(!data.is_empty(), "expected a fuzzy match for useEfect");
    assert!(data[0]["title"]
        .as_str()
        .unwrap()
        .to_lowercase()
        .contains("useeffect"));
}

#[sqlx::test]
async fn search_rejects_short_query(pool: PgPool) {
    let app = app(pool);
    let response = send(&app, "GET", "/api/v1/search?q=a", None, None).await;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test]
async fn pagination_returns_each_entry_once(pool: PgPool) {
    let app = app(pool);
    for i in 0..5 {
        create_snippet(&app, &format!("Entry {i}"), &[]).await;
    }

    let mut seen: Vec<String> = Vec::new();
    let mut cursor: Option<String> = None;
    loop {
        let uri = match &cursor {
            Some(c) => format!("/api/v1/entries?limit=2&cursor={c}"),
            None => "/api/v1/entries?limit=2".to_string(),
        };
        let response = send(&app, "GET", &uri, None, None).await;
        let body = body_json(response).await;
        for item in body["data"].as_array().unwrap() {
            seen.push(item["id"].as_str().unwrap().to_string());
        }
        match body["next_cursor"].as_str() {
            Some(next) => cursor = Some(next.to_string()),
            None => break,
        }
    }

    assert_eq!(seen.len(), 5);
    let mut unique = seen.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(unique.len(), 5, "pagination returned duplicates");
}

#[sqlx::test]
async fn export_then_import_is_idempotent(pool: PgPool) {
    let app = app(pool);
    create_snippet(&app, "Alpha", &["one"]).await;
    create_snippet(&app, "Beta", &["two"]).await;

    let exported = send(&app, "GET", "/api/v1/export?format=json", None, None).await;
    assert_eq!(exported.status(), StatusCode::OK);
    let document = body_json(exported).await;
    assert_eq!(document["entries"].as_array().unwrap().len(), 2);

    let imported = send(
        &app,
        "POST",
        "/api/v1/import?on_conflict=skip",
        Some(document),
        None,
    )
    .await;
    assert_eq!(imported.status(), StatusCode::OK);
    let summary = body_json(imported).await;
    assert_eq!(summary["skipped"], 2);
    assert_eq!(summary["imported"], 0);

    let list = send(&app, "GET", "/api/v1/entries", None, None).await;
    let list = body_json(list).await;
    assert_eq!(list["data"].as_array().unwrap().len(), 2);
}

#[sqlx::test]
async fn collection_slug_filters_entries(pool: PgPool) {
    let app = app(pool);

    let created = send(
        &app,
        "POST",
        "/api/v1/collections",
        Some(json!({ "name": "Backend", "slug": "backend" })),
        None,
    )
    .await;
    assert_eq!(created.status(), StatusCode::CREATED);

    create_snippet(&app, "In collection", &[]).await;
    let entry = create_snippet(&app, "Also in collection", &[]).await;
    let id = entry["id"].as_str().unwrap().to_string();

    let assigned = send(
        &app,
        "POST",
        "/api/v1/collections/backend/entries",
        Some(json!({ "entry_ids": [id] })),
        None,
    )
    .await;
    assert_eq!(assigned.status(), StatusCode::OK);

    let filtered = send(
        &app,
        "GET",
        "/api/v1/entries?collection=backend",
        None,
        None,
    )
    .await;
    let filtered = body_json(filtered).await;
    assert_eq!(filtered["data"].as_array().unwrap().len(), 1);
}

#[sqlx::test]
async fn vault_zip_export_returns_archive(pool: PgPool) {
    let app = app(pool);
    create_snippet(&app, "Zip me", &["archive"]).await;

    let response = send(&app, "GET", "/api/v1/export?format=zip", None, None).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get(header::CONTENT_TYPE)
            .unwrap()
            .to_str()
            .unwrap(),
        "application/zip"
    );

    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    assert!(bytes.starts_with(b"PK"), "expected a zip archive signature");
}
