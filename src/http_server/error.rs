use axum::response::IntoResponse;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid request: missing code")]
    InvalidRequest,

    #[error("Invalid response from Discord: {0}")]
    InvalidResponse(serde_json::Value),

    #[error("Could not find a Twitch account linked to your Discord account. Please link your Twitch account to Discord first.")]
    TwitterNotLinked,

    #[error("An error occurred while communicating with Discord: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("An error occurred while communicating with the database: {0}")]
    Sqlx(#[from] sqlx::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::http::Response<axum::body::Body> {
        let status = match &self {
            Error::InvalidRequest => axum::http::StatusCode::BAD_REQUEST,
            Error::InvalidResponse(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Error::TwitterNotLinked => axum::http::StatusCode::NOT_FOUND,
            Error::Reqwest(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Error::Sqlx(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = format!("{}", self);
        let response = axum::http::Response::builder()
            .status(status)
            .body(axum::body::Body::from(body))
            .unwrap();
        response
    }
}
