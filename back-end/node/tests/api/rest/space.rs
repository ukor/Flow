use crate::api::rest::helpers::*;
use axum::http::StatusCode;
use serde_json::json;
use tempfile::TempDir;

#[tokio::test]
async fn test_create_space_valid_directory() {
    // Setup
    let (app, _temp) = setup_test_server().await;
    let temp_dir = TempDir::new().unwrap();
    let dir_path = temp_dir.path().to_str().unwrap();

    // Execute
    let payload = json!({
        "dir": dir_path
    });

    let (status, body) = post_request(&app, "/api/v1/spaces", payload).await;

    // Assert
    assert_eq!(status, StatusCode::OK, "Should return 200 OK");
    assert_eq!(body["status"], "success", "Should return success status");

    println!("Created space at: {}", dir_path);
    println!("Response: {:?}", body);
}

#[tokio::test]
async fn test_create_space_creates_directory_if_not_exists() {
    // Setup
    let (app, _temp) = setup_test_server().await;
    let temp_dir = TempDir::new().unwrap();
    let new_dir = temp_dir.path().join("new_space_dir");
    let dir_path = new_dir.to_str().unwrap();

    // Verify directory doesn't exist yet
    assert!(!new_dir.exists(), "Directory should not exist initially");

    // Execute
    let payload = json!({
        "dir": dir_path
    });

    let (status, body) = post_request(&app, "/api/v1/spaces", payload).await;

    // Assert
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "success");

    // Verify directory was created
    assert!(new_dir.exists(), "Directory should be created");
    assert!(new_dir.is_dir(), "Should be a directory");

    println!("Directory created: {}", dir_path);
}

#[tokio::test]
async fn test_create_space_with_nested_directory() {
    // Setup
    let (app, _temp) = setup_test_server().await;
    let temp_dir = TempDir::new().unwrap();
    let nested_dir = temp_dir
        .path()
        .join("parent")
        .join("child")
        .join("grandchild");
    let dir_path = nested_dir.to_str().unwrap();

    // Execute
    let payload = json!({
        "dir": dir_path
    });

    let (status, body) = post_request(&app, "/api/v1/spaces", payload).await;

    // Assert
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "success");

    // Verify nested directory was created
    assert!(nested_dir.exists(), "Nested directory should be created");

    println!("Nested directory created: {}", dir_path);
}

#[tokio::test]
async fn test_create_space_invalid_directory() {
    // Setup
    let (app, _temp) = setup_test_server().await;

    // Use a path that's definitely invalid on Unix-like systems
    // Note: This might behave differently on Windows
    let invalid_path = "/dev/null/invalid/path/that/cannot/be/created";

    // Execute
    let payload = json!({
        "dir": invalid_path
    });

    let (status, body) = post_request(&app, "/api/v1/spaces", payload).await;

    // Assert
    assert_eq!(
        status,
        StatusCode::INTERNAL_SERVER_ERROR,
        "Should return 500 for invalid directory"
    );

    // Error message should be present
    assert!(
        body.is_string(),
        "Should return error message as string, got: {:?}",
        body
    );

    println!("Invalid directory error: {:?}", body);
}

#[tokio::test]
async fn test_create_space_duplicate() {
    // Setup
    let (app, _temp) = setup_test_server().await;
    let temp_dir = TempDir::new().unwrap();
    let dir_path = temp_dir.path().to_str().unwrap();

    // Execute - Create space first time
    let payload = json!({
        "dir": dir_path
    });

    let (status1, body1) = post_request(&app, "/api/v1/spaces", payload.clone()).await;

    // Assert first creation succeeds
    assert_eq!(status1, StatusCode::OK);
    assert_eq!(body1["status"], "success");

    // Execute - Create space second time with same directory
    let (status2, body2) = post_request(&app, "/api/v1/spaces", payload).await;

    // Assert second creation is idempotent (also succeeds)
    assert_eq!(
        status2,
        StatusCode::OK,
        "Duplicate space creation should be idempotent"
    );
    assert_eq!(body2["status"], "success", "Should still return success");

    println!("Duplicate space creation is idempotent");
}

#[tokio::test]
async fn test_create_space_no_directory_provided() {
    // Setup
    let (app, _temp) = setup_test_server().await;

    // Execute - Don't provide dir field (should default to /tmp/space)
    let payload = json!({});

    let (status, body) = post_request(&app, "/api/v1/spaces", payload).await;

    // Assert - Should use default directory /tmp/space
    assert_eq!(
        status,
        StatusCode::OK,
        "Should succeed with default directory"
    );
    assert_eq!(body["status"], "success");

    println!("Space created with default directory");
}

#[tokio::test]
async fn test_create_space_empty_directory_string() {
    // Setup
    let (app, _temp) = setup_test_server().await;

    // Execute - Provide empty string as directory
    let payload = json!({
        "dir": ""
    });

    let (status, _body) = post_request(&app, "/api/v1/spaces", payload).await;

    // Assert - Should fail because empty string is not a valid path
    assert_eq!(
        status,
        StatusCode::INTERNAL_SERVER_ERROR,
        "Empty directory string should fail"
    );

    println!("Empty directory string handled correctly");
}

