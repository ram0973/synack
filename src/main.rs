use std::net::SocketAddr;
use std::path::PathBuf;
use std::str::FromStr;

use axum::{
    body::Body,
    extract::{DefaultBodyLimit, State},
    http::{header, HeaderValue, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use rust_embed::Embed;
use serde_json::{json, Value};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions};

mod auth;
mod error;
mod media;
mod wiki;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if let Some(hash) = hash_password_command()? {
        println!("{hash}");
        return Ok(());
    }

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "wiki=info".into()),
        )
        .init();

    let db_path = std::env::var("WIKI_DB").unwrap_or_else(|_| "wiki.db".into());
    let pool = connect(&format!("sqlite://{db_path}")).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    auth::bootstrap_hash(&pool).await?;
    tracing::info!("migrations applied");

    let media_dir =
        PathBuf::from(std::env::var("WIKI_MEDIA").unwrap_or_else(|_| "media".to_string()));
    tokio::fs::create_dir_all(&media_dir).await?;

    let app = Router::new()
        .route("/health", get(health))
        .route(
            "/api/media",
            post(media::upload).layer(DefaultBodyLimit::max(210 * 1024 * 1024)),
        )
        .route("/media/{*path}", get(media::serve))
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/password", post(auth::change_password))
        .route("/api/auth/logout", post(auth::logout))
        .route("/api/auth/me", get(auth::me))
        .nest("/api/wiki", wiki::router())
        .layer(axum::middleware::from_fn_with_state(
            pool.clone(),
            auth::auth_guard,
        ))
        .layer(axum::Extension(media::MediaDir(media_dir)))
        .fallback(static_handler)
        .with_state(pool);

    let addr: SocketAddr = std::env::var("WIKI_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:3000".into())
        .parse()?;

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("listening on http://{addr}");
    axum::serve(listener, app).await?;

    Ok(())
}

async fn connect(url: &str) -> anyhow::Result<SqlitePool> {
    let options = SqliteConnectOptions::from_str(url)?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .foreign_keys(true);

    Ok(SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?)
}

#[derive(Embed)]
#[folder = "dashboard/build/"]
struct Assets;

/// Отдаёт собранный SvelteKit: файл, если он есть, иначе index.html (SPA-роутинг).
async fn static_handler(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');

    if path.starts_with("api/") {
        return (StatusCode::NOT_FOUND, Json(json!({ "error": "not found" }))).into_response();
    }

    let candidate = if path.is_empty() { "index.html" } else { path };

    match Assets::get(candidate) {
        Some(content) => asset_response(candidate, content.data.into_owned()),
        None => match Assets::get("index.html") {
            Some(content) => asset_response("index.html", content.data.into_owned()),
            None => (StatusCode::NOT_FOUND, "dashboard is not built").into_response(),
        },
    }
}

fn asset_response(path: &str, bytes: Vec<u8>) -> Response {
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    let mut response = Response::new(Body::from(bytes));

    if let Ok(value) = HeaderValue::from_str(mime.as_ref()) {
        response.headers_mut().insert(header::CONTENT_TYPE, value);
    }

    if path.starts_with("_app/immutable/") {
        response.headers_mut().insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static("public, max-age=31536000, immutable"),
        );
    }

    response
}

/// `wiki --hash-password` читает пароль из stdin и печатает argon2id-хеш.
fn hash_password_command() -> anyhow::Result<Option<String>> {
    let requested = std::env::args().skip(1).any(|arg| arg == "--hash-password");

    if !requested {
        return Ok(None);
    }

    let mut input = String::new();
    std::io::Read::read_to_string(&mut std::io::stdin(), &mut input)?;

    let password = input.trim_end_matches(['\n', '\r']);

    if password.is_empty() {
        anyhow::bail!("password is empty");
    }

    Ok(Some(auth::hash_password(password)?))
}

async fn health(State(pool): State<SqlitePool>) -> Json<Value> {
    let ok = sqlx::query_scalar::<_, i64>("SELECT 1")
        .fetch_one(&pool)
        .await
        .map(|v| v == 1)
        .unwrap_or(false);

    Json(json!({ "ok": ok, "version": env!("CARGO_PKG_VERSION") }))
}
