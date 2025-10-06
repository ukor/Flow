use crate::{api::servers::app_state::AppState, bootstrap::config::Config};
use axum::{
    Router,
    extract::State,
    http::{HeaderValue, Method, StatusCode},
    response::Json,
    routing::{get, post},
};
use errors::AppError;
use log::info;
use serde_json::{Value, json};
use tower_http::cors::{Any, CorsLayer};

pub async fn start(app_state: &AppState, config: &Config) -> Result<(), AppError> {
    // Configure CORS
    let cors = CorsLayer::new()
        // Allow requests from these origins
        .allow_origin(["http://localhost:3000".parse::<HeaderValue>().unwrap()])
        // Allow these HTTP methods
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        // Allow headers
        .allow_headers(Any)
        // Cache preflight requests for 1 hour
        .max_age(std::time::Duration::from_secs(3600));

    // Configure Router
    let app = Router::new()
        .route(
            "/api/v1/webauthn/start_registration",
            get(start_webauthn_registration),
        )
        .route(
            "/api/v1/webauthn/finish_registration",
            post(finish_webauthn_registration),
        )
        .route("/api/v1/spaces", post(create_space))
        .route("/api/v1/health", get(health_check))
        .with_state(app_state.clone())
        .layer(cors);

    let bind_addr = format!("0.0.0.0:{}", config.server.rest_port);
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    axum::serve(listener, app).await?;

    info!("Rest Server up on addr: {}", &bind_addr);

    Ok(())
}

async fn start_webauthn_registration(
    State(app_state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let node = app_state.node.read().await;
    match node.start_webauthn_registration().await {
        Ok(challenge) => Ok(Json(json!(challenge))),
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