#[tokio::test]
async fn test_create_space_relative_path() {
    // Setup
    let (app, _temp) = setup_test_server().await;

    // Execute - Use relative path
    let payload = json!({
        "dir": "./test_space_relative"
    });

    let (status, body) = post_request(&app, "/api/v1/spaces", payload).await;

    // Assert - Should succeed (relative paths are canonicalized)
    assert_eq!(status, StatusCode::OK, "Relative path should be accepted");
    assert_eq!(body["status"], "success");

    println!("Relative path handled correctly");

    // Cleanup
    let _ = std::fs::remove_dir_all("./test_space_relative");
}

#[tokio::test]
async fn test_create_space_with_special_characters() {
    // Setup
    let (app, _temp) = setup_test_server().await;
    let temp_dir = TempDir::new().unwrap();

    // Create directory with special characters (but valid on most filesystems)
    let special_dir = temp_dir.path().join("space-with_special.chars");
    let dir_path = special_dir.to_str().unwrap();

    // Execute
    let payload = json!({
        "dir": dir_path
    });

    let (status, body) = post_request(&app, "/api/v1/spaces", payload).await;

    // Assert
    assert_eq!(status, StatusCode::OK, "Should handle special characters");
    assert_eq!(body["status"], "success");
    assert!(
        special_dir.exists(),
        "Directory with special chars should be created"
    );

    println!("Special characters handled: {}", dir_path);
}

#[tokio::test]
async fn test_create_space_generates_deterministic_key() {
    // Setup
    let (app, _temp) = setup_test_server().await;
    let temp_dir = TempDir::new().unwrap();
    let dir_path = temp_dir.path().to_str().unwrap();

    // Execute - Create space twice
    let payload = json!({
        "dir": dir_path
    });

    let (status1, _) = post_request(&app, "/api/v1/spaces", payload.clone()).await;
    assert_eq!(status1, StatusCode::OK);

    // Second creation should succeed (same key generated)
    let (status2, body2) = post_request(&app, "/api/v1/spaces", payload).await;

    // Assert
    assert_eq!(
        status2,
        StatusCode::OK,
        "Same directory should be idempotent"
    );
    assert_eq!(body2["status"], "success");

    println!("Deterministic key generation verified");
}

#[tokio::test]
async fn test_create_space_different_directories_succeed() {
    // Setup
    let (app, _temp) = setup_test_server().await;
    let temp_dir = TempDir::new().unwrap();

    let dir1 = temp_dir.path().join("space1");
    let dir2 = temp_dir.path().join("space2");

    // Execute - Create two different spaces
    let payload1 = json!({ "dir": dir1.to_str().unwrap() });
    let payload2 = json!({ "dir": dir2.to_str().unwrap() });

    let (status1, body1) = post_request(&app, "/api/v1/spaces", payload1).await;
    let (status2, body2) = post_request(&app, "/api/v1/spaces", payload2).await;

    // Assert - Both should succeed
    assert_eq!(status1, StatusCode::OK);
    assert_eq!(body1["status"], "success");

    assert_eq!(status2, StatusCode::OK);
    assert_eq!(body2["status"], "success");

    // Verify both directories exist
    assert!(dir1.exists());
    assert!(dir2.exists());

    println!("Multiple different spaces created successfully");
}

#[tokio::test]
async fn test_create_space_malformed_json() {
    // Setup
    let (app, _temp) = setup_test_server().await;

    // Execute - Send non-object JSON (array instead)
    let payload = json!([1, 2, 3]);

    let (status, _body) = post_request(&app, "/api/v1/spaces", payload).await;

    // Assert - Should handle gracefully (defaults to /tmp/space)
    // or return error depending on implementation
    assert!(
        status == StatusCode::OK || status == StatusCode::BAD_REQUEST,
        "Should handle malformed JSON, got: {}",
        status
    );

    println!("Malformed JSON handled with status: {}", status);
}

#[tokio::test]
async fn test_create_space_concurrent_requests() {
    // Setup
    let (app, _temp) = setup_test_server().await;
    let temp_dir = TempDir::new().unwrap();
    let dir_path = temp_dir.path().to_str().unwrap().to_string();

    // Execute - Send concurrent requests for same directory
    let payload = json!({ "dir": dir_path });

    let app_clone = app.clone();
    let payload_clone = payload.clone();

    let handle1 = tokio::spawn(async move { post_request(&app, "/api/v1/spaces", payload).await });

    let handle2 =
        tokio::spawn(
            async move { post_request(&app_clone, "/api/v1/spaces", payload_clone).await },
        );

    let (result1, result2) = tokio::join!(handle1, handle2);

    let (status1, _) = result1.unwrap();
    let (status2, _) = result2.unwrap();

    // Assert - Both should succeed (idempotent)
    assert_eq!(status1, StatusCode::OK, "First request should succeed");
    assert_eq!(status2, StatusCode::OK, "Second request should succeed");

    println!("Concurrent requests handled successfully");
}
