use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::domain::{Entry, EntryKind};
use crate::dto::{
    CreateEntryRequest, EntryPage, EntryResponse, ListQuery, SortField, SortOrder,
    UpdateEntryRequest,
};
use crate::error::{AppError, AppResult, FieldError};
use crate::repo::entries::ListFilter;
use crate::repo::{collections as collections_repo, entries as entries_repo, tags as tags_repo};
use crate::state::AppState;

fn clean(value: Option<String>) -> Option<String> {
    value
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

fn max_len(value: &Option<String>, max: usize) -> bool {
    value
        .as_ref()
        .map(|v| v.chars().count() > max)
        .unwrap_or(false)
}

/// Attach tags and collections to a batch of entries.
pub async fn hydrate(pool: &PgPool, entries: Vec<Entry>) -> AppResult<Vec<EntryResponse>> {
    let ids: Vec<Uuid> = entries.iter().map(|e| e.id).collect();
    let tags = tags_repo::tags_for_entries(pool, &ids).await?;
    let collections = collections_repo::collections_for_entries(pool, &ids).await?;

    Ok(entries
        .into_iter()
        .map(|entry| {
            let entry_tags = tags.get(&entry.id).cloned().unwrap_or_default();
            let entry_collections = collections.get(&entry.id).cloned().unwrap_or_default();
            EntryResponse::from_parts(entry, entry_tags, entry_collections)
        })
        .collect())
}

async fn hydrate_one(pool: &PgPool, entry: Entry) -> AppResult<EntryResponse> {
    Ok(hydrate(pool, vec![entry])
        .await?
        .into_iter()
        .next()
        .expect("hydrate returns one row per input"))
}

fn validate_create(req: &CreateEntryRequest) -> AppResult<()> {
    req.validate()?;

    let mut fields = Vec::new();

    if req.kind == EntryKind::Link && clean(req.url.clone()).is_none() {
        fields.push(FieldError {
            field: "url".to_string(),
            code: "required".to_string(),
            message: "url is required for link entries".to_string(),
        });
    }
    if max_len(&req.language, 50) {
        fields.push(FieldError {
            field: "language".to_string(),
            code: "length".to_string(),
            message: "language must be at most 50 characters".to_string(),
        });
    }
    if max_len(&req.shell, 50) {
        fields.push(FieldError {
            field: "shell".to_string(),
            code: "length".to_string(),
            message: "shell must be at most 50 characters".to_string(),
        });
    }
    if max_len(&req.url, 2048) {
        fields.push(FieldError {
            field: "url".to_string(),
            code: "length".to_string(),
            message: "url must be at most 2048 characters".to_string(),
        });
    }

    if fields.is_empty() {
        Ok(())
    } else {
        Err(AppError::Validation(fields))
    }
}

pub async fn create(state: &AppState, req: CreateEntryRequest) -> AppResult<EntryResponse> {
    validate_create(&req)?;

    let now = Utc::now();
    let entry = Entry {
        id: Uuid::now_v7(),
        kind: req.kind,
        title: req.title.trim().to_string(),
        body: req.body,
        language: clean(req.language),
        shell: clean(req.shell),
        url: clean(req.url),
        is_pinned: req.is_pinned,
        is_archived: req.is_archived,
        created_at: now,
        updated_at: now,
        deleted_at: None,
    };

    let mut tx = state.pool.begin().await?;
    let stored = entries_repo::insert(&mut tx, &entry).await?;
    tags_repo::set_entry_tags(&mut tx, stored.id, &req.tags).await?;
    collections_repo::set_entry_collections(&mut tx, stored.id, &req.collections).await?;
    tx.commit().await?;

    hydrate_one(&state.pool, stored).await
}

pub async fn get(state: &AppState, id: Uuid) -> AppResult<EntryResponse> {
    let entry = entries_repo::find_by_id(&state.pool, id, false)
        .await?
        .ok_or_else(|| AppError::not_found("entry"))?;
    hydrate_one(&state.pool, entry).await
}

pub async fn list(state: &AppState, query: ListQuery) -> AppResult<EntryPage> {
    let filter = ListFilter {
        kind: query.kind,
        language: query.language,
        tags: query.tag,
        collection: query.collection,
        is_pinned: query.is_pinned,
        is_archived: query.is_archived,
        q: query.q,
        sort: query.sort.unwrap_or(SortField::UpdatedAt),
        order: query.order.unwrap_or(SortOrder::Desc),
        limit: state.config.clamp_limit(query.limit),
        cursor: query.cursor,
        trash: false,
    };

    let result = entries_repo::list(&state.pool, &filter).await?;
    let data = hydrate(&state.pool, result.entries).await?;

    Ok(EntryPage {
        data,
        next_cursor: result.next_cursor,
        has_more: result.has_more,
    })
}

pub async fn list_trash(
    state: &AppState,
    limit: Option<i64>,
    cursor: Option<String>,
) -> AppResult<EntryPage> {
    let filter = ListFilter {
        kind: None,
        language: None,
        tags: Vec::new(),
        collection: None,
        is_pinned: None,
        is_archived: None,
        q: None,
        sort: SortField::UpdatedAt,
        order: SortOrder::Desc,
        limit: state.config.clamp_limit(limit),
        cursor,
        trash: true,
    };

    let result = entries_repo::list(&state.pool, &filter).await?;
    let data = hydrate(&state.pool, result.entries).await?;

    Ok(EntryPage {
        data,
        next_cursor: result.next_cursor,
        has_more: result.has_more,
    })
}

pub async fn update(
    state: &AppState,
    id: Uuid,
    if_match: Option<String>,
    req: UpdateEntryRequest,
) -> AppResult<EntryResponse> {
    let expected =
        if_match.ok_or_else(|| AppError::precondition_required("If-Match header is required"))?;

    let mut tx = state.pool.begin().await?;

    let mut entry = match entries_repo::find_by_id_locked(&mut tx, id).await? {
        Some(entry) if entry.deleted_at.is_none() => entry,
        Some(_) => return Err(AppError::unprocessable("entry is in the trash")),
        None => return Err(AppError::not_found("entry")),
    };

    if entry.etag() != expected {
        return Err(AppError::conflict(
            "entry was modified since it was read; refetch and retry",
        ));
    }

    if let Some(title) = &req.title {
        let trimmed = title.trim();
        if trimmed.is_empty() || trimmed.chars().count() > 300 {
            return Err(AppError::Validation(vec![FieldError {
                field: "title".to_string(),
                code: "length".to_string(),
                message: "title must be 1..300 characters".to_string(),
            }]));
        }
        entry.title = trimmed.to_string();
    }
    if let Some(kind) = req.kind {
        entry.kind = kind;
    }
    if let Some(body) = req.body {
        entry.body = body;
    }
    if let Some(language) = req.language {
        entry.language = clean(language);
    }
    if let Some(shell) = req.shell {
        entry.shell = clean(shell);
    }
    if let Some(url) = req.url {
        entry.url = clean(url);
    }
    if let Some(is_pinned) = req.is_pinned {
        entry.is_pinned = is_pinned;
    }
    if let Some(is_archived) = req.is_archived {
        entry.is_archived = is_archived;
    }

    if entry.kind == EntryKind::Link && entry.url.is_none() {
        return Err(AppError::Validation(vec![FieldError {
            field: "url".to_string(),
            code: "required".to_string(),
            message: "url is required for link entries".to_string(),
        }]));
    }

    let updated = entries_repo::update(&mut tx, &entry).await?;

    if let Some(tags) = &req.tags {
        tags_repo::set_entry_tags(&mut tx, id, tags).await?;
    }
    if let Some(collections) = &req.collections {
        collections_repo::set_entry_collections(&mut tx, id, collections).await?;
    }

    tx.commit().await?;

    hydrate_one(&state.pool, updated).await
}

pub async fn soft_delete(state: &AppState, id: Uuid) -> AppResult<()> {
    let affected = entries_repo::soft_delete(&state.pool, id).await?;
    if affected == 0 {
        return Err(AppError::not_found("entry"));
    }
    Ok(())
}

pub async fn restore(state: &AppState, id: Uuid) -> AppResult<EntryResponse> {
    let existing = entries_repo::find_by_id(&state.pool, id, true).await?;
    match existing {
        None => Err(AppError::not_found("entry")),
        Some(entry) if entry.deleted_at.is_none() => {
            Err(AppError::conflict("entry is not in the trash"))
        }
        Some(_) => {
            entries_repo::restore(&state.pool, id).await?;
            get(state, id).await
        }
    }
}

pub async fn purge(state: &AppState, id: Uuid) -> AppResult<()> {
    let existing = entries_repo::find_by_id(&state.pool, id, true).await?;
    match existing {
        None => Err(AppError::not_found("entry")),
        Some(entry) if entry.deleted_at.is_none() => Err(AppError::conflict(
            "entry must be in the trash before it can be purged",
        )),
        Some(_) => {
            entries_repo::purge(&state.pool, id).await?;
            Ok(())
        }
    }
}
