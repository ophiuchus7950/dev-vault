use sqlx::{PgPool, Postgres, QueryBuilder};

use crate::cursor::{decode_cursor, encode_cursor};
use crate::domain::Entry;
use crate::dto::SearchMode;
use crate::error::{AppError, AppResult};
use crate::repo::entries::{push_filters, FilterOptions};

pub struct SearchFilter {
    pub q: String,
    pub mode: SearchMode,
    pub options: FilterOptions,
    pub limit: i64,
    pub cursor: Option<String>,
    pub min_trgm_score: f64,
}

pub struct SearchResult {
    pub hits: Vec<SearchHitRow>,
    pub has_more: bool,
    pub next_cursor: Option<String>,
}

#[derive(sqlx::FromRow)]
pub struct SearchHitRow {
    #[sqlx(flatten)]
    pub entry: Entry,
    pub score: f64,
    pub matched: String,
}

// -- FTS ------------------------------------------------------------------

fn push_fts_score(qb: &mut QueryBuilder<'_, Postgres>, q: &str) {
    qb.push("ts_rank_cd(e.search, websearch_to_tsquery('english', immutable_unaccent(")
        .push_bind(q.to_string())
        .push(")))");
}

fn push_fts_predicate(qb: &mut QueryBuilder<'_, Postgres>, q: &str) {
    qb.push("e.search @@ websearch_to_tsquery('english', immutable_unaccent(")
        .push_bind(q.to_string())
        .push("))");
}

// -- Trigram / substring fallback -----------------------------------------
//
// `sim` is the greatest of the word-similarity of the query against the title
// and body. Substring matching (`strpos`) covers exact fragments that are too
// short to score well with trigrams; `word_similarity` covers typos.

fn push_trgm_score(qb: &mut QueryBuilder<'_, Postgres>, q: &str) {
    qb.push("greatest(word_similarity(")
        .push_bind(q.to_string())
        .push(", e.title), word_similarity(")
        .push_bind(q.to_string())
        .push(", e.body))");
}

fn push_trgm_predicate(qb: &mut QueryBuilder<'_, Postgres>, q: &str, threshold: f64) {
    qb.push("(strpos(lower(e.title), lower(")
        .push_bind(q.to_string())
        .push(")) > 0 OR strpos(lower(e.body), lower(")
        .push_bind(q.to_string())
        .push(")) > 0 OR word_similarity(")
        .push_bind(q.to_string())
        .push(", e.title) >= ")
        .push_bind(threshold)
        .push(" OR word_similarity(")
        .push_bind(q.to_string())
        .push(", e.body) >= ")
        .push_bind(threshold)
        .push(")");
}

// -- Mode dispatch --------------------------------------------------------

fn push_score(qb: &mut QueryBuilder<'_, Postgres>, mode: SearchMode, q: &str) {
    // Wrap in an explicit float8 cast: ts_rank_cd / word_similarity return real.
    qb.push("(");
    match mode {
        SearchMode::Fts => push_fts_score(qb, q),
        SearchMode::Trgm => push_trgm_score(qb, q),
        SearchMode::Hybrid => {
            qb.push("(");
            push_fts_score(qb, q);
            qb.push(" + ");
            push_trgm_score(qb, q);
            qb.push(")");
        }
    }
    qb.push(")::float8");
}

fn push_predicate(qb: &mut QueryBuilder<'_, Postgres>, mode: SearchMode, q: &str, threshold: f64) {
    match mode {
        SearchMode::Fts => push_fts_predicate(qb, q),
        SearchMode::Trgm => push_trgm_predicate(qb, q, threshold),
        SearchMode::Hybrid => {
            qb.push("(");
            push_fts_predicate(qb, q);
            qb.push(" OR ");
            push_trgm_predicate(qb, q, threshold);
            qb.push(")");
        }
    }
}

fn matched_label(mode: SearchMode) -> &'static str {
    match mode {
        SearchMode::Fts => "ts",
        SearchMode::Trgm => "trgm",
        SearchMode::Hybrid => "hybrid",
    }
}

pub async fn search(pool: &PgPool, f: &SearchFilter) -> AppResult<SearchResult> {
    let mode = f.mode;
    let q = f.q.clone();
    let threshold = f.min_trgm_score;

    let mut qb: QueryBuilder<'_, Postgres> = QueryBuilder::new(
        "SELECT e.id, e.kind, e.title, e.body, e.language, e.shell, e.url, \
         e.is_pinned, e.is_archived, e.created_at, e.updated_at, e.deleted_at, ",
    );
    push_score(&mut qb, mode, &q);
    qb.push(" AS score, '");
    qb.push(matched_label(mode));
    qb.push("' AS matched FROM entries e WHERE e.deleted_at IS NULL AND ");
    push_predicate(&mut qb, mode, &q, threshold);

    push_filters(&mut qb, &f.options);

    // keyset pagination on (score, id) descending
    if let Some(cur) = &f.cursor {
        let payload = decode_cursor(cur)?;
        let score: f64 = payload.k.parse().map_err(|_| AppError::invalid_cursor())?;
        qb.push(" AND ((");
        push_score(&mut qb, mode, &q);
        qb.push("), e.id) < (")
            .push_bind(score)
            .push(", ")
            .push_bind(payload.id)
            .push(")");
    }

    qb.push(" ORDER BY score DESC, e.id DESC");
    qb.push(" LIMIT ").push_bind(f.limit + 1);

    let mut hits: Vec<SearchHitRow> = qb.build_query_as::<SearchHitRow>().fetch_all(pool).await?;

    let has_more = hits.len() as i64 > f.limit;
    if has_more {
        hits.truncate(f.limit as usize);
    }
    let next_cursor = if has_more {
        hits.last()
            .map(|h| encode_cursor(&format!("{}", h.score), h.entry.id))
    } else {
        None
    };

    Ok(SearchResult {
        hits,
        has_more,
        next_cursor,
    })
}
