use crate::bootstrap::init::NodeData;
use crate::modules::space;
use crate::modules::webauthn;
use crate::modules::webauthn::state::AuthState;
use errors::AppError;
use log::info;
use sea_orm::DatabaseConnection;
use sled::Db;
use webauthn_rs::prelude::CreationChallengeResponse;

pub struct Node {
    pub node_data: NodeData,
    pub db: DatabaseConnection,
    pub kv: Db,
    pub auth_state: AuthState,
}

impl Node {
    pub fn new(node_data: NodeData, db: DatabaseConnection, kv: Db, auth_state: AuthState) -> Self {
        Node {
            node_data,
            db,
            kv,
            auth_state,
        }
    }

    pub async fn create_space(&self, dir: &str) -> Result<(), AppError> {
        info!("Setting up space in Directory: {}", dir);
        space::new_space(&self.db, dir).await?;
        Ok(())
    }

    pub async fn start_webauthn_registration(&self) -> Result<CreationChallengeResponse, AppError> {
        info!("Starting WebAuthn Registration..");
        webauthn::auth::start_registration(self)
            .await
            .map_err(|e| AppError::Auth(format!("WebAuthn registration failed: {}", e)))
    }
}
