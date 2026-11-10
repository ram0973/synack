use axum::Extension;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::auth::Authenticated;



// Модели БД и DTO запросов/ответов.


/// Зритель запроса: `Some`, если сессия валидна.
pub(crate) type Viewer = Option<Extension<Authenticated>>;

/// Пускает ли зритель черновики.
pub(crate) fn is_admin(viewer: &Viewer) -> bool {
    viewer
        .as_ref()
        .map(|Extension(auth)| auth.0)
        .unwrap_or(false)
}

// ---------- модели ----------

#[derive(Debug, Clone, Serialize, FromRow)]
pub(crate) struct Category {
    pub(crate) id: i64,
    pub(crate) parent_id: Option<i64>,
    pub(crate) position: i64,
    pub(crate) name: String,
    pub(crate) slug: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct CategoryNode {
    #[serde(flatten)]
    pub(crate) category: Category,
    pub(crate) children: Vec<CategoryNode>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub(crate) struct Page {
    pub(crate) id: i64,
    pub(crate) category_id: i64,
    pub(crate) slug: String,
    pub(crate) title: String,
    pub(crate) body_md: String,
    pub(crate) status: String,
    pub(crate) created_at: String,
    pub(crate) updated_at: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct PageOut {
    pub(crate) id: i64,
    pub(crate) category_id: i64,
    pub(crate) slug: String,
    pub(crate) title: String,
    pub(crate) body_md: String,
    pub(crate) status: String,
    pub(crate) html: String,
    pub(crate) path: String,
    pub(crate) created_at: String,
    pub(crate) updated_at: String,
}

#[derive(Debug, Serialize, FromRow)]
pub(crate) struct PageSummary {
    pub(crate) id: i64,
    pub(crate) category_id: i64,
    pub(crate) slug: String,
    pub(crate) title: String,
    pub(crate) status: String,
    pub(crate) updated_at: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CreateCategory {
    pub(crate) parent_id: Option<i64>,
    pub(crate) name: String,
    pub(crate) slug: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct UpdateCategory {
    pub(crate) name: Option<String>,
    pub(crate) slug: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct MoveCategory {
    #[serde(default, deserialize_with = "deserialize_present")]
    pub(crate) parent_id: Option<Option<i64>>,
    pub(crate) position: Option<i64>,
}

/// Поле отсутствует → None; поле `null` → Some(None); число → Some(Some(id)).
pub(crate) fn deserialize_present<'de, D>(deserializer: D) -> Result<Option<Option<i64>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<i64>::deserialize(deserializer).map(Some)
}

#[derive(Debug, Deserialize)]
pub(crate) struct CreatePage {
    pub(crate) category_id: i64,
    pub(crate) title: String,
    pub(crate) body_md: Option<String>,
    pub(crate) slug: Option<String>,
    pub(crate) status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct UpdatePage {
    pub(crate) category_id: Option<i64>,
    pub(crate) title: Option<String>,
    pub(crate) body_md: Option<String>,
    pub(crate) slug: Option<String>,
    pub(crate) status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ListPagesQuery {
    pub(crate) category: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct PathQuery {
    pub(crate) path: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PreviewRequest {
    pub(crate) body_md: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SearchQuery {
    pub(crate) q: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct SearchHit {
    pub(crate) id: i64,
    pub(crate) title: String,
    pub(crate) path: String,
    pub(crate) status: String,
    pub(crate) snippet: String,
}

#[derive(Debug, Serialize, FromRow)]
pub(crate) struct Revision {
    pub(crate) id: i64,
    pub(crate) page_id: i64,
    pub(crate) title: String,
    pub(crate) body_md: String,
    pub(crate) created_at: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct RevisionSummary {
    pub(crate) id: i64,
    pub(crate) title: String,
    pub(crate) created_at: String,
    pub(crate) size: i64,
    pub(crate) preview: String,
}
