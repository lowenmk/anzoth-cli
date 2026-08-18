use super::*;

use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use serde_json::json;
use wiremock::Mock;
use wiremock::MockServer;
use wiremock::Request;
use wiremock::ResponseTemplate;
use wiremock::matchers::method;
use wiremock::matchers::path;

#[test]
fn device_code_prompt_renders_phishing_warning() {
    let prompt = device_code_prompt("https://example.com/device", Some("ABCD-EFGH"));

    assert!(prompt.contains(
        "\x1b[90mContinue only if you started this login in Anzoth CLI. If a website or another person gave you this code, cancel.\x1b[0m"
    ));
}

#[test]
fn device_code_prompt_omits_code_when_complete_url_is_available() {
    let prompt = device_code_prompt(
        "https://auth.anzoth.com/realms/anzoth/device?user_code=ABCD",
        None,
    );
    assert!(prompt.contains("https://auth.anzoth.com/realms/anzoth/device?user_code=ABCD"));
    assert!(!prompt.contains("Enter this one-time code"));
}

#[test]
fn native_device_endpoints_are_keycloak_native() {
    let auth_endpoint =
        native_device_authorization_endpoint("https://auth.anzoth.com/realms/anzoth");
    assert_eq!(
        auth_endpoint,
        "https://auth.anzoth.com/realms/anzoth/protocol/openid-connect/auth/device"
    );
    assert!(!auth_endpoint.contains("/codex/device"));
    assert!(!auth_endpoint.contains("auth.openai.com"));
    assert_eq!(
        native_device_token_endpoint("https://auth.anzoth.com/realms/anzoth"),
        "https://auth.anzoth.com/realms/anzoth/protocol/openid-connect/token"
    );
}

#[test]
fn native_poll_delay_reacts_to_error_codes() {
    assert_eq!(
        native_poll_delay_seconds(0, "authorization_pending"),
        Some(0)
    );
    assert_eq!(
        native_poll_delay_seconds(5, "authorization_pending"),
        Some(5)
    );
    assert_eq!(native_poll_delay_seconds(0, "slow_down"), Some(5));
    assert_eq!(native_poll_delay_seconds(7, "slow_down"), Some(12));
    assert_eq!(native_poll_delay_seconds(3, "access_denied"), None);
    assert_eq!(native_poll_delay_seconds(3, "expired_token"), None);
    assert_eq!(native_poll_delay_seconds(3, "something_else"), None);
}

#[test]
fn native_device_response_prefers_complete_url() {
    let device = device_code_from_native_response(
        NativeDeviceCodeResp {
            device_code: "device-code".to_string(),
            user_code: "USER-CODE".to_string(),
            verification_uri: "https://auth.anzoth.com/realms/anzoth/device".to_string(),
            verification_uri_complete: Some(
                "https://auth.anzoth.com/realms/anzoth/device?user_code=USER-CODE".to_string(),
            ),
            expires_in: 600,
            interval: 5,
        },
        Some("verifier".to_string()),
    );

    assert_eq!(
        device.verification_url,
        "https://auth.anzoth.com/realms/anzoth/device?user_code=USER-CODE"
    );
    assert_eq!(
        device.verification_url_complete.as_deref(),
        Some("https://auth.anzoth.com/realms/anzoth/device?user_code=USER-CODE")
    );
    assert_eq!(device.user_code, "USER-CODE");
    assert_eq!(device.expires_in, 600);
    assert_eq!(device.pkce_verifier.as_deref(), Some("verifier"));
}

#[test]
fn native_device_response_falls_back_to_verification_uri() {
    let device = device_code_from_native_response(
        NativeDeviceCodeResp {
            device_code: "device-code".to_string(),
            user_code: "USER-CODE".to_string(),
            verification_uri: "https://auth.anzoth.com/realms/anzoth/device".to_string(),
            verification_uri_complete: None,
            expires_in: 600,
            interval: 5,
        },
        Some("verifier".to_string()),
    );

    assert_eq!(
        device.verification_url,
        "https://auth.anzoth.com/realms/anzoth/device"
    );
    assert!(device.verification_url_complete.is_none());
    assert_eq!(device.user_code, "USER-CODE");
    assert_eq!(device.expires_in, 600);
    assert_eq!(device.pkce_verifier.as_deref(), Some("verifier"));
}

#[tokio::test]
async fn native_device_authorization_request_has_pkce_fields() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/protocol/openid-connect/auth/device"))
        .respond_with(|request: &Request| {
            let body = String::from_utf8(request.body.clone()).unwrap();
            assert!(body.contains("client_id=client-id"));
            assert!(body.contains("scope=openid+profile+email") || body.contains("scope=openid%20profile%20email"));
            assert!(body.contains("code_challenge="));
            assert!(body.contains("code_challenge_method=S256"));
            ResponseTemplate::new(200).set_body_json(json!({
                "device_code": "device-code-123",
                "user_code": "USER-CODE",
                "verification_uri": "https://auth.anzoth.com/realms/anzoth/device",
                "verification_uri_complete": "https://auth.anzoth.com/realms/anzoth/device?user_code=USER-CODE",
                "expires_in": 600,
                "interval": 5
            }))
        })
        .mount(&server)
        .await;

    let client = create_raw_auth_client(&server.uri(), None).unwrap();
    let (device, pkce_verifier) = request_native_device_code(&client, &server.uri(), "client-id")
        .await
        .expect("native device request should succeed");
    assert_eq!(device.device_code, "device-code-123");
    assert!(!pkce_verifier.is_empty());
}

