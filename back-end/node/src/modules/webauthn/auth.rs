use crate::api::node::Node;
use base64::prelude::*;
use log::info;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use webauthn_rs::prelude::*;

static REG_CACHE: Lazy<Arc<Mutex<HashMap<String, (Uuid, String, PasskeyRegistration)>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

pub async fn start_registration(node: &Node) -> Result<CreationChallengeResponse, WebauthnError> {
    /// - obtain did:key from bootstrap, or generate new
    /// - get credential id linked to did:key in persistence (PassKey table)
    /// - trigger start_passkey_registration call
    /// - use did:key as unique_id and usernames
    /// - use obtained credential id as exclude_credentials
    /// - map did:key to reg_state (kv-store)
    info!("Starting registration.");

    let uuid = Uuid::new_v4();
    let device_id = node.node_data.id.clone();

    let res = match node.auth_state.webauthn.start_passkey_registration(
        uuid,
        device_id.as_str(),
        device_id.as_str(),
        None,
    ) {
        Ok((ccr, reg_state)) => {
            let challenge_key = BASE64_STANDARD.encode(&ccr.public_key.challenge);
            let value = (uuid, device_id, reg_state);

            let mut cache = REG_CACHE.lock().unwrap();
            cache.insert(challenge_key.clone(), value);

            info!(
                "Started Registration process with challenge: {}",
                challenge_key
            );
            ccr
        }
        Err(e) => {
            info!("error -> {:?}", e);
            return Err(e);
        }
    };

    Ok(res)
}

pub async fn finish_registration(
    node: &Node,
    challenge_key: &str,
    reg: RegisterPublicKeyCredential,
) -> Result<(), WebauthnError> {
    let (_, _, reg_state) = {
        let mut cache = REG_CACHE.lock().unwrap();
        cache
            .remove(challenge_key)
            .ok_or(WebauthnError::MismatchedChallenge)?
    };

    // Complete the registration
    let _passkey = node
        .auth_state
        .webauthn
        .finish_passkey_registration(&reg, &reg_state)?;

    // TODO: Store passkey in database
    // TODO: Generate DID from passkey
    // TODO: Save user and passkey

    Ok(())
}
