use argon2::password_hash::{
    rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
};
use argon2::Argon2;
use axum::{
    extract::{Request, State},
    http::{header, HeaderValue, Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use rand_core::RngCore;
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::sqlite::SqlitePool;

use crate::error::AppError;

/// Проставляется middleware: был ли запрос с валидной сессией.
#[derive(Clone, Copy)]
pub struct Authenticated(pub bool);

#[derive(Debug, Deserialize)]
pub struct ChangePassword {
    pub current: String,
    pub next: String,
}

pub async fn change_password(
    State(pool): State<SqlitePool>,
    headers: axum::http::HeaderMap,
    Json(body): Json<ChangePassword>,
) -> Result<Json<Value>, AppError> {
    let hash = stored_hash(&pool)
        .await?
        .ok_or_else(|| AppError::internal("password is not initialised"))?;

    if !verify_password(&body.current, &hash) {
        return Err(AppError::new(StatusCode::UNAUTHORIZED, "wrong password"));
    }

    if body.next.chars().count() < 8 {
        return Err(AppError::bad_request(
            "password must be at least 8 characters",
        ));
    }

    let next_hash = hash_password(&body.next)?;
    set_hash(&pool, &next_hash).await?;
    save_default_flag(&pool, body.next == DEFAULT_PASSWORD).await?;

    match session_token_from(&headers) {
        Some(token) => {
            sqlx::query("DELETE FROM sessions WHERE token != ?")
                .bind(token)
                .execute(&pool)
                .await?;
        }
        None => {
            sqlx::query("DELETE FROM sessions").execute(&pool).await?;
        }
    }

    Ok(Json(json!({ "ok": true })))
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub password: String,
}

const DEFAULT_PASSWORD: &str = "admin";

pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);

    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|error| AppError::internal(error.to_string()))
}

fn verify_password(password: &str, hash: &str) -> bool {
    match PasswordHash::new(hash) {
        Ok(parsed) => Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok(),
        Err(_) => false,
    }
}

fn random_token() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn session_token(request: &Request) -> Option<String> {
    session_token_from(request.headers())
}

fn session_token_from(headers: &axum::http::HeaderMap) -> Option<String> {
    let raw = headers.get(header::COOKIE)?.to_str().ok()?;

    for part in raw.split(';') {
        let part = part.trim();
        if let Some(value) = part.strip_prefix("session=") {
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }

    None
}

async fn session_valid(pool: &SqlitePool, token: &str) -> Result<bool, AppError> {
    let row: Option<i64> = sqlx::query_scalar(
        "SELECT 1 FROM sessions WHERE token = ? AND expires_at > datetime('now')",
    )
    .bind(token)
    .fetch_optional(pool)
    .await?;

    Ok(row.is_some())
}

pub async fn stored_hash(pool: &SqlitePool) -> Result<Option<String>, AppError> {
    let value: Option<String> =
        sqlx::query_scalar("SELECT value FROM settings WHERE key = 'password_hash'")
            .fetch_optional(pool)
            .await?;

    Ok(value)
}

pub async fn set_hash(pool: &SqlitePool, hash: &str) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO settings (key, value) VALUES ('password_hash', ?)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = datetime('now')",
    )
    .bind(hash)
    .execute(pool)
    .await?;

    Ok(())
}

/// Первый запуск: хеш из WIKI_PASSWORD_HASH либо стандартный admin.
pub async fn bootstrap_hash(pool: &SqlitePool) -> Result<(), AppError> {
    if stored_hash(pool).await?.is_none() {
        let hash = match std::env::var("WIKI_PASSWORD_HASH") {
            Ok(value) if !value.trim().is_empty() => value.trim().to_string(),
            _ => hash_password(DEFAULT_PASSWORD)?,
        };

        set_hash(pool, &hash).await?;

        // хеш только что создан — флаг «стандартный пароль» пересчитаем ниже
        sqlx::query("DELETE FROM settings WHERE key = 'password_default'")
            .execute(pool)
            .await?;
    }

    if default_flag(pool).await?.is_none() {
        let hash = stored_hash(pool).await?.unwrap_or_default();
        let is_default = verify_password(DEFAULT_PASSWORD, &hash);

        save_default_flag(pool, is_default).await?;

        if is_default {
            tracing::warn!("default admin password is in effect, change it at /admin/password");
        }
    }

    Ok(())
}

async fn default_flag(pool: &SqlitePool) -> Result<Option<String>, AppError> {
    Ok(
        sqlx::query_scalar("SELECT value FROM settings WHERE key = 'password_default'")
            .fetch_optional(pool)
            .await?,
    )
}

async fn save_default_flag(pool: &SqlitePool, is_default: bool) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO settings (key, value) VALUES ('password_default', ?)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = datetime('now')",
    )
    .bind(if is_default { "1" } else { "0" })
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn is_default_password(pool: &SqlitePool) -> Result<bool, AppError> {
    Ok(default_flag(pool).await?.as_deref() == Some("1"))
}

pub async fn login(
    State(pool): State<SqlitePool>,
    headers: axum::http::HeaderMap,
    Json(body): Json<LoginRequest>,
) -> Result<Response, AppError> {
    let hash = stored_hash(&pool)
        .await?
        .ok_or_else(|| AppError::internal("password is not initialised"))?;

    if !verify_password(&body.password, &hash) {
        return Err(AppError::new(StatusCode::UNAUTHORIZED, "wrong password"));
    }

    let token = random_token();

    sqlx::query("INSERT INTO sessions (token, expires_at) VALUES (?, datetime('now', '+30 days'))")
        .bind(&token)
        .execute(&pool)
        .await?;

    let secure = headers
        .get("x-forwarded-proto")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.eq_ignore_ascii_case("https"))
        .unwrap_or(false);

    let cookie = format!(
        "session={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age=2592000{}",
        if secure { "; Secure" } else { "" }
    );

    let mut response = Json(json!({ "ok": true })).into_response();
    response.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie).map_err(|error| AppError::internal(error.to_string()))?,
    );

    Ok(response)
}

pub async fn logout(
    State(pool): State<SqlitePool>,
    request: Request,
) -> Result<Response, AppError> {
    if let Some(token) = session_token(&request) {
        sqlx::query("DELETE FROM sessions WHERE token = ?")
            .bind(token)
            .execute(&pool)
            .await?;
    }

    let mut response = Json(json!({ "ok": true })).into_response();
    response.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_static("session=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0"),
    );

    Ok(response)
}

pub async fn me(State(pool): State<SqlitePool>, request: Request) -> Result<Json<Value>, AppError> {
    let authenticated = match session_token(&request) {
        Some(token) => session_valid(&pool, &token).await?,
        None => false,
    };

    let default_password = is_default_password(&pool).await?;

    Ok(Json(json!({
        "authenticated": authenticated,
        "default_password": default_password
    })))
}

/// Пропускает чтение всем, а любые изменения требует с валидной сессией.
pub async fn auth_guard(
    State(pool): State<SqlitePool>,
    mut request: Request,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let is_login = request.uri().path() == "/api/auth/login";

    let authenticated = match session_token(&request) {
        Some(token) => session_valid(&pool, &token).await.unwrap_or(false),
        None => false,
    };

    request
        .extensions_mut()
        .insert(Authenticated(authenticated));

    if is_login || matches!(method, Method::GET | Method::HEAD | Method::OPTIONS) || authenticated {
        return next.run(request).await;
    }

    (
        StatusCode::UNAUTHORIZED,
        Json(json!({ "error": "unauthorized" })),
    )
        .into_response()
}
