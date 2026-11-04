-- Optional collections (folders on top of flat tags).

CREATE TABLE collections (
    id          uuid        PRIMARY KEY,
    name        text        NOT NULL CHECK (length(btrim(name)) BETWEEN 1 AND 100),
    slug        text        NOT NULL CHECK (slug ~ '^[a-z0-9]+(?:-[a-z0-9]+)*$'),
    description text        NOT NULL DEFAULT '',
    created_at  timestamptz NOT NULL DEFAULT now(),
    updated_at  timestamptz NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX idx_collections_slug ON collections (slug);

CREATE TRIGGER trg_collections_updated_at
    BEFORE UPDATE ON collections
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE entry_collections (
    entry_id        uuid NOT NULL REFERENCES entries(id)      ON DELETE CASCADE,
    collection_id   uuid NOT NULL REFERENCES collections(id)   ON DELETE CASCADE,
    PRIMARY KEY (entry_id, collection_id)
);

CREATE INDEX idx_entry_collections_collection ON entry_collections (collection_id);
