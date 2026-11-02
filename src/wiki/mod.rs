use std::collections::HashMap;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, patch, post},
    Json, Router,
};
use pulldown_cmark::{html, Event, Options, Parser, Tag, TagEnd};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::sqlite::SqlitePool;
use sqlx::FromRow;

use crate::error::AppError;

pub fn router() -> Router<SqlitePool> {
    Router::<SqlitePool>::new()
        .route("/tree", get(tree))
        .route("/categories", post(create_category))
        .route(
            "/categories/{id}",
            patch(update_category).delete(delete_category),
        )
        .route("/categories/{id}/move", post(move_category))
        .route("/pages", get(list_pages).post(create_page))
        .route("/pages/by-path", get(page_by_path))
        .route("/pages/{id}/revisions", get(list_revisions))
        .route("/revisions/{id}", get(get_revision))
        .route("/revisions/{id}/restore", post(restore_revision))
        .route(
            "/pages/{id}",
            get(get_page).patch(update_page).delete(delete_page),
        )
        .route("/resolve", get(resolve_path))
        .route("/search", get(search))
        .route("/preview", post(preview))
        .route("/categories/by-path", get(category_by_path))
}

// ---------- модели ----------

#[derive(Debug, Clone, Serialize, FromRow)]
struct Category {
    id: i64,
    parent_id: Option<i64>,
    position: i64,
    name: String,
    slug: String,
}

#[derive(Debug, Serialize)]
struct CategoryNode {
    #[serde(flatten)]
    category: Category,
    children: Vec<CategoryNode>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
struct Page {
    id: i64,
    category_id: i64,
    slug: String,
    title: String,
    body_md: String,
    status: String,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Serialize)]
struct PageOut {
    id: i64,
    category_id: i64,
    slug: String,
    title: String,
    body_md: String,
    status: String,
    html: String,
    path: String,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Serialize, FromRow)]
struct PageSummary {
    id: i64,
    category_id: i64,
    slug: String,
    title: String,
    status: String,
    updated_at: String,
}

