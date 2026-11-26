use axum::{
    extract::{rejection::JsonRejection, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use serde::Serialize;
use uuid::Uuid;

use super::{feedback::FeedbackSubmission, feedback_store::FeedbackStore};

pub fn routes(store: FeedbackStore) -> Router {
    Router::new()
        .route("/feedback", post(submit))
        .with_state(store)
}

async fn submit(
    State(store): State<FeedbackStore>,
    headers: HeaderMap,
    payload: Result<Json<FeedbackSubmission>, JsonRejection>,
) -> Result<(StatusCode, Json<FeedbackReceipt>), ApiError> {
    let content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next())
        .map(str::trim);
    if content_type != Some("application/json") {
        return Err(ApiError::unsupported_media_type());
    }
    let Json(submission) = payload.map_err(|error| {
        if error.status() == StatusCode::PAYLOAD_TOO_LARGE {
            ApiError {
                status: StatusCode::PAYLOAD_TOO_LARGE,
                code: "payload_too_large",
                message: "Request body must be 16 KiB or smaller.",
            }
        } else {
            ApiError::invalid_json()
        }
    })?;
    let feedback = submission.validate().map_err(ApiError::validation)?;
    let record = store.submit(feedback).await.map_err(|_| ApiError {
        status: StatusCode::SERVICE_UNAVAILABLE,
        code: "storage_unavailable",
        message: "Feedback could not be saved. Please try again later.",
    })?;
    Ok((StatusCode::CREATED, Json(FeedbackReceipt { id: record.id })))
}

#[derive(Serialize)]
struct FeedbackReceipt {
    id: Uuid,
}

#[derive(Serialize)]
struct ErrorBody {
    error: &'static str,
    message: &'static str,
}

struct ApiError {
    status: StatusCode,
    code: &'static str,
    message: &'static str,
}

impl ApiError {
    fn validation(error: super::feedback::ValidationError) -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            code: "validation_failed",
            message: error.message(),
        }
    }

    fn invalid_json() -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: "invalid_json",
            message: "request body must be valid JSON",
        }
    }

    fn unsupported_media_type() -> Self {
        Self {
            status: StatusCode::UNSUPPORTED_MEDIA_TYPE,
            code: "unsupported_media_type",
            message: "content-type must be application/json",
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorBody {
                error: self.code,
                message: self.message,
            }),
        )
            .into_response()
    }
}
