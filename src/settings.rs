use axum::{extract::State, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::sqlite::SqlitePool;

use crate::error::AppError;

#[derive(Debug, Deserialize)]
pub struct UpdateSettings {
    pub author_name: Option<String>,
    pub author_avatar: Option<String>,
}

async fn read_all(pool: &SqlitePool) -> Result<(String, String), AppError> {
    let rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT key, value FROM settings WHERE key IN ('author_name', 'author_avatar')",
    )
    .fetch_all(pool)
    .await?;

    let mut name = String::new();
    let mut avatar = String::new();

    for (key, value) in rows {
        match key.as_str() {
            "author_name" => name = value,
            "author_avatar" => avatar = value,
            _ => {}
        }
    }

    Ok((name, avatar))
}

pub async fn get_settings(State(pool): State<SqlitePool>) -> Result<Json<Value>, AppError> {
    let (name, avatar) = read_all(&pool).await?;

    Ok(Json(json!({
        "author_name": name,
        "author_avatar": avatar
    })))
}

async fn save(pool: &SqlitePool, key: &str, value: &str) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO settings (key, value) VALUES (?, ?)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = datetime('now')",
    )
    .bind(key)
    .bind(value)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn update_settings(
    State(pool): State<SqlitePool>,
    Json(body): Json<UpdateSettings>,
) -> Result<Json<Value>, AppError> {
    if let Some(name) = body.author_name {
        let name = name.trim().to_string();

        if name.chars().count() > 80 {
            return Err(AppError::bad_request("name is too long"));
        }

        save(&pool, "author_name", &name).await?;
    }

    if let Some(avatar) = body.author_avatar {
        let avatar = avatar.trim().to_string();

        if !avatar.is_empty()
            && !avatar.starts_with("/media/")
            && !avatar.starts_with("https://")
            && !avatar.starts_with("http://")
        {
            return Err(AppError::bad_request("avatar must be a /media/ url"));
        }

        save(&pool, "author_avatar", &avatar).await?;
    }

    let (name, avatar) = read_all(&pool).await?;

    Ok(Json(json!({
        "author_name": name,
        "author_avatar": avatar
    })))
}
