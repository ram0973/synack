use axum::{
    extract::{Path, Query, State},
    http::StatusCode, Json,
};
use serde_json::{json, Value};
use sqlx::sqlite::SqlitePool;

use crate::error::AppError;

use crate::wiki::categories::category_by_path;
use crate::wiki::markdown::{make_snippet, render_markdown, slugify};
use crate::wiki::models::{
    is_admin, Category, CreatePage, ListPagesQuery, Page, PageOut, PageSummary, PathQuery,
    PreviewRequest, SearchHit, SearchQuery, UpdatePage, Viewer,
};
use crate::wiki::repo::{
    category_chain, find_page, normalize_status, page_path, to_page_out,
};
use crate::wiki::revisions::save_revision;
// Страницы: список, чтение, создание, правка, поиск, резолв по пути.

// ---------- страницы ----------



pub(crate) async fn search(
    State(pool): State<SqlitePool>,
    Query(query): Query<SearchQuery>,
    viewer: Viewer,
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
         WHERE pages_fts MATCH ? AND (? OR p.status = 'published')
         ORDER BY bm25(pages_fts, 5.0, 1.0)
         LIMIT 20",
    )
    .bind(match_query)
    .bind(is_admin(&viewer))
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



pub(crate) async fn list_pages(
    State(pool): State<SqlitePool>,
    Query(query): Query<ListPagesQuery>,
    viewer: Viewer,
) -> Result<Json<Vec<PageSummary>>, AppError> {
    let admin = is_admin(&viewer);

    let rows = match query.category {
        Some(category_id) => {
            sqlx::query_as::<_, PageSummary>(
                "SELECT id, category_id, slug, title, status, updated_at
                 FROM pages
                 WHERE category_id = ? AND (? OR status = 'published')
                 ORDER BY title",
            )
            .bind(category_id)
            .bind(admin)
            .fetch_all(&pool)
            .await?
        }
        None => {
            sqlx::query_as::<_, PageSummary>(
                "SELECT id, category_id, slug, title, status, updated_at
                 FROM pages WHERE (? OR status = 'published') ORDER BY title",
            )
            .bind(admin)
            .fetch_all(&pool)
            .await?
        }
    };

    Ok(Json(rows))
}

pub(crate) async fn get_page(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    viewer: Viewer,
) -> Result<Json<PageOut>, AppError> {
    let page = find_page(&pool, id).await?;

    if !is_admin(&viewer) && page.status != "published" {
        return Err(AppError::not_found("page not found"));
    }
    let chain = category_chain(&pool, page.category_id).await?;

    Ok(Json(to_page_out(page, &chain)))
}


/// Проверяет уникальность заголовка и slug внутри категории.
pub(crate) async fn ensure_unique_page(
    pool: &SqlitePool,
    category_id: i64,
    title: &str,
    slug: &str,
    except_id: Option<i64>,
) -> Result<(), AppError> {
    let rows: Vec<(i64, String, String)> =
        sqlx::query_as("SELECT id, title, slug FROM pages WHERE category_id = ?")
            .bind(category_id)
            .fetch_all(pool)
            .await?;

    let wanted = title.to_lowercase();

    for (id, other_title, other_slug) in rows {
        if Some(id) == except_id {
            continue;
        }

        if other_title.to_lowercase() == wanted {
            return Err(AppError::conflict(
                "Страница с таким заголовком уже есть в этой категории",
            ));
        }

        if other_slug == slug {
            return Err(AppError::conflict("Такой slug уже занят в этой категории"));
        }
    }

    Ok(())
}

pub(crate) async fn create_page(
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

    ensure_unique_page(&pool, body.category_id, &title, &slug, None).await?;

    let mut tx = pool.begin().await?;

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
    .fetch_one(&mut *tx)
    .await?;

    save_revision(&mut tx, &page).await?;
    tx.commit().await?;

    Ok((StatusCode::CREATED, Json(to_page_out(page, &chain))))
}

pub(crate) async fn update_page(
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

    ensure_unique_page(&pool, category_id, &title, &slug, Some(id)).await?;

    let mut tx = pool.begin().await?;

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
    .fetch_one(&mut *tx)
    .await?;

    save_revision(&mut tx, &page).await?;
    tx.commit().await?;

    let chain = category_chain(&pool, category_id).await?;

    Ok(Json(to_page_out(page, &chain)))
}

pub(crate) async fn delete_page(
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

pub(crate) async fn page_by_path(
    State(pool): State<SqlitePool>,
    Query(query): Query<PathQuery>,
    viewer: Viewer,
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
         FROM pages
         WHERE category_id = ? AND slug = ? AND (? OR status = 'published')",
    )
    .bind(category_id)
    .bind(page_slug[0])
    .bind(is_admin(&viewer))
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::not_found("page not found"))?;

    let chain = category_chain(&pool, category_id).await?;

    Ok(Json(to_page_out(page, &chain)))
}



pub(crate) async fn preview(Json(body): Json<PreviewRequest>) -> Json<Value> {
    Json(json!({ "html": render_markdown(&body.body_md) }))
}


pub(crate) async fn resolve_path(
    State(pool): State<SqlitePool>,
    Query(query): Query<PathQuery>,
    viewer: Viewer,
) -> Result<Json<Value>, AppError> {
    let path = query.path.trim().trim_matches('/').to_string();

    match page_by_path(
        State(pool.clone()),
        Query(PathQuery { path: path.clone() }),
        viewer,
    )
    .await
    {
        Ok(Json(page)) => return Ok(Json(json!({ "kind": "page", "page": page }))),
        Err(error)
            if error.status == StatusCode::NOT_FOUND || error.status == StatusCode::BAD_REQUEST => {
        }
        Err(error) => return Err(error),
    }

    match category_by_path(State(pool), Query(PathQuery { path }), viewer).await {
        Ok(Json(category)) => Ok(Json(json!({ "kind": "category", "category": category }))),
        Err(error) if error.status == StatusCode::NOT_FOUND => Err(error),
        Err(error) => Err(error),
    }
}
