use serial_test::serial;

use super::api::{TIME_BUFFER, check_stale};

#[tokio::test]
async fn test_stale_check() {
    let result = check_stale(1, 1).await;
    assert!(result.is_ok());

    let result_server_larger = check_stale(1, 2).await;
    assert!(result_server_larger.is_ok());

    let result_user_larger = check_stale(2, 1).await;
    assert!(result_user_larger.is_ok());

    let fail_stale = check_stale(1, 2 + TIME_BUFFER).await.unwrap_err();
    assert_eq!(fail_stale.to_string(), "Message is too old".to_string());
}
