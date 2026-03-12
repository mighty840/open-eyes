use std::sync::Arc;

use open_eyes_core::LlmClient;

/// Wrapper for LLM client to use as Axum extension / Dioxus server context.
#[derive(Clone)]
pub struct LlmState(pub Arc<LlmClient>);

impl<S> axum::extract::FromRequestParts<S> for LlmState
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
            .get::<LlmState>()
            .cloned()
            .ok_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
    }
}
