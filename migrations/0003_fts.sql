CREATE VIRTUAL TABLE pages_fts USING fts5(
    title,
    body_md,
    content = 'pages',
    content_rowid = 'id',
    tokenize = 'unicode61'
);

CREATE TRIGGER pages_fts_insert AFTER INSERT ON pages BEGIN
    INSERT INTO pages_fts(rowid, title, body_md) VALUES (new.id, new.title, new.body_md);
END;

CREATE TRIGGER pages_fts_delete AFTER DELETE ON pages BEGIN
    INSERT INTO pages_fts(pages_fts, rowid, title, body_md)
    VALUES ('delete', old.id, old.title, old.body_md);
END;

CREATE TRIGGER pages_fts_update AFTER UPDATE ON pages BEGIN
    INSERT INTO pages_fts(pages_fts, rowid, title, body_md)
    VALUES ('delete', old.id, old.title, old.body_md);
    INSERT INTO pages_fts(rowid, title, body_md) VALUES (new.id, new.title, new.body_md);
END;

INSERT INTO pages_fts(rowid, title, body_md) SELECT id, title, body_md FROM pages;
