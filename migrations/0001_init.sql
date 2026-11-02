CREATE TABLE categories (
    id         INTEGER PRIMARY KEY,
    parent_id  INTEGER REFERENCES categories(id) ON DELETE RESTRICT,
    position   INTEGER NOT NULL DEFAULT 0,
    name       TEXT    NOT NULL,
    slug       TEXT    NOT NULL UNIQUE,
    created_at TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_categories_parent ON categories(parent_id);

CREATE TABLE pages (
    id          INTEGER PRIMARY KEY,
    category_id INTEGER NOT NULL REFERENCES categories(id) ON DELETE RESTRICT,
    slug        TEXT    NOT NULL,
    title       TEXT    NOT NULL,
    body_md     TEXT    NOT NULL DEFAULT '',
    status      TEXT    NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'published')),
    created_at  TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT    NOT NULL DEFAULT (datetime('now')),
    UNIQUE (category_id, slug)
);

CREATE INDEX idx_pages_category ON pages(category_id);

CREATE TABLE sessions (
    token      TEXT PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    expires_at TEXT NOT NULL
);
