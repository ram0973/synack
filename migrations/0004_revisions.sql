CREATE TABLE revisions (
    id         INTEGER PRIMARY KEY,
    page_id    INTEGER NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
    title      TEXT    NOT NULL,
    body_md    TEXT    NOT NULL,
    created_at TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_revisions_page ON revisions(page_id, id DESC);
