use crate::{api::servers::app_state::AppState, bootstrap::config::Config};
use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
};
use errors::AppError;
use serde_json::{Value, json};

pub async fn start(app_state: &AppState, config: &Config) -> Result<(), AppError> {
    let app = Router::new()
        .route("/api/v1/webauthn/start", post(start_webauthn_registration))
        .route(
            "/api/v1/webauthn/finish",
            post(finish_webauthn_registration),
        )
        .route("/api/v1/spaces", post(create_space))
        .route("/api/v1/health", get(health_check))
        .with_state(app_state.clone());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn start_webauthn_registration(
    State(app_state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let node = app_state.node.read().await;
    match node.start_webauthn_registration().await {
        Ok(challenge) => Ok(Json(json!({
            "challenge": challenge,
            "status": "success"
        }))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn finish_webauthn_registration(
    State(app_state): State<AppState>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    // Extract registration data from payload
    // Call node.finish_webauthn_registration()
    // Return result
    Ok(Json(json!({"status": "success"})))
}

async fn create_space(
    State(app_state): State<AppState>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let node = app_state.node.read().await;
    let dir = payload["dir"].as_str().unwrap_or("/tmp/space");

    match node.create_space(dir).await {
        Ok(_) => Ok(Json(json!({"status": "success"}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn health_check() -> Json<Value> {
    Json(json!({"status": "healthy", "timestamp": chrono::Utc::now()}))
}
