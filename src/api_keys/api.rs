use crate::{
    app_state::AppState, errors::Err, DeleteApiKeyInfo, DeployApiKeyInfo, SendApiKeyMessage,
};
use axum::{extract::State, http::StatusCode, Json};
use entropy_protocol::sign_and_encrypt::EncryptedSignedMessage;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::time::{SystemTime, UNIX_EPOCH};
use subxt::utils::AccountId32 as SubxtAccountId32;
use url::Url;

/// Defines the maximum allowed time difference for an api call in seconds
pub const TIME_BUFFER: u64 = 20;

pub async fn deploy_api_key(
    State(app_state): State<AppState>,
    Json(encrypted_msg): Json<EncryptedSignedMessage>,
) -> Result<StatusCode, Err> {
    let signed_message = encrypted_msg.decrypt(&app_state.x25519_secret, &[])?;

    let user_api_key_info: DeployApiKeyInfo = serde_json::from_slice(&signed_message.message.0)?;

    let request_author = SubxtAccountId32(*signed_message.account_id().as_ref());
    let current_timestamp = get_current_timestamp()?;

    check_stale(user_api_key_info.timestamp, current_timestamp).await?;
    let api_url = Url::parse(&user_api_key_info.api_url)?
        .host_str()
        .ok_or(Err::UrlHost)?
        .to_string();

    app_state.write_to_api_keys((request_author.0, api_url), user_api_key_info.api_key)?;

    Ok(StatusCode::OK)
}

pub async fn delete_secret(
    State(app_state): State<AppState>,
    Json(encrypted_msg): Json<EncryptedSignedMessage>,
) -> Result<StatusCode, Err> {
    let signed_message = encrypted_msg.decrypt(&app_state.x25519_secret, &[])?;

    let user_api_key_info: DeleteApiKeyInfo = serde_json::from_slice(&signed_message.message.0)?;
    let request_author = SubxtAccountId32(*signed_message.account_id().as_ref());

    let current_timestamp = get_current_timestamp()?;
    check_stale(user_api_key_info.timestamp, current_timestamp).await?;

    let api_url = Url::parse(&user_api_key_info.api_url)?
        .host_str()
        .ok_or(Err::UrlHost)?
        .to_string();

    app_state.delete_from_api_keys((request_author.0, api_url))?;

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

    let url_parsed = Url::parse(&user_make_request_info.api_url)?;
    let url_host = url_parsed.host_str().ok_or(Err::UrlHost)?.to_string();
    let api_key_info = app_state
        .read_from_api_keys(&(request_author.0, url_host))?
        .ok_or(Err::UrlEmpty)?;

    let client = reqwest::Client::new();
    let url = user_make_request_info
        .api_url
        .replace("xxxREPLACE_MExxx", &api_key_info);

    let mut headers = HeaderMap::new();
    for (key, value) in &user_make_request_info.http_headers {
        let first = key.replace("xxxREPLACE_MExxx", &api_key_info);
        let second = value.replace("xxxREPLACE_MExxx", &api_key_info);

        let header_name = HeaderName::from_bytes(first.as_bytes())?;
        let header_value = HeaderValue::from_str(&second)?;
        headers.insert(header_name, header_value);
    }

    let response = match user_make_request_info.http_verb.as_str() {
        "get" => Ok(client.get(url).headers(headers).send().await?),
        "post" => {
            let result = client
                .post(url)
                .headers(headers)
                .body(user_make_request_info.request_body)
                .send()
                .await?;

            Ok(result)
        }
        _ => Err(Err::UnsupportedHttpVerb),
    }?;

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
