use axum::{
    extract::{Path, State},
    http::StatusCode, Json,
};
use sqlx::sqlite::SqlitePool;

use crate::error::AppError;

use crate::wiki::markdown::{html_to_text, render_markdown};
use crate::wiki::models::{is_admin, Page, PageOut, Revision, RevisionSummary, Viewer};
use crate::wiki::repo::{category_chain, find_page, to_page_out};
// История версий страницы.

// ---------- вспомогательное ----------



pub(crate) const MAX_REVISIONS_PER_PAGE: i64 = 50;

pub(crate) async fn save_revision(executor: &mut sqlx::SqliteConnection, page: &Page) -> Result<(), AppError> {
    sqlx::query("INSERT INTO revisions (page_id, title, body_md) VALUES (?, ?, ?)")
        .bind(page.id)
        .bind(&page.title)
        .bind(&page.body_md)
        .execute(&mut *executor)
        .await?;

    // Оставляем только последние MAX_REVISIONS_PER_PAGE ревизий страницы.
    sqlx::query(
        "DELETE FROM revisions
         WHERE page_id = ?
           AND id NOT IN (
               SELECT id FROM revisions WHERE page_id = ? ORDER BY id DESC LIMIT ?
           )",
    )
    .bind(page.id)
    .bind(page.id)
    .bind(MAX_REVISIONS_PER_PAGE)
    .execute(&mut *executor)
    .await?;

    Ok(())
}

pub(crate) async fn list_revisions(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    viewer: Viewer,
) -> Result<Json<Vec<RevisionSummary>>, AppError> {
    if !is_admin(&viewer) {
        return Err(AppError::new(StatusCode::UNAUTHORIZED, "unauthorized"));
    }

    find_page(&pool, id).await?;

    let rows = sqlx::query_as::<_, (i64, String, String, i64, String)>(
        "SELECT id, title, created_at, length(body_md), body_md
         FROM revisions WHERE page_id = ? ORDER BY id DESC LIMIT ?",
    )
    .bind(id)
    .bind(MAX_REVISIONS_PER_PAGE)
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

pub(crate) async fn get_revision(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    viewer: Viewer,
) -> Result<Json<Revision>, AppError> {
    if !is_admin(&viewer) {
        return Err(AppError::new(StatusCode::UNAUTHORIZED, "unauthorized"));
    }

    let revision = sqlx::query_as::<_, Revision>(
        "SELECT id, page_id, title, body_md, created_at FROM revisions WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::not_found("revision not found"))?;

    Ok(Json(revision))
}

pub(crate) async fn restore_revision(
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

    let mut tx = pool.begin().await?;

    let page = sqlx::query_as::<_, Page>(
        "UPDATE pages SET title = ?, body_md = ?, updated_at = datetime('now')
         WHERE id = ?
         RETURNING id, category_id, slug, title, body_md, status, created_at, updated_at",
    )
    .bind(&revision.title)
    .bind(&revision.body_md)
    .bind(revision.page_id)
    .fetch_one(&mut *tx)
    .await?;

    save_revision(&mut tx, &page).await?;
    tx.commit().await?;

    let chain = category_chain(&pool, page.category_id).await?;

    Ok(Json(to_page_out(page, &chain)))
}
