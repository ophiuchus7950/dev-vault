use chrono::{DateTime, Utc};
use sqlx::{PgConnection, PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::cursor::{decode_cursor, encode_cursor};
use crate::domain::{Entry, EntryKind};
use crate::dto::{SortField, SortOrder};
use crate::error::{AppError, AppResult};

pub struct ListFilter {
    pub kind: Option<EntryKind>,
    pub language: Option<String>,
    pub tags: Vec<String>,
    pub collection: Option<String>,
    pub is_pinned: Option<bool>,
    pub is_archived: Option<bool>,
    pub q: Option<String>,
    pub sort: SortField,
    pub order: SortOrder,
    pub limit: i64,
    pub cursor: Option<String>,
    pub trash: bool,
}

pub struct ListResult {
    pub entries: Vec<Entry>,
    pub has_more: bool,
    pub next_cursor: Option<String>,
}

/// Shared filter set used by both list and search.
pub struct FilterOptions {
    pub kind: Option<EntryKind>,
    pub language: Option<String>,
    pub tags: Vec<String>,
    pub collection: Option<String>,
    pub is_pinned: Option<bool>,
    pub is_archived: Option<bool>,
}

/// Append the shared filter predicates (everything except paging / sorting).
pub fn push_filters(qb: &mut QueryBuilder<'_, Postgres>, f: &FilterOptions) {
    match f.is_archived {
        Some(v) => {
            qb.push(" AND e.is_archived = ").push_bind(v);
        }
        None => {
            qb.push(" AND e.is_archived = false");
        }
    }

    if let Some(kind) = f.kind {
        qb.push(" AND e.kind = ").push_bind(kind);
    }

    if let Some(lang) = &f.language {
        qb.push(" AND e.language = ").push_bind(lang.clone());
    }

    if let Some(pinned) = f.is_pinned {
        qb.push(" AND e.is_pinned = ").push_bind(pinned);
    }

    if let Some(collection) = &f.collection {
        qb.push(
            " AND EXISTS (SELECT 1 FROM entry_collections ec \
             JOIN collections c ON c.id = ec.collection_id \
             WHERE ec.entry_id = e.id AND (c.slug = ",
        )
        .push_bind(collection.clone())
        .push(" OR c.id::text = ")
        .push_bind(collection.clone())
        .push("))");
    }

    if !f.tags.is_empty() {
        let lower: Vec<String> = f.tags.iter().map(|t| t.trim().to_lowercase()).collect();
        qb.push(
            " AND (SELECT count(*) FROM entry_tags et JOIN tags t ON t.id = et.tag_id \
             WHERE et.entry_id = e.id AND lower(t.name) = ANY(",
        )
        .push_bind(lower)
        .push(")) = ")
        .push_bind(f.tags.len() as i64);
    }
}

fn key_of(entry: &Entry, sort: SortField, trash: bool) -> String {
    if trash {
        return format_ts(entry.deleted_at.unwrap_or(entry.updated_at));
    }
    match sort {
        SortField::Title => entry.title.clone(),
        SortField::CreatedAt => format_ts(entry.created_at),
        _ => format_ts(entry.updated_at),
    }
}

fn format_ts(ts: DateTime<Utc>) -> String {
    ts.to_rfc3339_opts(chrono::SecondsFormat::Micros, true)
}

fn parse_ts(raw: &str) -> AppResult<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(raw)
        .map(|d| d.with_timezone(&Utc))
        .map_err(|_| AppError::invalid_cursor())
}

