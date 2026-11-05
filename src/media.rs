use std::path::PathBuf;

use axum::{
    body::Body,
    extract::{Multipart, Path, State},
    http::{header, HeaderValue},
    response::Response,
    Extension, Json,
};
use rand_core::RngCore;
use serde::Serialize;
use sqlx::sqlite::SqlitePool;
use tokio::io::AsyncWriteExt;

use crate::error::AppError;

const MAX_IMAGE_BYTES: usize = 10 * 1024 * 1024;
const MAX_VIDEO_BYTES: usize = 200 * 1024 * 1024;

/// Каталог, куда складываются файлы (по умолчанию ./media).
#[derive(Clone)]
pub struct MediaDir(pub PathBuf);

#[derive(Debug, Serialize)]
pub struct MediaOut {
    pub url: String,
    pub name: String,
    pub size: i64,
}

fn extension_for(mime: &str) -> Option<&'static str> {
    match mime {
        "image/png" => Some("png"),
        "image/jpeg" => Some("jpg"),
        "image/gif" => Some("gif"),
        "image/webp" => Some("webp"),
        "image/avif" => Some("avif"),
        "video/mp4" => Some("mp4"),
        "video/webm" => Some("webm"),
        _ => None,
    }
}

fn detect_mime(filename: Option<&str>, content_type: Option<&str>) -> Option<String> {
    if let Some(value) = content_type {
        let essence = value.split(';').next().unwrap_or(value).trim();
        if extension_for(essence).is_some() {
            return Some(essence.to_string());
        }
    }

    let name = filename?;
    let guessed = mime_guess::from_path(name).first()?;

    if extension_for(guessed.essence_str()).is_some() {
        Some(guessed.essence_str().to_string())
    } else {
        None
    }
}

fn random_name(extension: &str) -> String {
    let mut bytes = [0u8; 16];
    rand_core::OsRng.fill_bytes(&mut bytes);
    let hex: String = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
    format!("{hex}.{extension}")
}

pub async fn upload(
    State(pool): State<SqlitePool>,
    Extension(dir): Extension<MediaDir>,
    mut multipart: Multipart,
) -> Result<Json<MediaOut>, AppError> {
    while let Some(mut field) = multipart
        .next_field()
        .await
        .map_err(|error| AppError::bad_request(error.to_string()))?
    {
        let filename = field.file_name().map(str::to_string);
        let content_type = field.content_type().map(|value| value.to_string());

        let mime = detect_mime(filename.as_deref(), content_type.as_deref())
            .ok_or_else(|| AppError::bad_request("unsupported file type"))?;

        let limit = if mime.starts_with("video/") {
            MAX_VIDEO_BYTES
        } else {
            MAX_IMAGE_BYTES
        };

        let (year, month): (String, String) =
            sqlx::query_as("SELECT strftime('%Y', 'now'), strftime('%m', 'now')")
                .fetch_one(&pool)
                .await?;

        let folder = dir.0.join(&year).join(&month);
        tokio::fs::create_dir_all(&folder)
            .await
            .map_err(|error| AppError::internal(error.to_string()))?;

        let extension = extension_for(&mime).expect("checked by detect_mime");
        let name = random_name(extension);
        let relative = format!("{year}/{month}/{name}");
        let path = dir.0.join(&relative);

        let mut file = tokio::fs::File::create(&path)
            .await
            .map_err(|error| AppError::internal(error.to_string()))?;

        let mut size: u64 = 0;

        // читаем поле кусками, чтобы не держать файл в памяти и успеть остановиться на лимите
        loop {
            let chunk = match field.chunk().await {
                Ok(Some(chunk)) => chunk,
                Ok(None) => break,
                Err(error) => {
                    drop(file);
                    let _ = tokio::fs::remove_file(&path).await;
                    return Err(AppError::bad_request(error.to_string()));
                }
            };

            size += chunk.len() as u64;

            if size > limit as u64 {
                drop(file);
                let _ = tokio::fs::remove_file(&path).await;
                return Err(AppError::bad_request("file is too large"));
            }

            if let Err(error) = file.write_all(&chunk).await {
                drop(file);
                let _ = tokio::fs::remove_file(&path).await;
                return Err(AppError::internal(error.to_string()));
            }
        }

        if let Err(error) = file.flush().await {
            drop(file);
            let _ = tokio::fs::remove_file(&path).await;
            return Err(AppError::internal(error.to_string()));
        }

        drop(file);

        if size == 0 {
            let _ = tokio::fs::remove_file(&path).await;
            continue;
        }

        sqlx::query("INSERT INTO media (filename, mime, size) VALUES (?, ?, ?)")
            .bind(&relative)
            .bind(&mime)
            .bind(size as i64)
            .execute(&pool)
            .await?;

        return Ok(Json(MediaOut {
            url: format!("/media/{relative}"),
            name: relative,
            size: size as i64,
        }));
    }

    Err(AppError::bad_request("file is required"))
}

pub async fn serve(
    Extension(dir): Extension<MediaDir>,
    Path(path): Path<String>,
) -> Result<Response, AppError> {
    let relative = path.trim_start_matches('/');

    if relative.is_empty() || relative.contains('\\') {
        return Err(AppError::not_found("not found"));
    }

    // Резолвим реальный путь и проверяем, что он не вышел за пределы media_dir
    // (защита от обхода каталога через `..`, в том числе закодированного).
    let root = tokio::fs::canonicalize(&dir.0)
        .await
        .map_err(|error| AppError::internal(error.to_string()))?;

    let full = tokio::fs::canonicalize(dir.0.join(relative))
        .await
        .map_err(|_| AppError::not_found("not found"))?;

    if !full.starts_with(&root) {
        return Err(AppError::not_found("not found"));
    }

    let bytes = tokio::fs::read(&full)
        .await
        .map_err(|_| AppError::not_found("not found"))?;

    let mime = mime_guess::from_path(&full).first_or_octet_stream();

    let mut response = Response::new(Body::from(bytes));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(mime.as_ref())
            .map_err(|error| AppError::internal(error.to_string()))?,
    );
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=31536000, immutable"),
    );

    Ok(response)
}
