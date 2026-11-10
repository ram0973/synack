use axum::{
    routing::{get, patch, post},
    Router,
};
use sqlx::sqlite::SqlitePool;

use crate::wiki::categories::{
    category_by_path, create_category, delete_category, move_category, tree, update_category,
};
use crate::wiki::pages::{
    create_page, delete_page, get_page, list_pages, page_by_path, preview, resolve_path, search,
    update_page,
};
use crate::wiki::revisions::{get_revision, list_revisions, restore_revision};

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
