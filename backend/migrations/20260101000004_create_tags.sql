-- Tags are normalized and shared across entries.

CREATE TABLE tags (
    id          uuid        PRIMARY KEY,
    name        text        NOT NULL CHECK (length(btrim(name)) BETWEEN 1 AND 50),
    created_at  timestamptz NOT NULL DEFAULT now()
);

-- case-insensitive uniqueness: "Rust" and "rust" are the same tag
CREATE UNIQUE INDEX idx_tags_name_lower ON tags (lower(name));

CREATE TABLE entry_tags (
    entry_id    uuid NOT NULL REFERENCES entries(id) ON DELETE CASCADE,
    tag_id      uuid NOT NULL REFERENCES tags(id)    ON DELETE CASCADE,
    PRIMARY KEY (entry_id, tag_id)
);

CREATE INDEX idx_entry_tags_tag   ON entry_tags (tag_id);
CREATE INDEX idx_entry_tags_entry ON entry_tags (entry_id);
