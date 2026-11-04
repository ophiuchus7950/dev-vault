use chrono::Utc;
use uuid::Uuid;
use validator::Validate;

use crate::domain::Collection;
use crate::dto::{
    AssignEntriesRequest, AssignResult, CollectionList, CollectionResponse,
    CreateCollectionRequest, UpdateCollectionRequest,
};
use crate::error::{AppError, AppResult, FieldError};
use crate::repo::collections as repo;
use crate::state::AppState;

fn slugify(name: &str) -> String {
    let mut out = String::new();
    let mut prev_dash = false;
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            prev_dash = false;
        } else if !prev_dash && !out.is_empty() {
            out.push('-');
            prev_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        out.push_str("collection");
    }
    out
}

fn valid_slug(slug: &str) -> bool {
    !slug.is_empty()
        && slug
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !slug.starts_with('-')
        && !slug.ends_with('-')
        && !slug.contains("--")
}

fn to_response(collection: Collection, entry_count: i64) -> CollectionResponse {
    CollectionResponse {
        id: collection.id,
        name: collection.name,
        slug: collection.slug,
        description: collection.description,
        entry_count,
        created_at: collection.created_at,
        updated_at: collection.updated_at,
    }
}

pub async fn create(
    state: &AppState,
    req: CreateCollectionRequest,
) -> AppResult<CollectionResponse> {
    req.validate()?;

    let slug = match req.slug.as_deref().map(str::trim) {
        Some(s) if !s.is_empty() => s.to_string(),
        _ => slugify(&req.name),
    };

    if !valid_slug(&slug) {
        return Err(AppError::Validation(vec![FieldError {
            field: "slug".to_string(),
            code: "format".to_string(),
            message: "slug must be lower-kebab-case (a-z, 0-9, -)".to_string(),
        }]));
    }

    let now = Utc::now();
    let collection = Collection {
        id: Uuid::now_v7(),
        name: req.name.trim().to_string(),
        slug,
        description: req.description.unwrap_or_default(),
        created_at: now,
        updated_at: now,
    };

    let stored = repo::insert(&state.pool, &collection).await?;
    Ok(to_response(stored, 0))
}

pub async fn list(state: &AppState) -> AppResult<CollectionList> {
    Ok(CollectionList {
        data: repo::list(&state.pool).await?,
    })
}

pub async fn get(state: &AppState, key: &str) -> AppResult<CollectionResponse> {
    repo::get_by_key(&state.pool, key)
        .await?
        .ok_or_else(|| AppError::not_found("collection"))
}

pub async fn update(
    state: &AppState,
    key: &str,
    req: UpdateCollectionRequest,
) -> AppResult<CollectionResponse> {
    let existing = repo::get_by_key(&state.pool, key)
        .await?
        .ok_or_else(|| AppError::not_found("collection"))?;

    if let Some(slug) = req.slug.as_deref() {
        if !valid_slug(slug.trim()) {
            return Err(AppError::Validation(vec![FieldError {
                field: "slug".to_string(),
                code: "format".to_string(),
                message: "slug must be lower-kebab-case (a-z, 0-9, -)".to_string(),
            }]));
        }
    }

    repo::update(
        &state.pool,
        existing.id,
        req.name.as_deref().map(str::trim),
        req.slug.as_deref().map(str::trim),
        req.description.as_deref(),
    )
    .await?;

    get(state, &existing.id.to_string()).await
}

pub async fn delete(state: &AppState, key: &str) -> AppResult<()> {
    let existing = repo::get_by_key(&state.pool, key)
        .await?
        .ok_or_else(|| AppError::not_found("collection"))?;
    repo::delete(&state.pool, existing.id).await?;
    Ok(())
}

pub async fn assign(
    state: &AppState,
    key: &str,
    req: AssignEntriesRequest,
) -> AppResult<AssignResult> {
    let existing = repo::get_by_key(&state.pool, key)
        .await?
        .ok_or_else(|| AppError::not_found("collection"))?;
    let count = repo::assign_entries(&state.pool, existing.id, &req.entry_ids).await?;
    Ok(AssignResult { count })
}

pub async fn unassign(
    state: &AppState,
    key: &str,
    req: AssignEntriesRequest,
) -> AppResult<AssignResult> {
    let existing = repo::get_by_key(&state.pool, key)
        .await?
        .ok_or_else(|| AppError::not_found("collection"))?;
    let count = repo::unassign_entries(&state.pool, existing.id, &req.entry_ids).await?;
    Ok(AssignResult { count })
}
