use crate::{app_state::AppState, errors::Err};
use axum::{Json, extract::State, http::StatusCode};
use entropy_protocol::sign_and_encrypt::EncryptedSignedMessage;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use subxt::utils::AccountId32 as SubxtAccountId32;

/// Defines the maximum allowed time difference for an api call in seconds
pub const TIME_BUFFER: u64 = 20;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DeployApiKeyInfo {
    pub api_key: String,
    pub api_url: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SendApiKeyMessage {
    pub request_body: String,
    pub http_verb: String,
    pub api_url: String,
    pub timestamp: u64,
}

pub async fn deploy_api_key(
    State(app_state): State<AppState>,
    Json(encrypted_msg): Json<EncryptedSignedMessage>,
) -> Result<StatusCode, Err> {
    let signed_message = encrypted_msg.decrypt(&app_state.x25519_secret, &[])?;

    let user_api_key_info: DeployApiKeyInfo = serde_json::from_slice(&signed_message.message.0)?;

    let request_author = SubxtAccountId32(*signed_message.account_id().as_ref());
    let current_timestamp = get_current_timestamp()?;

    check_stale(user_api_key_info.timestamp, current_timestamp).await?;

    app_state.write_to_api_keys(
        (request_author.0, user_api_key_info.api_url),
        user_api_key_info.api_key,
    )?;

    Ok(StatusCode::OK)
}

pub async fn make_request(
    State(app_state): State<AppState>,
    Json(encrypted_msg): Json<EncryptedSignedMessage>,
) -> Result<(StatusCode, String), Err> {
    let signed_message = encrypted_msg.decrypt(&app_state.x25519_secret, &[])?;

    let user_make_request_info: SendApiKeyMessage =
        serde_json::from_slice(&signed_message.message.0)?;

    let request_author = SubxtAccountId32(*signed_message.account_id().as_ref());
    let current_timestamp = get_current_timestamp()?;

    check_stale(user_make_request_info.timestamp, current_timestamp).await?;

    let api_key_info = app_state
        .read_from_api_keys(&(request_author.0, user_make_request_info.api_url.clone()))?
        .unwrap();

    let client = reqwest::Client::new();

    let response = match user_make_request_info.http_verb.as_str() {
        "get" => {
            let url = user_make_request_info
                .api_url
                .replace("xxxREPLACE_MExxx", &api_key_info);
            Ok(client.get(url).send().await?)
        }
        "post" => {
            let result = client
                .post(user_make_request_info.api_url)
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", &api_key_info))
                .body(user_make_request_info.request_body)
                .send()
                .await?;
            Ok(result)
        }
        _ => Err(Err::UnsupportedHttpVerb),
    }?;

    // let result = client
    //     .post(user_make_request_info.api_url)
    //     .header("Content-Type", "application/json")
    //     .body(user_make_request_info.request_body)
    //     .send()
    //     .await?;

    Ok((StatusCode::OK, response.text().await?))
}
// Get current timestamp
pub fn get_current_timestamp() -> Result<u64, Err> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
}

/// Checks if the message sent was within X amount of time
pub async fn check_stale(user_timestamp: u64, current_timestamp: u64) -> Result<(), Err> {
    let time_difference = current_timestamp.abs_diff(user_timestamp);
    if time_difference > TIME_BUFFER {
        return Err(Err::StaleMessage);
    }
    Ok(())
}
