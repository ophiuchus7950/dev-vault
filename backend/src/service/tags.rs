use uuid::Uuid;
use validator::Validate;

use crate::domain::Tag;
use crate::dto::{RenameTagRequest, TagList};
use crate::error::{AppError, AppResult};
use crate::repo::tags as repo;
use crate::state::AppState;

pub async fn list(state: &AppState, q: Option<String>) -> AppResult<TagList> {
    let query = q.map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
    let tags = repo::list_tags(&state.pool, query.as_deref()).await?;
    Ok(TagList { data: tags })
}

pub async fn rename(state: &AppState, id: Uuid, req: RenameTagRequest) -> AppResult<Tag> {
    req.validate()?;
    repo::rename_tag(&state.pool, id, &req.name).await
}

pub async fn delete(state: &AppState, id: Uuid) -> AppResult<()> {
    let affected = repo::delete_tag(&state.pool, id).await?;
    if affected == 0 {
        return Err(AppError::not_found("tag"));
    }
    Ok(())
}
