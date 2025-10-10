use crate::bootstrap::init::setup_test_node;
use base64::{Engine as _, prelude::BASE64_STANDARD};
use migration::{Migrator, MigratorTrait};
use node::api::node::Node;
use node::bootstrap::init::NodeData;
use node::modules::ssi::webauthn::state::AuthState;
use sea_orm::Database;
use tempfile::TempDir;
use webauthn_authenticator_rs::{AuthenticatorBackend, softpasskey::SoftPasskey};
use webauthn_rs::prelude::Url;

// ========== Registration Tests ==========

#[tokio::test]
async fn test_start_registration_creates_challenge() {
    // Setup
    let (node, _temp) = setup_test_node().await;

    // Execute: Start registration
    let result = node.start_webauthn_registration().await;

    // Assert: Registration should succeed
    assert!(
        result.is_ok(),
        "start_registration should succeed: {:?}",
        result.err()
    );

    let (challenge_response, challenge_id) = result.unwrap();

    // Verify challenge response structure
    assert!(
        !challenge_response.public_key.challenge.is_empty(),
        "Challenge should not be empty"
    );

    // Verify challenge ID is a valid base64 string
    assert!(!challenge_id.is_empty(), "Challenge ID should not be empty");

    // Verify challenge ID is base64 encoded
    let decoded = BASE64_STANDARD.decode(&challenge_id);
    assert!(
        decoded.is_ok(),
        "Challenge ID should be valid base64: {}",
        challenge_id
    );

    // Verify public key credential creation options
    let public_key = &challenge_response.public_key;

    // Check relying party info
    assert_eq!(
        public_key.rp.name, "Test Flow",
        "RP name should match config"
    );
    assert_eq!(
        public_key.rp.id,
        "localhost".to_string(),
        "RP ID should match config"
    );

    // Check user info (should use device_id)
    assert_eq!(
        public_key.user.name, "test-device-123",
        "User name should be device ID"
    );
    assert_eq!(
        public_key.user.display_name, "test-device-123",
        "Display name should be device ID"
    );
    assert!(
        !public_key.user.id.is_empty(),
        "User ID should not be empty"
    );

    // Check credential parameters (should support ES256 at minimum)
    assert!(
        !public_key.pub_key_cred_params.is_empty(),
        "Should have at least one credential parameter"
    );

    let has_es256 = public_key
        .pub_key_cred_params
        .iter()
        .any(|param| param.alg == -7); // ES256 = -7 in COSE
    assert!(has_es256, "Should support ES256 algorithm");

    // Check timeout (should be reasonable)
    if let Some(timeout) = public_key.timeout {
        assert!(
            timeout >= 30000 && timeout <= 600000,
            "Timeout should be between 30s and 10min, got: {}ms",
            timeout
        );
    }

    // Verify exclude_credentials is None (no existing credentials for new device)
    assert!(
        public_key.exclude_credentials.is_none()
            || public_key.exclude_credentials.as_ref().unwrap().is_empty(),
        "Should not exclude credentials for new device"
    );

    // Verify attestation preference
    assert!(
        public_key.attestation.is_some(),
        "Should specify attestation preference"
    );
}

#[tokio::test]
async fn test_start_registration_challenge_is_unique() {
    // Setup
    let (node, _temp) = setup_test_node().await;

    // Execute: Start registration twice
    let result1 = node.start_webauthn_registration().await.unwrap();
    let result2 = node.start_webauthn_registration().await.unwrap();

    let (challenge1, challenge_id1) = result1;
    let (challenge2, challenge_id2) = result2;

    // Assert: Challenges should be different
    assert_ne!(
        challenge1.public_key.challenge, challenge2.public_key.challenge,
        "Each registration should generate a unique challenge"
    );

    assert_ne!(
        challenge_id1, challenge_id2,
        "Each registration should generate a unique challenge ID"
    );
}

