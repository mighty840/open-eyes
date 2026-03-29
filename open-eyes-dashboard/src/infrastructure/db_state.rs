use open_eyes_core::DbPool;

/// Wrapper for DB pool to use as Axum extension / Dioxus server context.
#[derive(Clone)]
pub struct DbState(pub DbPool);

impl<S> axum::extract::FromRequestParts<S> for DbState
where
    S: Send + Sync,
{
    type Rejection = axum::http::StatusCode;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<DbState>()
            .cloned()
            .ok_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
    }
}
