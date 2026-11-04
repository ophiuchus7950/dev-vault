use crate::dto::{SearchHit, SearchMode, SearchPage, SearchQuery};
use crate::error::{AppError, AppResult, FieldError};
use crate::repo::entries::FilterOptions;
use crate::repo::search::{search as do_search, SearchFilter};
use crate::service::entries::hydrate;
use crate::state::AppState;

pub async fn search(state: &AppState, query: SearchQuery) -> AppResult<SearchPage> {
    let q = query.q.trim().to_string();
    if q.chars().count() < 2 {
        return Err(AppError::Validation(vec![FieldError {
            field: "q".to_string(),
            code: "length".to_string(),
            message: "q must be at least 2 characters".to_string(),
        }]));
    }

    let filter = SearchFilter {
        q,
        mode: query.mode.unwrap_or(SearchMode::Hybrid),
        options: FilterOptions {
            kind: query.kind,
            language: query.language,
            tags: query.tag,
            collection: query.collection,
            is_pinned: query.is_pinned,
            is_archived: query.is_archived,
        },
        limit: state.config.clamp_limit(query.limit),
        cursor: query.cursor,
        min_trgm_score: state.config.search_min_trgm_score,
    };

    let result = do_search(&state.pool, &filter).await?;

    let entries = result.hits.iter().map(|h| h.entry.clone()).collect();
    let hydrated = hydrate(&state.pool, entries).await?;

    let data = hydrated
        .into_iter()
        .zip(result.hits)
        .map(|(entry, hit)| SearchHit {
            entry,
            score: hit.score,
            matched: hit.matched,
        })
        .collect();

    Ok(SearchPage {
        data,
        next_cursor: result.next_cursor,
        has_more: result.has_more,
    })
}