pub async fn list(pool: &PgPool, f: &ListFilter) -> AppResult<ListResult> {
    let mut qb: QueryBuilder<'_, Postgres> = QueryBuilder::new(
        "SELECT e.id, e.kind, e.title, e.body, e.language, e.shell, e.url, \
         e.is_pinned, e.is_archived, e.created_at, e.updated_at, e.deleted_at FROM entries e WHERE ",
    );

    qb.push(if f.trash {
        "e.deleted_at IS NOT NULL"
    } else {
        "e.deleted_at IS NULL"
    });

    if !f.trash {
        push_filters(
            &mut qb,
            &FilterOptions {
                kind: f.kind,
                language: f.language.clone(),
                tags: f.tags.clone(),
                collection: f.collection.clone(),
                is_pinned: f.is_pinned,
                is_archived: f.is_archived,
            },
        );

        if let Some(q) = &f.q {
            qb.push(" AND e.search @@ websearch_to_tsquery('english', immutable_unaccent(")
                .push_bind(q.clone())
                .push("))");
        }
    }

    if let Some(cur) = &f.cursor {
        let payload = decode_cursor(cur)?;
        let op = if f.order == SortOrder::Desc { "<" } else { ">" };

        if f.trash {
            qb.push(format!(" AND (e.deleted_at, e.id) {op} ("))
                .push_bind(parse_ts(&payload.k)?)
                .push(", ")
                .push_bind(payload.id)
                .push(")");
        } else {
            match f.sort {
                SortField::Title => {
                    qb.push(format!(" AND (e.title, e.id) {op} ("))
                        .push_bind(payload.k.clone())
                        .push(", ")
                        .push_bind(payload.id)
                        .push(")");
                }
                SortField::CreatedAt => {
                    qb.push(format!(" AND (e.created_at, e.id) {op} ("))
                        .push_bind(parse_ts(&payload.k)?)
                        .push(", ")
                        .push_bind(payload.id)
                        .push(")");
                }
                _ => {
                    qb.push(format!(" AND (e.updated_at, e.id) {op} ("))
                        .push_bind(parse_ts(&payload.k)?)
                        .push(", ")
                        .push_bind(payload.id)
                        .push(")");
                }
            }
        }
    }

    qb.push(if f.trash {
        " ORDER BY e.deleted_at DESC, e.id DESC"
    } else {
        order_clause(f.sort, f.order)
    });
    qb.push(" LIMIT ").push_bind(f.limit + 1);

    let mut entries: Vec<Entry> = qb.build_query_as::<Entry>().fetch_all(pool).await?;

    let has_more = entries.len() as i64 > f.limit;
    if has_more {
        entries.truncate(f.limit as usize);
    }
    let next_cursor = if has_more {
        entries
            .last()
            .map(|e| encode_cursor(&key_of(e, f.sort, f.trash), e.id))
    } else {
        None
    };

    Ok(ListResult {
        entries,
        has_more,
        next_cursor,
    })
}

fn order_clause(sort: SortField, order: SortOrder) -> &'static str {
    match (sort, order) {
        (SortField::Title, SortOrder::Asc) => " ORDER BY e.title ASC, e.id ASC",
        (SortField::Title, SortOrder::Desc) => " ORDER BY e.title DESC, e.id DESC",
        (SortField::CreatedAt, SortOrder::Asc) => " ORDER BY e.created_at ASC, e.id ASC",
        (SortField::CreatedAt, SortOrder::Desc) => " ORDER BY e.created_at DESC, e.id DESC",
        (SortField::UpdatedAt, SortOrder::Asc) => " ORDER BY e.updated_at ASC, e.id ASC",
        (_, SortOrder::Asc) => " ORDER BY e.updated_at ASC, e.id ASC",
        (_, SortOrder::Desc) => " ORDER BY e.updated_at DESC, e.id DESC",
    }
}

pub async fn find_by_id(
    pool: &PgPool,
    id: Uuid,
    include_deleted: bool,
) -> AppResult<Option<Entry>> {
    let entry = sqlx::query_as!(
        Entry,
        r#"SELECT id, kind AS "kind: EntryKind", title, body, language, shell, url,
                  is_pinned, is_archived, created_at, updated_at, deleted_at
           FROM entries WHERE id = $1 AND ($2::bool OR deleted_at IS NULL)"#,
        id,
        include_deleted
    )
    .fetch_optional(pool)
    .await?;
    Ok(entry)
}

