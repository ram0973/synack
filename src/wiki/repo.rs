use std::collections::HashMap;
use sqlx::sqlite::SqlitePool;

use crate::error::AppError;

use crate::wiki::markdown::{render_markdown, strip_duplicate_title};
use crate::wiki::models::{Category, CategoryNode, Page, PageOut};
// Низкоуровневые запросы, общие для нескольких модулей.

pub(crate) async fn find_category(pool: &SqlitePool, id: i64) -> Result<Category, AppError> {
    sqlx::query_as::<_, Category>(
        "SELECT id, parent_id, position, name, slug FROM categories WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::not_found("category not found"))
}

pub(crate) async fn find_page(pool: &SqlitePool, id: i64) -> Result<Page, AppError> {
    sqlx::query_as::<_, Page>(
        "SELECT id, category_id, slug, title, body_md, status, created_at, updated_at
         FROM pages WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::not_found("page not found"))
}

pub(crate) async fn resolve_category(pool: &SqlitePool, slugs: &[&str]) -> Result<Category, AppError> {
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

pub(crate) async fn category_chain(pool: &SqlitePool, id: i64) -> Result<Vec<Category>, AppError> {
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

pub(crate) fn to_page_out(page: Page, chain: &[Category]) -> PageOut {
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

pub(crate) fn page_path(chain: &[Category], slug: &str) -> String {
    let mut parts: Vec<&str> = chain
        .iter()
        .map(|category| category.slug.as_str())
        .collect();
    parts.push(slug);
    format!("/{}", parts.join("/"))
}

pub(crate) fn normalize_status(status: Option<&str>) -> Result<String, AppError> {
    match status.unwrap_or("draft") {
        "draft" => Ok("draft".to_string()),
        "published" => Ok("published".to_string()),
        other => Err(AppError::bad_request(format!("unknown status: {other}"))),
    }
}

/// Убирает первый заголовок H1, если он повторяет заголовок страницы.
pub(crate) async fn next_position(pool: &SqlitePool, parent_id: Option<i64>) -> Result<i64, AppError> {
    let next: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(position), -1) + 1 FROM categories WHERE parent_id IS ?",
    )
    .bind(parent_id)
    .fetch_one(pool)
    .await?;

    Ok(next)
}

pub(crate) async fn is_descendant(pool: &SqlitePool, ancestor: i64, candidate: i64) -> Result<bool, AppError> {
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

pub(crate) fn build_tree(rows: Vec<Category>) -> Vec<CategoryNode> {
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
