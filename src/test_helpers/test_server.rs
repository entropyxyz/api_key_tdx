//! A simple HTTP API which has authentication - for testing
use axum::{
    body::{Body, Bytes},
    extract::State,
    http::{Request, StatusCode},
    middleware::{self, Next},
    response::Response,
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;

const API_KEY_HEADER: &str = "api-key";
const VALID_API_KEY: &str = "some-secret";

/// Application state containing API keys of users
#[derive(Clone)]
struct AppState {
    accepted_api_keys: Vec<String>,
}

/// Start the test server in a spawned task
pub async fn start_test_api_server() {
    let app_state = Arc::new(AppState {
        accepted_api_keys: vec![VALID_API_KEY.to_string()],
    });

    let app = Router::new()
        .route("/protected", get(protected_handler))
        .route("/protected", post(protected_post_handler))
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            api_key_auth,
        ))
        .with_state(app_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3002));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::debug!("Test HTTP server running at http://{}", addr);

    tokio::spawn(async move {
        axum::serve(listener, app.into_make_service())
            .await
            .unwrap();
    });
}

/// An example GET handler
async fn protected_handler() -> &'static str {
    "Success response"
}

/// An example POST handler
async fn protected_post_handler(body: Bytes) -> String {
    format!(
        "Succcess response - input was {}",
        String::from_utf8_lossy(&body)
    )
}

/// Middleware to accept API keys given in either the header or the URL
async fn api_key_auth(
    State(state): State<Arc<AppState>>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    match request.headers().get(API_KEY_HEADER) {
        Some(key)
            if state
                .accepted_api_keys
                .contains(&key.to_str().unwrap_or_default().to_string()) =>
        {
            Ok(next.run(request).await)
        }
        _ => match request.uri().query() {
            Some(query_string) => {
                let params: std::collections::HashMap<_, _> =
                    url::form_urlencoded::parse(query_string.as_bytes())
                        .into_owned()
                        .collect();

                match params.get(API_KEY_HEADER) {
                    Some(key) if state.accepted_api_keys.contains(&key) => {
                        Ok(next.run(request).await)
                    }
                    _ => Err(StatusCode::UNAUTHORIZED),
                }
            }
            None => Err(StatusCode::UNAUTHORIZED),
        },
    }
}
