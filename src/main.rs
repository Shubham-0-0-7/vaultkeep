use std::net::SocketAddr;
use tokio::signal;
use tracing::info;

mod app;
mod db;
mod routes;
mod telemetry;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    telemetry::init_telemetry();

    let database_url = std::env::var("DATABASE_URL")
    .unwrap_or_else(|_| "postgres://vaultkeep:vaultkeep_secret_pass@localhost:5432/vaultkeep".to_string());

    let db_pool = db::init_db_pool(&database_url).await?;
    let app = app::create_app(db_pool);
    let addr = SocketAddr::from(([0, 0, 0, 0], 7777));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("vaultkeep listening on http://{}", addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    info!("vaultkeep shutdown cleanly.");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => info!("Received SIGINT (Ctrl+C), starting graceful shutdown"),
        _ = terminate => info!("Received SIGTERM, starting graceful shutdown"),
    }
}