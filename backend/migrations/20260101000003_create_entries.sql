-- Core content table. All kinds share this table (see ADR-0003).

CREATE TABLE entries (
    id              uuid        PRIMARY KEY,

    kind            entry_kind  NOT NULL,
    title           text        NOT NULL CHECK (length(btrim(title)) BETWEEN 1 AND 300),
    body            text        NOT NULL DEFAULT '',

    -- kind-specific fields (nullable; enforced in the validation layer)
    language        text        CHECK (language IS NULL OR length(language) <= 50),
    shell           text        CHECK (shell IS NULL OR length(shell) <= 50),
    url             text        CHECK (url IS NULL OR length(url) <= 2048),

    is_pinned       boolean     NOT NULL DEFAULT false,
    is_archived     boolean     NOT NULL DEFAULT false,

    -- weighted full-text document: title (A) > language/shell (B) > body (C)
    search          tsvector    GENERATED ALWAYS AS (
        setweight(to_tsvector('english', immutable_unaccent(coalesce(title, ''))), 'A') ||
        setweight(to_tsvector('english', immutable_unaccent(coalesce(language, ''))), 'B') ||
        setweight(to_tsvector('english', immutable_unaccent(coalesce(shell, ''))),    'B') ||
        setweight(to_tsvector('english', immutable_unaccent(coalesce(body, ''))),     'C')
    ) STORED,

    created_at      timestamptz NOT NULL DEFAULT now(),
    updated_at      timestamptz NOT NULL DEFAULT now(),
    deleted_at      timestamptz,

    -- a link must carry a url
    CONSTRAINT link_requires_url CHECK (kind <> 'link' OR url IS NOT NULL)
);

CREATE TRIGGER trg_entries_updated_at
    BEFORE UPDATE ON entries
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();
