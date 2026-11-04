-- Indexes for search, filtering and cursor pagination.

-- Full-text search
CREATE INDEX idx_entries_search ON entries USING gin (search);

-- Fuzzy / partial matching (title weighted more than body)
CREATE INDEX idx_entries_title_trgm ON entries USING gin (title gin_trgm_ops);
CREATE INDEX idx_entries_body_trgm  ON entries USING gin (body  gin_trgm_ops);

-- Listing & cursor pagination: live rows ordered by created_at
CREATE INDEX idx_entries_live_created
    ON entries (created_at DESC, id DESC)
    WHERE deleted_at IS NULL;

-- "recently updated" sort
CREATE INDEX idx_entries_live_updated
    ON entries (updated_at DESC, id DESC)
    WHERE deleted_at IS NULL;

-- kind filter among live rows
CREATE INDEX idx_entries_live_kind
    ON entries (kind)
    WHERE deleted_at IS NULL;

-- pinned-first ordering
CREATE INDEX idx_entries_live_pinned
    ON entries (is_pinned DESC, created_at DESC, id DESC)
    WHERE deleted_at IS NULL;

-- language filter among live snippets
CREATE INDEX idx_entries_live_language
    ON entries (language)
    WHERE deleted_at IS NULL AND language IS NOT NULL;

-- Trash view
CREATE INDEX idx_entries_trash
    ON entries (deleted_at DESC, id DESC)
    WHERE deleted_at IS NOT NULL;
