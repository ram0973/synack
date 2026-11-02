CREATE TABLE media (
    id         INTEGER PRIMARY KEY,
    filename   TEXT    NOT NULL UNIQUE,
    mime       TEXT    NOT NULL,
    size       INTEGER NOT NULL,
    created_at TEXT    NOT NULL DEFAULT (datetime('now'))
);
