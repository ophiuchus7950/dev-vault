use std::collections::{HashMap, HashSet};

use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use crate::domain::Tag;
use crate::dto::TagWithCount;
use crate::error::{AppError, AppResult};
use crate::repo::map_unique;

/// Find-or-create a tag (case-insensitive) and return its id.
pub async fn upsert_tag(conn: &mut PgConnection, name: &str) -> AppResult<Uuid> {
    let trimmed = name.trim();
    let lower = trimmed.to_lowercase();

    if let Some(id) = sqlx::query_scalar!("SELECT id FROM tags WHERE lower(name) = $1", lower)
        .fetch_optional(&mut *conn)
        .await?
    {
        return Ok(id);
    }

    let id = Uuid::now_v7();
    sqlx::query!(
        "INSERT INTO tags (id, name) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        id,
        trimmed
    )
    .execute(&mut *conn)
    .await?;

    let id = sqlx::query_scalar!("SELECT id FROM tags WHERE lower(name) = $1", lower)
        .fetch_one(&mut *conn)
        .await?;
    Ok(id)
}

/// Replace the full set of tags on an entry.
pub async fn set_entry_tags(
    conn: &mut PgConnection,
    entry_id: Uuid,
    names: &[String],
) -> AppResult<()> {
    sqlx::query!("DELETE FROM entry_tags WHERE entry_id = $1", entry_id)
        .execute(&mut *conn)
        .await?;

    let mut seen: HashSet<String> = HashSet::new();
    for raw in names {
        let name = raw.trim();
        if name.is_empty() {
            continue;
        }
        if !seen.insert(name.to_lowercase()) {
            continue;
        }
        let tag_id = upsert_tag(&mut *conn, name).await?;
        sqlx::query!(
            "INSERT INTO entry_tags (entry_id, tag_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            entry_id,
            tag_id
        )
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}

pub async fn tags_for_entries(
    pool: &PgPool,
    ids: &[Uuid],
) -> AppResult<HashMap<Uuid, Vec<String>>> {
    if ids.is_empty() {
        return Ok(HashMap::new());
    }
    let rows = sqlx::query!(
        "SELECT et.entry_id AS entry_id, t.name AS name
         FROM entry_tags et JOIN tags t ON t.id = et.tag_id
         WHERE et.entry_id = ANY($1)
         ORDER BY lower(t.name)",
        ids
    )
    .fetch_all(pool)
    .await?;

    let mut map: HashMap<Uuid, Vec<String>> = HashMap::new();
    for row in rows {
        map.entry(row.entry_id).or_default().push(row.name);
    }
    Ok(map)
}

pub async fn list_tags(pool: &PgPool, q: Option<&str>) -> AppResult<Vec<TagWithCount>> {
    let rows = sqlx::query!(
        r#"SELECT t.id AS id, t.name AS name, count(e.id) AS "count!"
           FROM tags t
           LEFT JOIN entry_tags et ON et.tag_id = t.id
           LEFT JOIN entries e ON e.id = et.entry_id AND e.deleted_at IS NULL
           WHERE ($1::text IS NULL OR lower(t.name) LIKE '%' || lower($1) || '%')
           GROUP BY t.id, t.name
           ORDER BY count(e.id) DESC, lower(t.name) ASC"#,
        q
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| TagWithCount {
            id: r.id,
            name: r.name,
            count: r.count,
        })
        .collect())
}

pub async fn rename_tag(pool: &PgPool, id: Uuid, name: &str) -> AppResult<Tag> {
    let result = sqlx::query_as!(
        Tag,
        "UPDATE tags SET name = $2 WHERE id = $1 RETURNING id, name, created_at",
        id,
        name.trim()
    )
    .fetch_optional(pool)
    .await;

    match result {
        Ok(Some(tag)) => Ok(tag),
        Ok(None) => Err(AppError::not_found("tag")),
        Err(err) => Err(map_unique(err)),
    }
}

pub async fn delete_tag(pool: &PgPool, id: Uuid) -> AppResult<u64> {
    let result = sqlx::query!("DELETE FROM tags WHERE id = $1", id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

/// All tag names, sorted (used by export).
pub async fn all_names(pool: &PgPool) -> AppResult<Vec<String>> {
    let rows = sqlx::query!("SELECT name FROM tags ORDER BY lower(name)")
        .fetch_all(pool)
        .await?;
    Ok(rows.into_iter().map(|r| r.name).collect())
}
