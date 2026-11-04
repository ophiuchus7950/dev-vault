use std::io::{Cursor, Write};

use chrono::Utc;
use zip::write::SimpleFileOptions;
use zip::CompressionMethod;

use crate::domain::{Collection, Entry, EntryKind};
use crate::dto::{
    CollectionResponse, ConflictPolicy, EntryResponse, ExportDocument, ImportSummary,
};
use crate::error::{AppError, AppResult};
use crate::repo::{collections as collections_repo, entries as entries_repo, tags as tags_repo};
use crate::service::entries::hydrate;
use crate::state::AppState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Json,
    Markdown,
    Zip,
}

impl ExportFormat {
    pub fn parse(raw: &str) -> AppResult<Self> {
        match raw {
            "json" => Ok(ExportFormat::Json),
            "md" | "markdown" => Ok(ExportFormat::Markdown),
            "zip" => Ok(ExportFormat::Zip),
            other => Err(AppError::unsupported_format(other)),
        }
    }
}

fn json<T: serde::Serialize>(value: &T) -> AppResult<String> {
    serde_json::to_string_pretty(value).map_err(|e| AppError::Internal(anyhow::Error::new(e)))
}

fn zip_err(error: zip::result::ZipError) -> AppError {
    AppError::Internal(anyhow::Error::new(error))
}

fn io_err(error: std::io::Error) -> AppError {
    AppError::Internal(anyhow::Error::new(error))
}

pub fn entry_to_markdown(e: &EntryResponse) -> String {
    let mut out = String::new();
    out.push_str("---\n");
    out.push_str(&format!("id: {}\n", e.id));
    out.push_str(&format!("kind: {}\n", e.kind.as_str()));
    out.push_str(&format!("title: {}\n", e.title.replace(['\n', '\r'], " ")));
    if let Some(language) = &e.language {
        out.push_str(&format!("language: {language}\n"));
    }
    if let Some(shell) = &e.shell {
        out.push_str(&format!("shell: {shell}\n"));
    }
    if let Some(url) = &e.url {
        out.push_str(&format!("url: {url}\n"));
    }
    out.push_str(&format!("tags: [{}]\n", e.tags.join(", ")));
    out.push_str(&format!("collections: [{}]\n", e.collections.join(", ")));
    out.push_str(&format!("is_pinned: {}\n", e.is_pinned));
    out.push_str(&format!("is_archived: {}\n", e.is_archived));
    out.push_str(&format!("created_at: {}\n", e.created_at.to_rfc3339()));
    out.push_str(&format!("updated_at: {}\n", e.updated_at.to_rfc3339()));
    out.push_str("---\n\n");

    match e.kind {
        EntryKind::Snippet | EntryKind::Command => {
            let language = e
                .language
                .clone()
                .or_else(|| e.shell.clone())
                .unwrap_or_else(|| "text".to_string());
            out.push_str(&format!("```{language}\n{}\n```\n", e.body));
        }
        EntryKind::Link => {
            out.push_str(&format!(
                "[{}]({})\n\n{}\n",
                e.title,
                e.url.clone().unwrap_or_default(),
                e.body
            ));
        }
        _ => {
            out.push_str(&e.body);
            out.push('\n');
        }
    }

    out
}

fn build_vault_zip(manifest_json: &str, entries: &[EntryResponse]) -> AppResult<Vec<u8>> {
    let mut buffer = Cursor::new(Vec::new());
    {
        let mut writer = zip::ZipWriter::new(&mut buffer);
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

        writer
            .start_file("manifest.json", options)
            .map_err(zip_err)?;
        writer.write_all(manifest_json.as_bytes()).map_err(io_err)?;

        for entry in entries {
            let name = format!("entries/{}/{}.md", entry.kind.as_str(), entry.id);
            writer.start_file(name, options).map_err(zip_err)?;
            writer
                .write_all(entry_to_markdown(entry).as_bytes())
                .map_err(io_err)?;
        }

        writer.finish().map_err(zip_err)?;
    }
    Ok(buffer.into_inner())
}

