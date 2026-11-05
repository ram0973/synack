use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

// ---------- ошибки ----------

#[derive(Debug)]
pub(crate) struct AppError {
    pub(crate) status: StatusCode,
    message: String,
}

impl AppError {
    pub(crate) fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }

    pub(crate) fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, message)
    }

    pub(crate) fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, message)
    }

    pub(crate) fn conflict(message: impl Into<String>) -> Self {
        Self::new(StatusCode::CONFLICT, message)
    }

    pub(crate) fn internal(message: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, message)
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.message)
    }
}

impl std::error::Error for AppError {}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        if self.status.is_server_error() {
            tracing::error!("internal error: {}", self.message);

            return (
                self.status,
                Json(json!({ "error": "internal server error" })),
            )
                .into_response();
        }

        (self.status, Json(json!({ "error": self.message }))).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        if let sqlx::Error::Database(db) = &err {
            if db.is_unique_violation() {
                let message = db.message();
                if message.contains("idx_pages_title_unique") {
                    return AppError::conflict(
                        "Страница с таким заголовком уже есть в этой категории",
                    );
                }
                return AppError::conflict("Такой slug уже занят в этой категории");
            }
        }
        AppError::new(StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
    }
}
