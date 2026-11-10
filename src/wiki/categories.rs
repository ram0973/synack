use axum::{
    extract::{Path, Query, State},
    http::StatusCode, Json,
};
use serde::Serialize;
use sqlx::sqlite::SqlitePool;

use crate::error::AppError;

use crate::wiki::markdown::slugify;
use crate::wiki::models::{
    is_admin, Category, CategoryNode, CreateCategory, MoveCategory, PageSummary, PathQuery,
    UpdateCategory, Viewer,
};
use crate::wiki::repo::{
    build_tree, category_chain, find_category, is_descendant, next_position,
    resolve_category,
};
// Категории: дерево, создание, переименование, перенос, удаление.

// ---------- категории ----------

pub(crate) async fn tree(State(pool): State<SqlitePool>) -> Result<Json<Vec<CategoryNode>>, AppError> {
    let rows =
        sqlx::query_as::<_, Category>("SELECT id, parent_id, position, name, slug FROM categories")
            .fetch_all(&pool)
            .await?;

    Ok(Json(build_tree(rows)))
}

pub(crate) async fn create_category(
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

pub(crate) async fn update_category(
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

pub(crate) async fn move_category(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(body): Json<MoveCategory>,
) -> Result<Json<Category>, AppError> {
    let current = find_category(&pool, id).await?;

    let parent_id = match body.parent_id {
        Some(value) => value,
        None => current.parent_id,
    };

    if let Some(parent_id) = parent_id {
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

    let mut tx = pool.begin().await?;

    // Соседи в целевой группе, без самой перемещаемой категории, в текущем порядке.
    let siblings: Vec<i64> = sqlx::query_scalar(
        "SELECT id FROM categories WHERE parent_id IS ? AND id != ? ORDER BY position, id",
    )
    .bind(parent_id)
    .bind(id)
    .fetch_all(&mut *tx)
    .await?;

    // Куда вставить: по явно заданной позиции либо в конец группы.
    let insert_at = match body.position {
        Some(position) => position.clamp(0, siblings.len() as i64) as usize,
        None => siblings.len(),
    };

    let mut order = siblings;
    order.insert(insert_at, id);

    // Перенумеровываем всю группу подряд, чтобы не оставалось совпадающих position.
    for (index, category_id) in order.iter().enumerate() {
        sqlx::query("UPDATE categories SET parent_id = ?, position = ? WHERE id = ?")
            .bind(parent_id)
            .bind(index as i64)
            .bind(category_id)
            .execute(&mut *tx)
            .await?;
    }

    let row = sqlx::query_as::<_, Category>(
        "SELECT id, parent_id, position, name, slug FROM categories WHERE id = ?",
    )
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(Json(row))
}

pub(crate) async fn delete_category(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    // Проверка и удаление в одной транзакции: иначе между проверкой count()
    // и DELETE в категорию может успеть попасть страница из другого запроса.
    let mut tx = pool.begin().await?;

    let exists: Option<i64> = sqlx::query_scalar("SELECT 1 FROM categories WHERE id = ?")
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?;
    if exists.is_none() {
        return Err(AppError::not_found("category not found"));
    }

    let children: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM categories WHERE parent_id = ?")
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;
    if children > 0 {
        return Err(AppError::conflict("category has children"));
    }

    let pages: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM pages WHERE category_id = ?")
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;
    if pages > 0 {
        return Err(AppError::conflict("category has pages"));
    }

    sqlx::query("DELETE FROM categories WHERE id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Serialize)]
pub(crate) struct CategoryView {
    #[serde(flatten)]
    category: Category,
    path: String,
    children: Vec<Category>,
    pages: Vec<PageSummary>,
}

pub(crate) async fn category_by_path(
    State(pool): State<SqlitePool>,
    Query(query): Query<PathQuery>,
    viewer: Viewer,
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
         FROM pages
         WHERE category_id = ? AND (? OR status = 'published')
         ORDER BY title",
    )
    .bind(category.id)
    .bind(is_admin(&viewer))
    .fetch_all(&pool)
    .await?;

    Ok(Json(CategoryView {
        category,
        path,
        children,
        pages,
    }))
}
