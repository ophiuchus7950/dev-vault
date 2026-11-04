use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::domain::{Entry, EntryKind};

// ---------------------------------------------------------------------------
// Entry
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct EntryResponse {
    pub id: Uuid,
    pub kind: EntryKind,
    pub title: String,
    pub body: String,
    pub language: Option<String>,
    pub shell: Option<String>,
    pub url: Option<String>,
    pub tags: Vec<String>,
    pub collections: Vec<String>,
    pub is_pinned: bool,
    pub is_archived: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl EntryResponse {
    pub fn from_parts(entry: Entry, mut tags: Vec<String>, mut collections: Vec<String>) -> Self {
        tags.sort();
        collections.sort();
        Self {
            id: entry.id,
            kind: entry.kind,
            title: entry.title,
            body: entry.body,
            language: entry.language,
            shell: entry.shell,
            url: entry.url,
            tags,
            collections,
            is_pinned: entry.is_pinned,
            is_archived: entry.is_archived,
            created_at: entry.created_at,
            updated_at: entry.updated_at,
            deleted_at: entry.deleted_at,
        }
    }

    pub fn etag(&self) -> String {
        format!("\"{}-{}\"", self.id, self.updated_at.timestamp_micros())
    }
}

#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct CreateEntryRequest {
    pub kind: EntryKind,
    #[validate(length(min = 1, max = 300))]
    pub title: String,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub shell: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub collections: Vec<String>,
    #[serde(default)]
    pub is_pinned: bool,
    #[serde(default)]
    pub is_archived: bool,
}

/// Distinguishes "field absent" (`None`) from "field explicitly null" (`Some(None)`).
fn double_option<'de, T, D>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    T: Deserialize<'de>,
    D: serde::Deserializer<'de>,
{
    Deserialize::deserialize(deserializer).map(Some)
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateEntryRequest {
    #[serde(default)]
    pub kind: Option<EntryKind>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default, deserialize_with = "double_option")]
    pub language: Option<Option<String>>,
    #[serde(default, deserialize_with = "double_option")]
    pub shell: Option<Option<String>>,
    #[serde(default, deserialize_with = "double_option")]
    pub url: Option<Option<String>>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub collections: Option<Vec<String>>,
    #[serde(default)]
    pub is_pinned: Option<bool>,
    #[serde(default)]
    pub is_archived: Option<bool>,
}

// ---------------------------------------------------------------------------
// Query params
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SortField {
    UpdatedAt,
    CreatedAt,
    Title,
    Relevance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    Asc,
    Desc,
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListQuery {
    pub kind: Option<EntryKind>,
    pub language: Option<String>,
    #[serde(default)]
    pub tag: Vec<String>,
    pub collection: Option<String>,
    pub is_pinned: Option<bool>,
    pub is_archived: Option<bool>,
    pub q: Option<String>,
    pub sort: Option<SortField>,
    pub order: Option<SortOrder>,
    pub limit: Option<i64>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum SearchMode {
    Fts,
    Trgm,
    Hybrid,
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub struct SearchQuery {
    pub q: String,
    pub mode: Option<SearchMode>,
    pub kind: Option<EntryKind>,
    pub language: Option<String>,
    #[serde(default)]
    pub tag: Vec<String>,
    pub collection: Option<String>,
    pub is_pinned: Option<bool>,
    pub is_archived: Option<bool>,
    pub sort: Option<SortField>,
    pub order: Option<SortOrder>,
    pub limit: Option<i64>,
    pub cursor: Option<String>,
}

// ---------------------------------------------------------------------------
// Response envelopes
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct EntryPage {
    pub data: Vec<EntryResponse>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SearchHit {
    #[serde(flatten)]
    pub entry: EntryResponse,
    pub score: f64,
    pub matched: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SearchPage {
    pub data: Vec<SearchHit>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

// ---------------------------------------------------------------------------
// Tags
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TagWithCount {
    pub id: Uuid,
    pub name: String,
    pub count: i64,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TagList {
    pub data: Vec<TagWithCount>,
}

#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct RenameTagRequest {
    #[validate(length(min = 1, max = 50))]
    pub name: String,
}

// ---------------------------------------------------------------------------
// Collections
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CollectionResponse {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub entry_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CollectionList {
    pub data: Vec<CollectionResponse>,
}

#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct CreateCollectionRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateCollectionRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct AssignEntriesRequest {
    pub entry_ids: Vec<Uuid>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct AssignResult {
    pub count: u64,
}

// ---------------------------------------------------------------------------
// Import / export
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ExportDocument {
    pub version: u32,
    pub exported_at: DateTime<Utc>,
    pub entries: Vec<EntryResponse>,
    pub collections: Vec<CollectionResponse>,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ImportSummary {
    pub imported: u64,
    pub skipped: u64,
    pub overwritten: u64,
    pub errors: Vec<String>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ImportQuery {
    #[serde(default)]
    pub on_conflict: Option<ConflictPolicy>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum ConflictPolicy {
    Skip,
    Overwrite,
}

// ---------------------------------------------------------------------------
// Health
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_s: u64,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ReadyResponse {
    pub status: String,
    pub db: String,
    pub migration: String,
}
