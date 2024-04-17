use axum::http::{header, HeaderName};
use tower_http::trace::{MakeSpan as TowerMakeSpan, OnResponse as TowerOnResponse};
use tracing::field;

#[derive(Default, Clone)]
pub struct MakeSpan;

#[derive(Default, Clone)]
pub struct OnResponse;

impl<B> TowerMakeSpan<B> for MakeSpan {
    fn make_span(&mut self, request: &axum::http::Request<B>) -> tracing::Span {
        let user_agent = request.headers().get(header::USER_AGENT);
        if user_agent.is_some() {
            tracing::info_span!(
                "request",
                method = %request.method(),
                uri = %request.uri(),
                version = ?request.version(),
                request_id = ?request.headers().get(HeaderName::from_static("x-request-id")).unwrap(),
                user_agent = ?user_agent.unwrap(),
                status = field::Empty,
            )
        } else {
            tracing::info_span!(
                "request",
                method = %request.method(),
                uri = %request.uri(),
                version = ?request.version(),
                request_id = ?request.headers().get(HeaderName::from_static("x-request-id")).unwrap(),
                status = field::Empty,
            )
        }
    }
}

impl<B> TowerOnResponse<B> for OnResponse {
    fn on_response(
        self,
        response: &axum::http::Response<B>,
        _: std::time::Duration,
        span: &tracing::Span,
    ) {
        span.record("status", response.status().as_u16());
    }
}