#[tokio::test]
async fn native_poll_authorization_pending_retries_until_success() {
    let server = MockServer::start().await;
    let attempts = Arc::new(AtomicUsize::new(0));
    let attempts_for_mock = attempts.clone();
    Mock::given(method("POST"))
        .and(path("/protocol/openid-connect/token"))
        .respond_with(move |_: &Request| {
            let attempt = attempts_for_mock.fetch_add(1, Ordering::SeqCst);
            if attempt == 0 {
                ResponseTemplate::new(400).set_body_json(json!({
                    "error": "authorization_pending"
                }))
            } else {
                ResponseTemplate::new(200).set_body_json(json!({
                    "id_token": "id-token-123",
                    "access_token": "access-token-123",
                    "refresh_token": "refresh-token-123"
                }))
            }
        })
        .expect(2)
        .mount(&server)
        .await;

    let client = create_raw_auth_client(&server.uri(), None).unwrap();
    let tokens = poll_native_for_token(
        &client,
        &server.uri(),
        "device-code",
        "client-id",
        Some("verifier"),
        30,
        0,
    )
    .await
    .expect("authorization_pending should retry");

    assert_eq!(tokens.id_token, "id-token-123");
    assert_eq!(tokens.access_token, "access-token-123");
    assert_eq!(tokens.refresh_token, "refresh-token-123");
    assert_eq!(attempts.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn native_poll_sends_code_verifier() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/protocol/openid-connect/token"))
        .respond_with(|request: &Request| {
            let body = String::from_utf8(request.body.clone()).unwrap();
            assert!(
                body.contains("grant_type=urn%3Aietf%3Aparams%3Aoauth%3Agrant-type%3Adevice_code")
                    || body.contains("grant_type=urn:ietf:params:oauth:grant-type:device_code")
            );
            assert!(body.contains("code_verifier=verifier-123"));
            ResponseTemplate::new(200).set_body_json(json!({
                "id_token": "id-token-123",
                "access_token": "access-token-123",
                "refresh_token": "refresh-token-123"
            }))
        })
        .mount(&server)
        .await;

    let client = create_raw_auth_client(&server.uri(), None).unwrap();
    let tokens = poll_native_for_token(
        &client,
        &server.uri(),
        "device-code",
        "client-id",
        Some("verifier-123"),
        30,
        0,
    )
    .await
    .expect("code_verifier should be sent");

    assert_eq!(tokens.id_token, "id-token-123");
}

#[tokio::test]
async fn native_poll_slow_down_then_succeeds() {
    let server = MockServer::start().await;
    let attempts = Arc::new(AtomicUsize::new(0));
    let attempts_for_mock = attempts.clone();
    Mock::given(method("POST"))
        .and(path("/protocol/openid-connect/token"))
        .respond_with(move |_: &Request| {
            let attempt = attempts_for_mock.fetch_add(1, Ordering::SeqCst);
            if attempt == 0 {
                ResponseTemplate::new(400).set_body_json(json!({
                    "error": "slow_down"
                }))
            } else {
                ResponseTemplate::new(200).set_body_json(json!({
                    "id_token": "id-token-123",
                    "access_token": "access-token-123",
                    "refresh_token": "refresh-token-123"
                }))
            }
        })
        .expect(2)
        .mount(&server)
        .await;

    let client = create_raw_auth_client(&server.uri(), None).unwrap();
    let tokens = poll_native_for_token(
        &client,
        &server.uri(),
        "device-code",
        "client-id",
        Some("verifier"),
        30,
        0,
    )
    .await
    .expect("slow_down should retry");

    assert_eq!(tokens.id_token, "id-token-123");
    assert_eq!(attempts.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn native_poll_access_denied_fails_closed() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/protocol/openid-connect/token"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "error": "access_denied"
        })))
        .mount(&server)
        .await;

    let client = create_raw_auth_client(&server.uri(), None).unwrap();
    let err = poll_native_for_token(
        &client,
        &server.uri(),
        "device-code",
        "client-id",
        Some("verifier"),
        30,
        0,
    )
    .await
    .expect_err("access_denied should fail closed");
    assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied);
}

#[tokio::test]
async fn native_poll_expired_token_fails_closed() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/protocol/openid-connect/token"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "error": "expired_token"
        })))
        .mount(&server)
        .await;

    let client = create_raw_auth_client(&server.uri(), None).unwrap();
    let err = poll_native_for_token(
        &client,
        &server.uri(),
        "device-code",
        "client-id",
        Some("verifier"),
        30,
        0,
    )
    .await
    .expect_err("expired_token should fail closed");
    assert_eq!(err.kind(), std::io::ErrorKind::TimedOut);
}

#[tokio::test]
async fn native_poll_respects_expires_in() {
    let server = MockServer::start().await;

    let client = create_raw_auth_client(&server.uri(), None).unwrap();
    let err = poll_native_for_token(
        &client,
        &server.uri(),
        "device-code",
        "client-id",
        Some("verifier"),
        0,
        0,
    )
    .await
    .expect_err("expires_in=0 should time out immediately");
    assert_eq!(err.kind(), std::io::ErrorKind::TimedOut);
}
