use std::collections::HashMap;

use chrono::{DateTime, Utc};
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use crate::domain::Collection;
use crate::dto::CollectionResponse;
use crate::error::{AppError, AppResult};
use crate::repo::map_unique;

#[derive(sqlx::FromRow)]
struct CollectionWithCount {
    id: Uuid,
    name: String,
    slug: String,
    description: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    entry_count: i64,
}

impl CollectionWithCount {
    fn into_response(self) -> CollectionResponse {
        CollectionResponse {
            id: self.id,
            name: self.name,
            slug: self.slug,
            description: self.description,
            entry_count: self.entry_count,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

pub async fn insert(pool: &PgPool, collection: &Collection) -> AppResult<Collection> {
    sqlx::query_as!(
        Collection,
        "INSERT INTO collections (id, name, slug, description) VALUES ($1, $2, $3, $4)
         RETURNING id, name, slug, description, created_at, updated_at",
        collection.id,
        collection.name,
        collection.slug,
        collection.description
    )
    .fetch_one(pool)
    .await
    .map_err(map_unique)
}

pub async fn list(pool: &PgPool) -> AppResult<Vec<CollectionResponse>> {
    let rows = sqlx::query_as!(
        CollectionWithCount,
        r#"SELECT c.id, c.name, c.slug, c.description, c.created_at, c.updated_at,
                  count(e.id) AS "entry_count!"
           FROM collections c
           LEFT JOIN entry_collections ec ON ec.collection_id = c.id
           LEFT JOIN entries e ON e.id = ec.entry_id AND e.deleted_at IS NULL
           GROUP BY c.id
           ORDER BY lower(c.name)"#
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(CollectionWithCount::into_response)
        .collect())
}

pub async fn get_by_key(pool: &PgPool, key: &str) -> AppResult<Option<CollectionResponse>> {
    let uuid = Uuid::parse_str(key).ok();
    let row = sqlx::query_as!(
        CollectionWithCount,
        r#"SELECT c.id, c.name, c.slug, c.description, c.created_at, c.updated_at,
                  count(e.id) AS "entry_count!"
           FROM collections c
           LEFT JOIN entry_collections ec ON ec.collection_id = c.id
           LEFT JOIN entries e ON e.id = ec.entry_id AND e.deleted_at IS NULL
           WHERE c.slug = $1 OR c.id = $2
           GROUP BY c.id"#,
        key,
        uuid
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(CollectionWithCount::into_response))
}

pub async fn find_id_by_key(conn: &mut PgConnection, key: &str) -> AppResult<Uuid> {
    let uuid = Uuid::parse_str(key).ok();
    let row = sqlx::query!(
        "SELECT id FROM collections WHERE slug = $1 OR id = $2",
        key,
        uuid
    )
    .fetch_optional(&mut *conn)
    .await?;

    row.map(|r| r.id)
        .ok_or_else(|| AppError::unprocessable(format!("collection `{key}` not found")))
}

pub async fn update(
    pool: &PgPool,
    id: Uuid,
    name: Option<&str>,
    slug: Option<&str>,
    description: Option<&str>,
) -> AppResult<Collection> {
    let result = sqlx::query_as!(
        Collection,
        "UPDATE collections
         SET name = COALESCE($2, name),
             slug = COALESCE($3, slug),
             description = COALESCE($4, description)
         WHERE id = $1
         RETURNING id, name, slug, description, created_at, updated_at",
        id,
        name,
        slug,
        description
    )
    .fetch_optional(pool)
    .await;

    match result {
        Ok(Some(collection)) => Ok(collection),
        Ok(None) => Err(AppError::not_found("collection")),
        Err(err) => Err(map_unique(err)),
    }
}

pub async fn delete(pool: &PgPool, id: Uuid) -> AppResult<u64> {
    let result = sqlx::query!("DELETE FROM collections WHERE id = $1", id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

/// Replace the full set of collections on an entry. Keys may be slugs or UUIDs.
pub async fn set_entry_collections(
    conn: &mut PgConnection,
    entry_id: Uuid,
    keys: &[String],
) -> AppResult<()> {
    sqlx::query!(
        "DELETE FROM entry_collections WHERE entry_id = $1",
        entry_id
    )
    .execute(&mut *conn)
    .await?;

    for raw in keys {
        let key = raw.trim();
        if key.is_empty() {
            continue;
        }
        let collection_id = find_id_by_key(&mut *conn, key).await?;
        sqlx::query!(
            "INSERT INTO entry_collections (entry_id, collection_id) VALUES ($1, $2)
             ON CONFLICT DO NOTHING",
            entry_id,
            collection_id
        )
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}

pub async fn collections_for_entries(
    pool: &PgPool,
    ids: &[Uuid],
) -> AppResult<HashMap<Uuid, Vec<String>>> {
    if ids.is_empty() {
        return Ok(HashMap::new());
    }
    let rows = sqlx::query!(
        "SELECT ec.entry_id AS entry_id, c.slug AS slug
         FROM entry_collections ec JOIN collections c ON c.id = ec.collection_id
         WHERE ec.entry_id = ANY($1)
         ORDER BY lower(c.slug)",
        ids
    )
    .fetch_all(pool)
    .await?;

    let mut map: HashMap<Uuid, Vec<String>> = HashMap::new();
    for row in rows {
        map.entry(row.entry_id).or_default().push(row.slug);
    }
    Ok(map)
}

pub async fn assign_entries(
    pool: &PgPool,
    collection_id: Uuid,
    entry_ids: &[Uuid],
) -> AppResult<u64> {
    if entry_ids.is_empty() {
        return Ok(0);
    }
    let result = sqlx::query!(
        "INSERT INTO entry_collections (entry_id, collection_id)
         SELECT e, $2 FROM unnest($1::uuid[]) AS e
         ON CONFLICT DO NOTHING",
        entry_ids,
        collection_id
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

pub async fn unassign_entries(
    pool: &PgPool,
    collection_id: Uuid,
    entry_ids: &[Uuid],
) -> AppResult<u64> {
    if entry_ids.is_empty() {
        return Ok(0);
    }
    let result = sqlx::query!(
        "DELETE FROM entry_collections WHERE collection_id = $2 AND entry_id = ANY($1)",
        entry_ids,
        collection_id
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

/// All collections without counts (used by export).
pub async fn all(pool: &PgPool) -> AppResult<Vec<Collection>> {
    let rows = sqlx::query_as!(
        Collection,
        "SELECT id, name, slug, description, created_at, updated_at
         FROM collections ORDER BY lower(name)"
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Insert a collection during import, ignoring slug conflicts.
pub async fn import_upsert(conn: &mut PgConnection, collection: &Collection) -> AppResult<()> {
    sqlx::query!(
        "INSERT INTO collections (id, name, slug, description)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (slug) DO NOTHING",
        collection.id,
        collection.name,
        collection.slug,
        collection.description
    )
    .execute(&mut *conn)
    .await?;
    Ok(())
}
