use log::info;
use webauthn_rs::prelude::*;

use crate::modules::auth::state::AuthState;

pub async fn start_registration(auth_state: AuthState, id: &str) {
    info!("Starting registration.");
}
