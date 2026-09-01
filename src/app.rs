use crate::routes::health::health_check;
use axum::{
    routing::get,
    Router,
};
use http::{
    header::{HeaderName, HeaderValue},
    HeaderMap, Request, Response,
};
use std::time::Duration;
use tower::ServiceBuilder;
use tower_http::{
    catch_panic::CatchPanicLayer,
    limit::RequestBodyLimitLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    set_header::SetResponseHeaderLayer,
    timeout::TimeoutLayer,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};



pub fn create_app() -> Router {
    let x_request_id = HeaderName::from_static("x_request_id");
    let security_headers = ServiceBuilder::new()
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("x-content-type-options"),
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("x-frame-options"),
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("strict-transport-security"),
            HeaderValue::from_static("max-age=63072000; includeSubDomains; preload"),
        ));

    let middleware_stack = ServiceBuilder::new()
        .layer(SetRequestIdLayer::new(x_request_id.clone(), MakeRequestUuid))
        .layer(PropagateRequestIdLayer::new(x_request_id))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().include_headers(false))
                .on_response(
                    DefaultOnResponse::new()
                        .level(tracing::Level::INFO)
                        .latency_unit(LatencyUnit::Millis),
                ),
        )
        .layer(CatchPanicLayer::new())
        .layer(TimeoutLayer::with_status_code(http::StatusCode::REQUEST_TIMEOUT, Duration::from_secs(30)))
        .layer(security_headers);

    Router::new()
        .route("/health", get(health_check))
        .layer(RequestBodyLimitLayer::new( 2 * 1024 * 1024))
        .layer(middleware_stack) 
}