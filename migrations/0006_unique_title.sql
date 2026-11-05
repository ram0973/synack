-- Уникальность заголовка внутри категории на уровне БД
-- (ранее проверка была только в Rust, что оставляло окно для race condition).
CREATE UNIQUE INDEX idx_pages_title_unique ON pages(category_id, lower(title));