#[derive(Debug, Deserialize)]
struct CreateCategory {
    parent_id: Option<i64>,
    name: String,
    slug: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UpdateCategory {
    name: Option<String>,
    slug: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MoveCategory {
    parent_id: Option<i64>,
    position: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct CreatePage {
    category_id: i64,
    title: String,
    body_md: Option<String>,
    slug: Option<String>,
    status: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UpdatePage {
    category_id: Option<i64>,
    title: Option<String>,
    body_md: Option<String>,
    slug: Option<String>,
    status: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ListPagesQuery {
    category: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct PathQuery {
    path: String,
}

// ---------- категории ----------

async fn tree(State(pool): State<SqlitePool>) -> Result<Json<Vec<CategoryNode>>, AppError> {
    let rows =
        sqlx::query_as::<_, Category>("SELECT id, parent_id, position, name, slug FROM categories")
            .fetch_all(&pool)
            .await?;

    Ok(Json(build_tree(rows)))
}

async fn create_category(
    State(pool): State<SqlitePool>,
    Json(body): Json<CreateCategory>,
) -> Result<(StatusCode, Json<Category>), AppError> {
    let name = body.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::bad_request("name is required"));
    }

    if let Some(parent_id) = body.parent_id {
        find_category(&pool, parent_id).await?;
    }

    let slug = body
        .slug
        .as_deref()
        .map(slugify)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| slugify(&name));
    let slug = if slug.is_empty() {
        "category".to_string()
    } else {
        slug
    };

    let position = next_position(&pool, body.parent_id).await?;

    let row = sqlx::query_as::<_, Category>(
        "INSERT INTO categories (parent_id, position, name, slug)
         VALUES (?, ?, ?, ?)
         RETURNING id, parent_id, position, name, slug",
    )
    .bind(body.parent_id)
    .bind(position)
    .bind(&name)
    .bind(&slug)
    .fetch_one(&pool)
    .await?;

    Ok((StatusCode::CREATED, Json(row)))
}

async fn update_category(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(body): Json<UpdateCategory>,
) -> Result<Json<Category>, AppError> {
    let current = find_category(&pool, id).await?;

    let name = match body.name {
        Some(name) => {
            let trimmed = name.trim().to_string();
            if trimmed.is_empty() {
                return Err(AppError::bad_request("name is required"));
            }
            trimmed
        }
        None => current.name,
    };

    let slug = match body.slug {
        Some(slug) => {
            let value = slugify(&slug);
            if value.is_empty() {
                return Err(AppError::bad_request("slug is invalid"));
            }
            value
        }
        None => current.slug,
    };

    let row = sqlx::query_as::<_, Category>(
        "UPDATE categories SET name = ?, slug = ? WHERE id = ?
         RETURNING id, parent_id, position, name, slug",
    )
    .bind(name)
    .bind(slug)
    .bind(id)
    .fetch_one(&pool)
    .await?;

    Ok(Json(row))
}

async fn move_category(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(body): Json<MoveCategory>,
) -> Result<Json<Category>, AppError> {
    find_category(&pool, id).await?;

    if let Some(parent_id) = body.parent_id {
        if parent_id == id {
            return Err(AppError::bad_request("cannot move a category into itself"));
        }
        find_category(&pool, parent_id).await?;
        if is_descendant(&pool, id, parent_id).await? {
            return Err(AppError::bad_request(
                "cannot move a category into its own subtree",
            ));
        }
    }

    let position = match body.position {
        Some(position) => position,
        None => next_position(&pool, body.parent_id).await?,
    };

    let row = sqlx::query_as::<_, Category>(
        "UPDATE categories SET parent_id = ?, position = ? WHERE id = ?
         RETURNING id, parent_id, position, name, slug",
    )
    .bind(body.parent_id)
    .bind(position)
    .bind(id)
    .fetch_one(&pool)
    .await?;

    Ok(Json(row))
}

async fn delete_category(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    find_category(&pool, id).await?;

    let children: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM categories WHERE parent_id = ?")
        .bind(id)
        .fetch_one(&pool)
        .await?;
    if children > 0 {
        return Err(AppError::conflict("category has children"));
    }

    let pages: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM pages WHERE category_id = ?")
        .bind(id)
        .fetch_one(&pool)
        .await?;
    if pages > 0 {
        return Err(AppError::conflict("category has pages"));
    }

    sqlx::query("DELETE FROM categories WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

// ---------- страницы ----------

#[derive(Debug, Deserialize)]
struct SearchQuery {
    q: String,
}

#[derive(Debug, Serialize)]
struct SearchHit {
    id: i64,
    title: String,
    path: String,
    status: String,
    snippet: String,
}

async fn search(
    State(pool): State<SqlitePool>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Vec<SearchHit>>, AppError> {
    let terms: Vec<String> = query
        .q
        .split_whitespace()
        .map(|term| {
            term.chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>()
        })
        .filter(|term| !term.is_empty())
        .collect();

    if terms.is_empty() {
        return Ok(Json(Vec::new()));
    }

    let match_query = terms
        .iter()
        .map(|term| format!("\"{term}\"*"))
        .collect::<Vec<_>>()
        .join(" ");

    let rows = sqlx::query_as::<_, (i64, i64, String, String, String, String)>(
        "SELECT p.id, p.category_id, p.title, p.status, p.slug, p.body_md
         FROM pages_fts
         JOIN pages p ON p.id = pages_fts.rowid
         WHERE pages_fts MATCH ?
         ORDER BY bm25(pages_fts, 5.0, 1.0)
         LIMIT 20",
    )
    .bind(match_query)
    .fetch_all(&pool)
    .await?;

    let mut hits = Vec::new();
    for (id, category_id, title, status, slug, body_md) in rows {
        let chain = category_chain(&pool, category_id).await?;
        hits.push(SearchHit {
            id,
            title,
            path: page_path(&chain, &slug),
            status,
            snippet: make_snippet(&body_md, &terms),
        });
    }

    Ok(Json(hits))
}

/// Грубое превращение HTML в текст: теги выкидываем, сущности раскрываем.
fn html_to_text(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;

    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                out.push(' ');
            }
            other if !in_tag => out.push(other),
            _ => {}
        }
    }

    let decoded = out
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ");

    decoded.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Кусок текста вокруг первого совпадения, совпадения помечены [[ ]].
fn make_snippet(markdown: &str, terms: &[String]) -> String {
    let text = html_to_text(&render_markdown(markdown));
    let lower = text.to_lowercase();

    let mut ranges: Vec<(usize, usize)> = Vec::new();

    for term in terms {
        let needle = term.to_lowercase();
        if needle.is_empty() {
            continue;
        }

        let mut offset = 0;
        while let Some(found) = lower[offset..].find(&needle) {
            let start = offset + found;
            ranges.push((start, start + needle.len()));
            offset = start + needle.len();
            if ranges.len() >= 32 {
                break;
            }
        }
    }

    if ranges.is_empty() {
        return text.chars().take(160).collect();
    }

    ranges.sort();
    let mut merged: Vec<(usize, usize)> = Vec::new();
    for (start, end) in ranges {
        match merged.last_mut() {
            Some(last) if start <= last.1 => last.1 = last.1.max(end),
            _ => merged.push((start, end)),
        }
    }

    let first = merged[0].0;
    let mut window_start = first.saturating_sub(70);
    while window_start > 0 && !text.is_char_boundary(window_start) {
        window_start -= 1;
    }

    let mut window_end = (first + 160).min(text.len());
    while window_end < text.len() && !text.is_char_boundary(window_end) {
        window_end += 1;
    }

    let mut out = String::new();
    if window_start > 0 {
        out.push('…');
    }

    let mut cursor = window_start;
    for (start, end) in merged {
        if start < window_start || end > window_end {
            continue;
        }
        out.push_str(&text[cursor..start]);
        out.push_str("[[");
        out.push_str(&text[start..end]);
        out.push_str("]]");
        cursor = end;
    }

    out.push_str(&text[cursor..window_end]);
    if window_end < text.len() {
        out.push('…');
    }

    out
}

async fn list_pages(
    State(pool): State<SqlitePool>,
    Query(query): Query<ListPagesQuery>,
) -> Result<Json<Vec<PageSummary>>, AppError> {
    let rows = match query.category {
        Some(category_id) => {
            sqlx::query_as::<_, PageSummary>(
                "SELECT id, category_id, slug, title, status, updated_at
                 FROM pages WHERE category_id = ? ORDER BY title",
            )
            .bind(category_id)
            .fetch_all(&pool)
            .await?
        }
        None => {
            sqlx::query_as::<_, PageSummary>(
                "SELECT id, category_id, slug, title, status, updated_at
                 FROM pages ORDER BY title",
            )
            .fetch_all(&pool)
            .await?
        }
    };

    Ok(Json(rows))
}

async fn get_page(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<PageOut>, AppError> {
    let page = find_page(&pool, id).await?;
    let chain = category_chain(&pool, page.category_id).await?;

    Ok(Json(to_page_out(page, &chain)))
}

async fn create_page(
    State(pool): State<SqlitePool>,
    Json(body): Json<CreatePage>,
) -> Result<(StatusCode, Json<PageOut>), AppError> {
    let title = body.title.trim().to_string();
    if title.is_empty() {
        return Err(AppError::bad_request("title is required"));
    }

    let chain = category_chain(&pool, body.category_id).await?;

    let slug = body
        .slug
        .as_deref()
        .map(slugify)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| slugify(&title));
    let slug = if slug.is_empty() {
        "page".to_string()
    } else {
        slug
    };

    let status = normalize_status(body.status.as_deref())?;
    let body_md = body.body_md.unwrap_or_default();

    let page = sqlx::query_as::<_, Page>(
        "INSERT INTO pages (category_id, slug, title, body_md, status)
         VALUES (?, ?, ?, ?, ?)
         RETURNING id, category_id, slug, title, body_md, status, created_at, updated_at",
    )
    .bind(body.category_id)
    .bind(&slug)
    .bind(&title)
    .bind(&body_md)
    .bind(&status)
    .fetch_one(&pool)
    .await?;

    save_revision(&pool, &page).await?;

    Ok((StatusCode::CREATED, Json(to_page_out(page, &chain))))
}

async fn update_page(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(body): Json<UpdatePage>,
) -> Result<Json<PageOut>, AppError> {
    let current = find_page(&pool, id).await?;

    let category_id = match body.category_id {
        Some(category_id) => {
            category_chain(&pool, category_id).await?;
            category_id
        }
        None => current.category_id,
    };

    let title = match body.title {
        Some(title) => {
            let trimmed = title.trim().to_string();
            if trimmed.is_empty() {
                return Err(AppError::bad_request("title is required"));
            }
            trimmed
        }
        None => current.title,
    };

    let slug = match body.slug {
        Some(slug) => {
            let value = slugify(&slug);
            if value.is_empty() {
                return Err(AppError::bad_request("slug is invalid"));
            }
            value
        }
        None => current.slug,
    };

    let body_md = body.body_md.unwrap_or(current.body_md);
    let status = match body.status {
        Some(status) => normalize_status(Some(&status))?,
        None => current.status,
    };

    let page = sqlx::query_as::<_, Page>(
        "UPDATE pages
         SET category_id = ?, slug = ?, title = ?, body_md = ?, status = ?, updated_at = datetime('now')
         WHERE id = ?
         RETURNING id, category_id, slug, title, body_md, status, created_at, updated_at",
    )
    .bind(category_id)
    .bind(&slug)
    .bind(&title)
    .bind(&body_md)
    .bind(&status)
    .bind(id)
    .fetch_one(&pool)
    .await?;

    save_revision(&pool, &page).await?;

    let chain = category_chain(&pool, category_id).await?;

    Ok(Json(to_page_out(page, &chain)))
}

async fn delete_page(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    find_page(&pool, id).await?;

    sqlx::query("DELETE FROM pages WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn page_by_path(
    State(pool): State<SqlitePool>,
    Query(query): Query<PathQuery>,
) -> Result<Json<PageOut>, AppError> {
    let raw = query.path.trim().trim_matches('/').to_string();
    if raw.is_empty() {
        return Err(AppError::bad_request("path is required"));
    }

    let parts: Vec<&str> = raw.split('/').filter(|part| !part.is_empty()).collect();
    if parts.len() < 2 {
        return Err(AppError::not_found("page not found"));
    }

    let (category_slugs, page_slug) = parts.split_at(parts.len() - 1);

    let mut parent: Option<i64> = None;
    for slug in category_slugs {
        let category = sqlx::query_as::<_, Category>(
            "SELECT id, parent_id, position, name, slug
             FROM categories WHERE parent_id IS ? AND slug = ?",
        )
        .bind(parent)
        .bind(*slug)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::not_found(format!("category {slug} not found")))?;

        parent = Some(category.id);
    }

    let category_id = parent.ok_or_else(|| AppError::bad_request("category is required"))?;

    let page = sqlx::query_as::<_, Page>(
        "SELECT id, category_id, slug, title, body_md, status, created_at, updated_at
         FROM pages WHERE category_id = ? AND slug = ?",
    )
    .bind(category_id)
    .bind(page_slug[0])
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::not_found("page not found"))?;

    let chain = category_chain(&pool, category_id).await?;

    Ok(Json(to_page_out(page, &chain)))
}

#[derive(Debug, Deserialize)]
struct PreviewRequest {
    body_md: String,
}

#[derive(Debug, Serialize)]
struct CategoryView {
    #[serde(flatten)]
    category: Category,
    path: String,
    children: Vec<Category>,
    pages: Vec<PageSummary>,
}

async fn preview(Json(body): Json<PreviewRequest>) -> Json<Value> {
    Json(json!({ "html": render_markdown(&body.body_md) }))
}

async fn category_by_path(
    State(pool): State<SqlitePool>,
    Query(query): Query<PathQuery>,
) -> Result<Json<CategoryView>, AppError> {
    let raw = query.path.trim().trim_matches('/').to_string();
    let slugs: Vec<&str> = raw.split('/').filter(|part| !part.is_empty()).collect();

    let category = resolve_category(&pool, &slugs).await?;
    let chain = category_chain(&pool, category.id).await?;
    let path = format!(
        "/{}",
        chain
            .iter()
            .map(|c| c.slug.as_str())
            .collect::<Vec<_>>()
            .join("/")
    );

    let children = sqlx::query_as::<_, Category>(
        "SELECT id, parent_id, position, name, slug
         FROM categories WHERE parent_id = ? ORDER BY position, id",
    )
    .bind(category.id)
    .fetch_all(&pool)
    .await?;

    let pages = sqlx::query_as::<_, PageSummary>(
        "SELECT id, category_id, slug, title, status, updated_at
         FROM pages WHERE category_id = ? ORDER BY title",
    )
    .bind(category.id)
    .fetch_all(&pool)
    .await?;

    Ok(Json(CategoryView {
        category,
        path,
        children,
        pages,
    }))
}

async fn resolve_path(
    State(pool): State<SqlitePool>,
    Query(query): Query<PathQuery>,
) -> Result<Json<Value>, AppError> {
    let path = query.path.trim().trim_matches('/').to_string();

    if let Ok(Json(page)) =
        page_by_path(State(pool.clone()), Query(PathQuery { path: path.clone() })).await
    {
        return Ok(Json(json!({ "kind": "page", "page": page })));
    }

    if let Ok(Json(category)) = category_by_path(State(pool), Query(PathQuery { path })).await {
        return Ok(Json(json!({ "kind": "category", "category": category })));
    }

    Err(AppError::not_found("not found"))
}

// ---------- вспомогательное ----------

#[derive(Debug, Serialize)]
struct RevisionSummary {
    id: i64,
    title: String,
    created_at: String,
    size: i64,
    preview: String,
}

#[derive(Debug, Serialize, FromRow)]
struct Revision {
    id: i64,
    page_id: i64,
    title: String,
    body_md: String,
    created_at: String,
}

async fn save_revision(pool: &SqlitePool, page: &Page) -> Result<(), AppError> {
    sqlx::query("INSERT INTO revisions (page_id, title, body_md) VALUES (?, ?, ?)")
        .bind(page.id)
        .bind(&page.title)
        .bind(&page.body_md)
        .execute(pool)
        .await?;

    Ok(())
}

async fn list_revisions(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<Vec<RevisionSummary>>, AppError> {
    find_page(&pool, id).await?;

    let rows = sqlx::query_as::<_, (i64, String, String, i64, String)>(
        "SELECT id, title, created_at, length(body_md), body_md
         FROM revisions WHERE page_id = ? ORDER BY id DESC LIMIT 50",
    )
    .bind(id)
    .fetch_all(&pool)
    .await?;

    let items = rows
        .into_iter()
        .map(|(id, title, created_at, size, body_md)| {
            let text = html_to_text(&render_markdown(&body_md));
            RevisionSummary {
                id,
                title,
                created_at,
                size,
                preview: text.chars().take(90).collect(),
            }
        })
        .collect();

    Ok(Json(items))
}

async fn get_revision(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<Revision>, AppError> {
    let revision = sqlx::query_as::<_, Revision>(
        "SELECT id, page_id, title, body_md, created_at FROM revisions WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::not_found("revision not found"))?;

    Ok(Json(revision))
}

async fn restore_revision(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<PageOut>, AppError> {
    let revision = sqlx::query_as::<_, Revision>(
        "SELECT id, page_id, title, body_md, created_at FROM revisions WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::not_found("revision not found"))?;

    find_page(&pool, revision.page_id).await?;

    let page = sqlx::query_as::<_, Page>(
        "UPDATE pages SET title = ?, body_md = ?, updated_at = datetime('now')
         WHERE id = ?
         RETURNING id, category_id, slug, title, body_md, status, created_at, updated_at",
    )
    .bind(&revision.title)
    .bind(&revision.body_md)
    .bind(revision.page_id)
    .fetch_one(&pool)
    .await?;

    save_revision(&pool, &page).await?;

    let chain = category_chain(&pool, page.category_id).await?;

    Ok(Json(to_page_out(page, &chain)))
}

async fn find_category(pool: &SqlitePool, id: i64) -> Result<Category, AppError> {
    sqlx::query_as::<_, Category>(
        "SELECT id, parent_id, position, name, slug FROM categories WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::not_found("category not found"))
}

async fn find_page(pool: &SqlitePool, id: i64) -> Result<Page, AppError> {
    sqlx::query_as::<_, Page>(
        "SELECT id, category_id, slug, title, body_md, status, created_at, updated_at
         FROM pages WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::not_found("page not found"))
}

async fn resolve_category(pool: &SqlitePool, slugs: &[&str]) -> Result<Category, AppError> {
    let mut parent: Option<i64> = None;
    let mut found: Option<Category> = None;

    for slug in slugs {
        let category = sqlx::query_as::<_, Category>(
            "SELECT id, parent_id, position, name, slug
             FROM categories WHERE parent_id IS ? AND slug = ?",
        )
        .bind(parent)
        .bind(*slug)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::not_found(format!("category {slug} not found")))?;

        parent = Some(category.id);
        found = Some(category);
    }

    found.ok_or_else(|| AppError::bad_request("category is required"))
}

async fn category_chain(pool: &SqlitePool, id: i64) -> Result<Vec<Category>, AppError> {
    let mut chain = Vec::new();
    let mut current = id;
    let mut steps = 0;

    loop {
        let category = find_category(pool, current).await?;
        let parent = category.parent_id;
        chain.push(category);

        match parent {
            Some(parent_id) => {
                current = parent_id;
                steps += 1;
                if steps > 1024 {
                    return Err(AppError::conflict("category tree is corrupted"));
                }
            }
            None => break,
        }
    }

    chain.reverse();
    Ok(chain)
}

fn to_page_out(page: Page, chain: &[Category]) -> PageOut {
    let path = page_path(chain, &page.slug);
    let markdown = strip_duplicate_title(&page.body_md, &page.title);
    let html = render_markdown(&markdown);

    PageOut {
        id: page.id,
        category_id: page.category_id,
        slug: page.slug,
        title: page.title,
        body_md: page.body_md,
        status: page.status,
        html,
        path,
        created_at: page.created_at,
        updated_at: page.updated_at,
    }
}

fn page_path(chain: &[Category], slug: &str) -> String {
    let mut parts: Vec<&str> = chain
        .iter()
        .map(|category| category.slug.as_str())
        .collect();
    parts.push(slug);
    format!("/{}", parts.join("/"))
}

fn normalize_status(status: Option<&str>) -> Result<String, AppError> {
    match status.unwrap_or("draft") {
        "draft" => Ok("draft".to_string()),
        "published" => Ok("published".to_string()),
        other => Err(AppError::bad_request(format!("unknown status: {other}"))),
    }
}

/// Убирает первый заголовок H1, если он повторяет заголовок страницы.
fn strip_duplicate_title(markdown: &str, title: &str) -> String {
    let mut lines = markdown.lines();

    if let Some(first) = lines.next() {
        let trimmed = first.trim();
        if let Some(heading) = trimmed.strip_prefix("# ") {
            if heading.trim().eq_ignore_ascii_case(title.trim()) {
                return lines
                    .collect::<Vec<_>>()
                    .join("\n")
                    .trim_start()
                    .to_string();
            }
        }
    }

    markdown.to_string()
}

/// Достаёт директивы ::video{...}, ::youtube{...} и блоки :::note ... ::: до рендера Markdown.
fn is_callout_kind(kind: &str) -> bool {
    matches!(kind, "note" | "tip" | "warning" | "caution")
}

fn extract_directives(markdown: &str) -> (String, Vec<(String, String)>) {
    let mut prepared = String::with_capacity(markdown.len());
    let mut inserts: Vec<(String, String)> = Vec::new();
    let mut lines = markdown.lines().peekable();

    while let Some(line) = lines.next() {
        let trimmed = line.trim();

        if let Some(kind) = trimmed.strip_prefix(":::").map(str::trim) {
            if is_callout_kind(kind) {
                let mut inner = String::new();
                let mut closed = false;

                for next in lines.by_ref() {
                    if next.trim() == ":::" {
                        closed = true;
                        break;
                    }
                    inner.push_str(next);
                    inner.push('\n');
                }

                if closed {
                    let token = format!("DSHCALLOUT{}", inserts.len());
                    let html = format!(
                        "<div class=\"callout callout-{kind}\">{}</div>",
                        render_markdown(&inner)
                    );
                    prepared.push_str(&token);
                    prepared.push('\n');
                    inserts.push((token, html));
                    continue;
                }

                prepared.push_str(line);
                prepared.push('\n');
                prepared.push_str(&inner);
                continue;
            }
        }

        if let Some(body) = trimmed
            .strip_prefix("::video{")
            .and_then(|rest| rest.strip_suffix('}'))
        {
            if let Some(src) = attr_value(body, "src") {
                if let Some(html) = video_html(&src) {
                    let token = format!("DSHVIDEO{}", inserts.len());
                    prepared.push_str(&token);
                    prepared.push('\n');
                    inserts.push((token, html));
                    continue;
                }
            }
        }

        if let Some(body) = trimmed
            .strip_prefix("::youtube{")
            .and_then(|rest| rest.strip_suffix('}'))
        {
            if let Some(id) = attr_value(body, "id") {
                if let Some(html) = youtube_html(&id) {
                    let token = format!("DSHYOUTUBE{}", inserts.len());
                    prepared.push_str(&token);
                    prepared.push('\n');
                    inserts.push((token, html));
                    continue;
                }
            }
        }

        prepared.push_str(line);
        prepared.push('\n');
    }

    (prepared, inserts)
}

fn attr_value(body: &str, key: &str) -> Option<String> {
    let prefix = format!("{key}=");

    for part in body.split(',') {
        let part = part.trim();
        if let Some(raw) = part.strip_prefix(&prefix) {
            let value = raw.trim().trim_matches('"').trim_matches('\'').trim();
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }

    None
}

fn escape_attr(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn video_html(src: &str) -> Option<String> {
    let allowed =
        src.starts_with("/media/") || src.starts_with("https://") || src.starts_with("http://");

    if !allowed {
        return None;
    }

    Some(format!(
        "<figure class=\"video\"><video controls preload=\"metadata\" src=\"{}\"></video></figure>",
        escape_attr(src)
    ))
}

fn youtube_html(id: &str) -> Option<String> {
    let valid = id.len() >= 6
        && id.len() <= 24
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');

    if !valid {
        return None;
    }

    Some(format!(
        "<figure class=\"video\"><iframe src=\"https://www.youtube-nocookie.com/embed/{id}\" title=\"YouTube\" loading=\"lazy\" allow=\"encrypted-media; picture-in-picture\" allowfullscreen></iframe></figure>"
    ))
}

fn render_markdown(markdown: &str) -> String {
    let (prepared, inserts) = extract_directives(markdown);

    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);

    let events: Vec<Event> = Parser::new_ext(&prepared, options).collect();
    let mut transformed: Vec<Event> = Vec::with_capacity(events.len());
    let mut used_ids: HashMap<String, usize> = HashMap::new();
    let mut index = 0;

    while index < events.len() {
        if let Event::Start(Tag::Heading {
            level,
            id: None,
            classes,
            attrs,
        }) = &events[index]
        {
            let mut text = String::new();
            let mut cursor = index + 1;

            while cursor < events.len() {
                match &events[cursor] {
                    Event::End(TagEnd::Heading(_)) => break,
                    Event::Text(value) | Event::Code(value) => text.push_str(value),
                    _ => {}
                }
                cursor += 1;
            }

            let mut slug = slugify(&text);
            if slug.is_empty() {
                slug = "section".to_string();
            }

            let count = used_ids.entry(slug.clone()).or_insert(0);
            *count += 1;

            let anchor = if *count > 1 {
                format!("{slug}-{count}")
            } else {
                slug
            };

            transformed.push(Event::Start(Tag::Heading {
                level: *level,
                id: Some(anchor.into()),
                classes: classes.clone(),
                attrs: attrs.clone(),
            }));

            index += 1;
            continue;
        }

        transformed.push(events[index].clone());
        index += 1;
    }

    let mut html_output = String::new();
    html::push_html(&mut html_output, transformed.into_iter());

    let mut cleaner = ammonia::Builder::default();
    cleaner.add_generic_attributes(["class", "id"]);
    let mut safe = cleaner.clean(&html_output).to_string();

    for (token, html) in inserts {
        let wrapped = format!("<p>{token}</p>");
        safe = safe.replace(&wrapped, &html);
        safe = safe.replace(&token, &html);
    }

    safe
}

async fn next_position(pool: &SqlitePool, parent_id: Option<i64>) -> Result<i64, AppError> {
    let next: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(position), -1) + 1 FROM categories WHERE parent_id IS ?",
    )
    .bind(parent_id)
    .fetch_one(pool)
    .await?;

    Ok(next)
}

async fn is_descendant(pool: &SqlitePool, ancestor: i64, candidate: i64) -> Result<bool, AppError> {
    let mut current = Some(candidate);
    let mut steps = 0;

    while let Some(id) = current {
        if id == ancestor {
            return Ok(true);
        }
        steps += 1;
        if steps > 1024 {
            return Err(AppError::conflict("category tree is corrupted"));
        }

        let parent: Option<Option<i64>> =
            sqlx::query_scalar("SELECT parent_id FROM categories WHERE id = ?")
                .bind(id)
                .fetch_optional(pool)
                .await?;

        match parent {
            Some(parent_id) => current = parent_id,
            None => return Ok(false),
        }
    }

    Ok(false)
}

fn build_tree(rows: Vec<Category>) -> Vec<CategoryNode> {
    let mut by_parent: HashMap<Option<i64>, Vec<Category>> = HashMap::new();
    for row in rows {
        by_parent.entry(row.parent_id).or_default().push(row);
    }
    for siblings in by_parent.values_mut() {
        siblings.sort_by_key(|c| (c.position, c.id));
    }

    fn take(
        parent: Option<i64>,
        map: &mut HashMap<Option<i64>, Vec<Category>>,
    ) -> Vec<CategoryNode> {
        map.remove(&parent)
            .unwrap_or_default()
            .into_iter()
            .map(|category| {
                let children = take(Some(category.id), map);
                CategoryNode { category, children }
            })
            .collect()
    }

    take(None, &mut by_parent)
}

fn slugify(input: &str) -> String {
    let mut out = String::new();
    let mut dashed = false;

    for ch in input.chars() {
        let ch = ch.to_lowercase().next().unwrap_or(ch);
        for c in translit(ch).chars() {
            if c.is_ascii_alphanumeric() {
                out.push(c.to_ascii_lowercase());
                dashed = false;
            } else if !dashed && !out.is_empty() {
                out.push('-');
                dashed = true;
            }
        }
    }

    out.trim_end_matches('-').to_string()
}

fn translit(ch: char) -> String {
    let mapped = match ch {
        'а' => "a",
        'б' => "b",
        'в' => "v",
        'г' => "g",
        'д' => "d",
        'е' => "e",
        'ё' => "e",
        'ж' => "zh",
        'з' => "z",
        'и' => "i",
        'й' => "y",
        'к' => "k",
        'л' => "l",
        'м' => "m",
        'н' => "n",
        'о' => "o",
        'п' => "p",
        'р' => "r",
        'с' => "s",
        'т' => "t",
        'у' => "u",
        'ф' => "f",
        'х' => "h",
        'ц' => "ts",
        'ч' => "ch",
        'ш' => "sh",
        'щ' => "sch",
        'ъ' => "",
        'ы' => "y",
        'ь' => "",
        'э' => "e",
        'ю' => "yu",
        'я' => "ya",
        _ => return ch.to_string(),
    };
    mapped.to_string()
}

// ---------- тесты ----------

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
    use std::str::FromStr;

    async fn test_pool() -> SqlitePool {
        let options = SqliteConnectOptions::from_str("sqlite::memory:").unwrap();
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        pool
    }

    #[test]
    fn slugify_transliterates() {
        assert_eq!(slugify("Привет мир"), "privet-mir");
        assert_eq!(slugify("  Rust / Async  "), "rust-async");
        assert_eq!(slugify("Щука"), "schuka");
    }

    #[test]
    fn build_tree_nests_and_sorts() {
        let rows = vec![
            Category {
                id: 1,
                parent_id: None,
                position: 0,
                name: "A".into(),
                slug: "a".into(),
            },
            Category {
                id: 2,
                parent_id: None,
                position: 1,
                name: "B".into(),
                slug: "b".into(),
            },
            Category {
                id: 3,
                parent_id: Some(1),
                position: 0,
                name: "A1".into(),
                slug: "a1".into(),
            },
        ];
        let tree = build_tree(rows);
        assert_eq!(tree.len(), 2);
        assert_eq!(tree[0].category.name, "A");
        assert_eq!(tree[0].children[0].category.name, "A1");
    }

    #[test]
    fn markdown_is_sanitized() {
        let html = render_markdown("<script>alert(1)</script>\n\n**bold**");
        assert!(!html.contains("<script"));
        assert!(html.contains("<strong>bold</strong>"));
    }

    #[tokio::test]
    async fn create_move_and_reject_cycle() {
        let pool = test_pool().await;
        let state = State(pool.clone());

        let (_, Json(a)) = create_category(
            state.clone(),
            Json(CreateCategory {
                parent_id: None,
                name: "Rust".into(),
                slug: None,
            }),
        )
        .await
        .unwrap();

        let (_, Json(b)) = create_category(
            state.clone(),
            Json(CreateCategory {
                parent_id: Some(a.id),
                name: "Async".into(),
                slug: None,
            }),
        )
        .await
        .unwrap();

        assert_eq!(b.parent_id, Some(a.id));

        let err = move_category(
            state.clone(),
            Path(a.id),
            Json(MoveCategory {
                parent_id: Some(b.id),
                position: None,
            }),
        )
        .await
        .unwrap_err();
        assert_eq!(err.status, StatusCode::BAD_REQUEST);

        let err = delete_category(state.clone(), Path(a.id))
            .await
            .unwrap_err();
        assert_eq!(err.status, StatusCode::CONFLICT);
    }

    #[test]
    fn snippet_marks_terms() {
        let snippet = make_snippet(
            "# Введение\n\nТекст про **Markdown** и таблицы.",
            &["markdown".to_string()],
        );

        assert!(snippet.contains("[[Markdown]]"), "{snippet}");
        assert!(!snippet.contains('<'), "{snippet}");
    }

    #[tokio::test]
    async fn page_path_resolves_and_renders() {
        let pool = test_pool().await;
        let state = State(pool.clone());

        let (_, Json(rust)) = create_category(
            state.clone(),
            Json(CreateCategory {
                parent_id: None,
                name: "Rust".into(),
                slug: None,
            }),
        )
        .await
        .unwrap();

        let (_, Json(page)) = create_page(
            state.clone(),
            Json(CreatePage {
                category_id: rust.id,
                title: "Введение".into(),
                body_md: Some("# Привет".into()),
                slug: None,
                status: None,
            }),
        )
        .await
        .unwrap();

        assert_eq!(page.path, "/rust/vvedenie");
        assert!(page.html.contains("<h1"));

        let found = page_by_path(
            state.clone(),
            Query(PathQuery {
                path: "rust/vvedenie".into(),
            }),
        )
        .await
        .unwrap();

        assert_eq!(found.0.id, page.id);
    }
}
