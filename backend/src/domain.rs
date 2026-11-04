use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema,
)]
#[sqlx(type_name = "entry_kind", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum EntryKind {
    Snippet,
    Command,
    Note,
    Link,
    Doc,
}

impl EntryKind {
    pub fn as_str(self) -> &'static str {
        match self {
            EntryKind::Snippet => "snippet",
            EntryKind::Command => "command",
            EntryKind::Note => "note",
            EntryKind::Link => "link",
            EntryKind::Doc => "doc",
        }
    }
}

/// A row of the `entries` table (without joins).
#[derive(Debug, Clone, FromRow, Serialize, utoipa::ToSchema)]
pub struct Entry {
    pub id: Uuid,
    pub kind: EntryKind,
    pub title: String,
    pub body: String,
    pub language: Option<String>,
    pub shell: Option<String>,
    pub url: Option<String>,
    pub is_pinned: bool,
    pub is_archived: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Entry {
    /// Strong ETag derived from the row identity and its last update time.
    pub fn etag(&self) -> String {
        format!("\"{}-{}\"", self.id, self.updated_at.timestamp_micros())
    }
}

#[derive(Debug, Clone, FromRow, Serialize, utoipa::ToSchema)]
pub struct Tag {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, utoipa::ToSchema)]
pub struct Collection {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
