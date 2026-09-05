use axum::{response::IntoResponse, Json};
use axum::extract::State;
use http::StatusCode;
use serde::Serialize;
use sqlx::PgPool;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub version: &'static str,
    pub database: &'static str,
}

pub async fn health_check(State(pool): State<PgPool>) -> Result<impl IntoResponse, StatusCode> {
    match sqlx::query("SELECT 1").execute(&pool).await {
        Ok(_) => Ok(Json(HealthResponse {
            status: "ok",
            version: env!("CARGO_PKG_VERSION"),
            database: "connected",
        })),
        Err(err) => {
            tracing::error!("Database health check failed: {:?}", err);
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }
}