pub async fn export_entry(
    state: &AppState,
    id: uuid::Uuid,
    format: ExportFormat,
) -> AppResult<(String, Vec<u8>)> {
    let entry = entries_repo::find_by_id(&state.pool, id, true)
        .await?
        .ok_or_else(|| AppError::not_found("entry"))?;

    let response = hydrate(&state.pool, vec![entry])
        .await?
        .into_iter()
        .next()
        .expect("one entry");

    match format {
        ExportFormat::Json => Ok((
            "application/json".to_string(),
            json(&response)?.into_bytes(),
        )),
        ExportFormat::Markdown => Ok((
            "text/markdown; charset=utf-8".to_string(),
            entry_to_markdown(&response).into_bytes(),
        )),
        ExportFormat::Zip => Err(AppError::unsupported_format(
            "zip (single-entry export supports json or md)",
        )),
    }
}

pub async fn export_vault(state: &AppState, format: ExportFormat) -> AppResult<(String, Vec<u8>)> {
    let entries = entries_repo::all_ordered(&state.pool).await?;
    let hydrated = hydrate(&state.pool, entries).await?;
    let collections = collections_repo::all(&state.pool).await?;
    let tags = tags_repo::all_names(&state.pool).await?;

    let collection_payload: Vec<CollectionResponse> = collections
        .into_iter()
        .map(|c| CollectionResponse {
            id: c.id,
            name: c.name,
            slug: c.slug,
            description: c.description,
            entry_count: 0,
            created_at: c.created_at,
            updated_at: c.updated_at,
        })
        .collect();

    match format {
        ExportFormat::Json => {
            let document = ExportDocument {
                version: 1,
                exported_at: Utc::now(),
                entries: hydrated,
                collections: collection_payload,
                tags,
            };
            Ok((
                "application/json".to_string(),
                json(&document)?.into_bytes(),
            ))
        }
        ExportFormat::Markdown => {
            let mut out = String::new();
            out.push_str("# DevVault export\n\n");
            out.push_str(&format!("Exported at: {}\n\n", Utc::now().to_rfc3339()));
            for entry in &hydrated {
                out.push_str("---\n\n");
                out.push_str(&entry_to_markdown(entry));
                out.push('\n');
            }
            Ok(("text/markdown; charset=utf-8".to_string(), out.into_bytes()))
        }
        ExportFormat::Zip => {
            let document = ExportDocument {
                version: 1,
                exported_at: Utc::now(),
                entries: hydrated.clone(),
                collections: collection_payload,
                tags,
            };
            let manifest = json(&document)?;
            let bytes = build_vault_zip(&manifest, &hydrated)?;
            Ok(("application/zip".to_string(), bytes))
        }
    }
}

pub async fn import(
    state: &AppState,
    document: ExportDocument,
    policy: ConflictPolicy,
) -> AppResult<ImportSummary> {
    let mut tx = state.pool.begin().await?;

    for collection in &document.collections {
        let model = Collection {
            id: collection.id,
            name: collection.name.clone(),
            slug: collection.slug.clone(),
            description: collection.description.clone(),
            created_at: collection.created_at,
            updated_at: collection.updated_at,
        };
        collections_repo::import_upsert(&mut tx, &model).await?;
    }

    let mut summary = ImportSummary {
        imported: 0,
        skipped: 0,
        overwritten: 0,
        errors: Vec::new(),
    };

    for item in &document.entries {
        let exists = sqlx::query_scalar!("SELECT id FROM entries WHERE id = $1", item.id)
            .fetch_optional(&mut *tx)
            .await?
            .is_some();

        if exists {
            match policy {
                ConflictPolicy::Skip => {
                    summary.skipped += 1;
                    continue;
                }
                ConflictPolicy::Overwrite => {
                    sqlx::query!("DELETE FROM entries WHERE id = $1", item.id)
                        .execute(&mut *tx)
                        .await?;
                    summary.overwritten += 1;
                }
            }
        } else {
            summary.imported += 1;
        }

        let entry = Entry {
            id: item.id,
            kind: item.kind,
            title: item.title.clone(),
            body: item.body.clone(),
            language: item.language.clone(),
            shell: item.shell.clone(),
            url: item.url.clone(),
            is_pinned: item.is_pinned,
            is_archived: item.is_archived,
            created_at: item.created_at,
            updated_at: item.updated_at,
            deleted_at: item.deleted_at,
        };

        entries_repo::insert_full(&mut tx, &entry).await?;
        tags_repo::set_entry_tags(&mut tx, item.id, &item.tags).await?;
        collections_repo::set_entry_collections(&mut tx, item.id, &item.collections).await?;
    }

    tx.commit().await?;
    Ok(summary)
}
