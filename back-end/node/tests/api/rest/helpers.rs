use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use node::api::servers::app_state::AppState;
use node::api::servers::rest;
use serde_json::Value;
use tempfile::TempDir;
use tower::ServiceExt;

/// Setup a test server with app state
pub async fn setup_test_server() -> (Router, TempDir) {
    // Create test node using bootstrap helpers
    let (node, temp) = crate::bootstrap::init::setup_test_node().await;

    // Create app state
    let app_state = AppState::new(node);

    // Use the actual router builder from rest.rs
    let router = rest::build_router(app_state);

    (router, temp)
}

/// Helper to make GET request
pub async fn get_request(app: &Router, uri: &str) -> (StatusCode, Value) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(uri)
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body)
        .unwrap_or_else(|_| String::from_utf8_lossy(&body).to_string().into());

    (status, json)
}

/// Helper to make POST request
pub async fn post_request(app: &Router, uri: &str, body: Value) -> (StatusCode, Value) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(uri)
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body)
        .unwrap_or_else(|_| String::from_utf8_lossy(&body).to_string().into());

    (status, json)
}

/// Check for CORS headers
pub fn assert_cors_headers(headers: &axum::http::HeaderMap) {
    assert!(
        headers.contains_key("access-control-allow-origin"),
        "Should have CORS allow-origin header"
    );
    assert!(
        headers.contains_key("access-control-allow-methods"),
        "Should have CORS allow-methods header"
    );
}