pub async fn find_by_id_locked(conn: &mut PgConnection, id: Uuid) -> AppResult<Option<Entry>> {
    let entry = sqlx::query_as!(
        Entry,
        r#"SELECT id, kind AS "kind: EntryKind", title, body, language, shell, url,
                  is_pinned, is_archived, created_at, updated_at, deleted_at
           FROM entries WHERE id = $1 FOR UPDATE"#,
        id
    )
    .fetch_optional(&mut *conn)
    .await?;
    Ok(entry)
}

pub async fn insert(conn: &mut PgConnection, entry: &Entry) -> AppResult<Entry> {
    let row = sqlx::query_as!(
        Entry,
        r#"INSERT INTO entries (id, kind, title, body, language, shell, url, is_pinned, is_archived)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
           RETURNING id, kind AS "kind: EntryKind", title, body, language, shell, url,
                     is_pinned, is_archived, created_at, updated_at, deleted_at"#,
        entry.id,
        entry.kind as EntryKind,
        entry.title,
        entry.body,
        entry.language,
        entry.shell,
        entry.url,
        entry.is_pinned,
        entry.is_archived
    )
    .fetch_one(&mut *conn)
    .await?;
    Ok(row)
}

pub async fn update(conn: &mut PgConnection, entry: &Entry) -> AppResult<Entry> {
    let row = sqlx::query_as!(
        Entry,
        r#"UPDATE entries SET kind = $2, title = $3, body = $4, language = $5, shell = $6,
                  url = $7, is_pinned = $8, is_archived = $9
           WHERE id = $1
           RETURNING id, kind AS "kind: EntryKind", title, body, language, shell, url,
                     is_pinned, is_archived, created_at, updated_at, deleted_at"#,
        entry.id,
        entry.kind as EntryKind,
        entry.title,
        entry.body,
        entry.language,
        entry.shell,
        entry.url,
        entry.is_pinned,
        entry.is_archived
    )
    .fetch_one(&mut *conn)
    .await?;
    Ok(row)
}

pub async fn soft_delete(pool: &PgPool, id: Uuid) -> AppResult<u64> {
    let result = sqlx::query!(
        "UPDATE entries SET deleted_at = now() WHERE id = $1 AND deleted_at IS NULL",
        id
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

pub async fn restore(pool: &PgPool, id: Uuid) -> AppResult<u64> {
    let result = sqlx::query!(
        "UPDATE entries SET deleted_at = NULL WHERE id = $1 AND deleted_at IS NOT NULL",
        id
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

pub async fn purge(pool: &PgPool, id: Uuid) -> AppResult<u64> {
    let result = sqlx::query!(
        "DELETE FROM entries WHERE id = $1 AND deleted_at IS NOT NULL",
        id
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

/// Insert preserving all timestamps (used by import).
pub async fn insert_full(conn: &mut PgConnection, entry: &Entry) -> AppResult<Entry> {
    let row = sqlx::query_as!(
        Entry,
        r#"INSERT INTO entries (id, kind, title, body, language, shell, url, is_pinned,
                                is_archived, created_at, updated_at, deleted_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
           RETURNING id, kind AS "kind: EntryKind", title, body, language, shell, url,
                     is_pinned, is_archived, created_at, updated_at, deleted_at"#,
        entry.id,
        entry.kind as EntryKind,
        entry.title,
        entry.body,
        entry.language,
        entry.shell,
        entry.url,
        entry.is_pinned,
        entry.is_archived,
        entry.created_at,
        entry.updated_at,
        entry.deleted_at
    )
    .fetch_one(&mut *conn)
    .await?;
    Ok(row)
}

/// Every entry (any state), oldest first. Used by export.
pub async fn all_ordered(pool: &PgPool) -> AppResult<Vec<Entry>> {
    let rows = sqlx::query_as!(
        Entry,
        r#"SELECT id, kind AS "kind: EntryKind", title, body, language, shell, url,
                  is_pinned, is_archived, created_at, updated_at, deleted_at
           FROM entries ORDER BY created_at, id"#
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
