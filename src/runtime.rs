use crate::config::Config;
use crate::engine::execute_action;
use axum::{
    routing::post,
    Json, Router,
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[derive(Deserialize)]
struct ExecuteRequest {
    config: Config,
}

pub async fn serve(addr: &str) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/health", post(health))
        .route("/execute", post(execute));

    let addr: SocketAddr = addr.parse()?;
    let listener = TcpListener::bind(addr).await?;

    eprintln!("hsemulate runtime listening on http://{}", addr);
    eprintln!("health check: http://{}/health", addr);
    eprintln!("execute endpoint: POST http://{}/execute", addr);

    axum::serve(listener, app).await?;

    Ok(())
}


async fn health() -> &'static str {
    "ok"
}

async fn execute(
    Json(req): Json<ExecuteRequest>,
) -> impl IntoResponse {
    match execute_action(req.config, None).await {
        Ok(result) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "ok": result.ok,
                "runs": result.runs,
                "failures": result.failures,
                "max_duration_ms": result.max_duration_ms,
                "max_memory_kb": result.max_memory_kb,
                "snapshots_ok": result.snapshots_ok
            })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "ok": false,
                "error": e.to_string()
            })),
        ),
    }
}