#[tokio::test]
async fn test_start_registration_with_multiple_devices() {
    // Setup
    let temp_dir = TempDir::new().unwrap();

    // Create database (shared)
    let db_path = temp_dir.path().join("test.db");
    let db_url = format!("sqlite://{}?mode=rwc", db_path.display());
    let db = Database::connect(&db_url).await.unwrap();
    Migrator::up(&db, None).await.unwrap();

    // Create two nodes with different device IDs
    let kv_path1 = temp_dir.path().join("kv1");
    let kv1 = sled::open(&kv_path1).unwrap();

    let kv_path2 = temp_dir.path().join("kv2");
    let kv2 = sled::open(&kv_path2).unwrap();

    let auth_config = node::modules::ssi::webauthn::state::AuthConfig {
        rp_id: "localhost".to_string(),
        rp_origin: "http://localhost:3000".to_string(),
        rp_name: "Test Flow".to_string(),
    };

    let node1 = Node::new(
        NodeData {
            id: "device-1".to_string(),
            private_key: vec![0u8; 32],
            public_key: vec![0u8; 32],
        },
        db.clone(),
        kv1,
        AuthState::new(auth_config.clone()).unwrap(),
    );

    let node2 = Node::new(
        NodeData {
            id: "device-2".to_string(),
            private_key: vec![1u8; 32],
            public_key: vec![1u8; 32],
        },
        db.clone(),
        kv2,
        AuthState::new(auth_config).unwrap(),
    );

    // Execute: Start registration on both nodes
    let result1 = node1.start_webauthn_registration().await;
    let result2 = node2.start_webauthn_registration().await;

    // Assert: Both should succeed
    assert!(result1.is_ok(), "Device 1 registration should succeed");
    assert!(result2.is_ok(), "Device 2 registration should succeed");

    let (challenge1, _) = result1.unwrap();
    let (challenge2, _) = result2.unwrap();

    // Verify: Each device gets its own user info
    assert_eq!(challenge1.public_key.user.name, "device-1");
    assert_eq!(challenge2.public_key.user.name, "device-2");
}

#[tokio::test]
async fn test_end_to_end_registration_and_authentication() {
    // Setup test node
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db_url = format!("sqlite://{}?mode=rwc", db_path.display());
    let db = Database::connect(&db_url).await.unwrap();
    Migrator::up(&db, None).await.unwrap();

    let kv_path = temp_dir.path().join("kv");
    let kv = sled::open(&kv_path).unwrap();

    let auth_config = node::modules::ssi::webauthn::state::AuthConfig {
        rp_id: "localhost".to_string(),
        rp_origin: "http://localhost:3000".to_string(),
        rp_name: "Test Flow".to_string(),
    };

    let node = Node::new(
        NodeData {
            id: "test-device-e2e".to_string(),
            private_key: vec![0u8; 32],
            public_key: vec![0u8; 32],
        },
        db.clone(),
        kv,
        AuthState::new(auth_config).unwrap(),
    );

    // ===== REGISTRATION FLOW =====

    // 1. Start registration
    let (creation_challenge, challenge_id) = node
        .start_webauthn_registration()
        .await
        .expect("Failed to start registration");

    println!("Registration challenge created: {}", challenge_id);

    // 2. Simulate authenticator creating credential
    let mut authenticator = SoftPasskey::new(true); // user_verified = true, must be mutable

    let registration_credential = authenticator
        .perform_register(
            Url::parse("http://localhost:3000").unwrap(),
            creation_challenge.public_key.clone(), // Extract the PublicKeyCredentialCreationOptions
            60000,                                 // timeout in milliseconds
        )
        .expect("Failed to create credential");

    // 3. Finish registration
    let (did, user_id) = node
        .finish_webauthn_registration(&challenge_id, registration_credential)
        .await
        .expect("Failed to finish registration");

    println!("Registration successful!");
    println!("DID: {}", did);
    println!("User ID: {}", user_id);

    // Verify DID was created
    assert!(did.starts_with("did:key:"));
    assert!(!user_id.is_empty());

    // ===== AUTHENTICATION FLOW =====

    // 1. Start authentication
    let (auth_challenge, auth_challenge_id) = node
        .start_webauthn_authentication()
        .await
        .expect("Failed to start authentication");

    println!("Authentication challenge created: {}", auth_challenge_id);

    // 2. Simulate authenticator signing challenge
    let authentication_credential = authenticator
        .perform_auth(
            Url::parse("http://localhost:3000").unwrap(),
            auth_challenge.public_key.clone(), // Extract the PublicKeyCredentialRequestOptions
            60000,                             // timeout in milliseconds
        )
        .expect("Failed to authenticate");

    // 3. Finish authentication
    let auth_result = node
        .finish_webauthn_authentication(&auth_challenge_id, authentication_credential)
        .await
        .expect("Failed to finish authentication");

    println!("Authentication successful!");
    println!("Counter: {}", auth_result.counter());
    println!("User verified: {}", auth_result.user_verified());

    // Verify authentication result
    assert!(auth_result.user_verified());
    assert_eq!(auth_result.counter(), 1); // First authentication
}
