use open_eyes_core::DuckDbPool;

/// Wrapper for DuckDB pool to use as Axum extension / Dioxus server context.
#[derive(Clone)]
pub struct DuckDbState(pub DuckDbPool);

impl<S> axum::extract::FromRequestParts<S> for DuckDbState
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
            .get::<DuckDbState>()
            .cloned()
            .ok_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
    }
}
