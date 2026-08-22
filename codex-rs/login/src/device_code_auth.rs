use codex_http_client::HttpClient;
use http::StatusCode;
use serde::Deserialize;
use serde::Serialize;
use serde::de::Deserializer;
use serde::de::{self};
use std::time::Duration;
use std::time::Instant;

use crate::default_client::create_raw_auth_client;
use crate::pkce::PkceCodes;
use crate::pkce::generate_pkce;
use crate::server::ServerOptions;
use std::io;

const ANSI_BLUE: &str = "\x1b[94m";
const ANSI_GRAY: &str = "\x1b[90m";
const ANSI_RESET: &str = "\x1b[0m";

#[derive(Debug, Clone)]
pub struct DeviceCode {
    pub verification_url: String,
    pub verification_url_complete: Option<String>,
    pub user_code: String,
    expires_in: u64,
    pkce_verifier: Option<String>,
    device_auth_id: String,
    interval: u64,
    flow: DeviceAuthFlow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DeviceAuthFlow {
    Legacy,
    Native,
}

#[derive(Deserialize)]
struct UserCodeResp {
    device_auth_id: String,
    #[serde(alias = "user_code", alias = "usercode")]
    user_code: String,
    #[serde(default, deserialize_with = "deserialize_interval")]
    interval: u64,
}

#[derive(Serialize)]
struct UserCodeReq {
    client_id: String,
}

#[derive(Serialize)]
struct TokenPollReq {
    device_auth_id: String,
    user_code: String,
}

#[derive(Deserialize)]
struct NativeDeviceCodeResp {
    device_code: String,
    user_code: String,
    verification_uri: String,
    #[serde(default)]
    verification_uri_complete: Option<String>,
    expires_in: u64,
    interval: u64,
}

#[derive(Debug, Deserialize)]
struct NativeTokenResp {
    id_token: String,
    access_token: String,
    refresh_token: String,
}

#[derive(Deserialize)]
struct NativeTokenErrorResp {
    error: Option<String>,
    error_description: Option<String>,
}

fn deserialize_interval<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    s.trim().parse::<u64>().map_err(de::Error::custom)
}

#[derive(Deserialize)]
struct CodeSuccessResp {
    authorization_code: String,
    code_challenge: String,
    code_verifier: String,
}

/// Request the user code and polling interval.
async fn request_user_code(
    client: &HttpClient,
    auth_base_url: &str,
    client_id: &str,
) -> std::io::Result<UserCodeResp> {
    let url = format!("{auth_base_url}/deviceauth/usercode");
    let body = serde_json::to_string(&UserCodeReq {
        client_id: client_id.to_string(),
    })
    .map_err(std::io::Error::other)?;
    let resp = client
        .post(url)
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await
        .map_err(std::io::Error::other)?;

    if !resp.status().is_success() {
        let status = resp.status();
        if status == StatusCode::NOT_FOUND {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "device code login is not enabled for this Codex server. Use the browser login or verify the server URL.",
            ));
        }

        return Err(std::io::Error::other(format!(
            "device code request failed with status {status}"
        )));
    }

    let body = resp.text().await.map_err(std::io::Error::other)?;
    serde_json::from_str(&body).map_err(std::io::Error::other)
}

fn native_device_authorization_endpoint(base_url: &str) -> String {
    format!(
        "{}/protocol/openid-connect/auth/device",
        base_url.trim_end_matches('/')
    )
}

fn native_device_token_endpoint(base_url: &str) -> String {
    format!(
        "{}/protocol/openid-connect/token",
        base_url.trim_end_matches('/')
    )
}

fn native_poll_delay_seconds(current_interval: u64, error: &str) -> Option<u64> {
    match error {
        "authorization_pending" => Some(current_interval),
        "slow_down" => Some(current_interval.saturating_add(5)),
        "access_denied" | "expired_token" => None,
        _ => None,
    }
}

fn device_code_from_native_response(
    uc: NativeDeviceCodeResp,
    pkce_verifier: Option<String>,
) -> DeviceCode {
    let verification_url_complete = uc.verification_uri_complete.clone();
    DeviceCode {
        verification_url: verification_url_complete
            .clone()
            .unwrap_or_else(|| uc.verification_uri.clone()),
        verification_url_complete,
        user_code: uc.user_code,
        expires_in: uc.expires_in,
        pkce_verifier,
        device_auth_id: uc.device_code,
        interval: uc.interval,
        flow: DeviceAuthFlow::Native,
    }
}

async fn request_native_device_code(
    client: &HttpClient,
    auth_base_url: &str,
    client_id: &str,
    login_hint: Option<&str>,
) -> std::io::Result<(NativeDeviceCodeResp, String)> {
    let pkce = generate_pkce();
    let body = {
        let mut serializer = url::form_urlencoded::Serializer::new(String::new());
        serializer
            .append_pair("client_id", client_id)
            .append_pair("scope", "openid profile email")
            .append_pair("code_challenge", &pkce.code_challenge)
            .append_pair("code_challenge_method", "S256");
        if let Some(login_hint) = login_hint {
            let login_hint = login_hint.trim();
            if !login_hint.is_empty() {
                serializer.append_pair("login_hint", login_hint);
            }
        }
        serializer.finish()
    };
    let resp = client
        .post(native_device_authorization_endpoint(auth_base_url))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .map_err(std::io::Error::other)?;

    if !resp.status().is_success() {
        let status = resp.status();
        return Err(std::io::Error::other(format!(
            "native device code request failed with status {status}"
        )));
    }

    let body = resp.text().await.map_err(std::io::Error::other)?;
    let response = serde_json::from_str(&body).map_err(std::io::Error::other)?;
    Ok((response, pkce.code_verifier))
}

/// Poll token endpoint until a code is issued or timeout occurs.
async fn poll_for_token(
    client: &HttpClient,
    auth_base_url: &str,
    device_auth_id: &str,
    user_code: &str,
    interval: u64,
) -> std::io::Result<CodeSuccessResp> {
    let url = format!("{auth_base_url}/deviceauth/token");
    let max_wait = Duration::from_secs(15 * 60);
    let start = Instant::now();

    loop {
        let body = serde_json::to_string(&TokenPollReq {
            device_auth_id: device_auth_id.to_string(),
            user_code: user_code.to_string(),
        })
        .map_err(std::io::Error::other)?;
        let resp = client
            .post(&url)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await
            .map_err(std::io::Error::other)?;

        let status = resp.status();

        if status.is_success() {
            return resp.json().await.map_err(std::io::Error::other);
        }

        if status == StatusCode::FORBIDDEN || status == StatusCode::NOT_FOUND {
            if start.elapsed() >= max_wait {
                return Err(std::io::Error::other(
                    "device auth timed out after 15 minutes",
                ));
            }
            let sleep_for = Duration::from_secs(interval).min(max_wait - start.elapsed());
            tokio::time::sleep(sleep_for).await;
            continue;
        }

        return Err(std::io::Error::other(format!(
            "device auth failed with status {}",
            resp.status()
        )));
    }
}

async fn poll_native_for_token(
    client: &HttpClient,
    auth_base_url: &str,
    device_code: &str,
    client_id: &str,
    pkce_verifier: Option<&str>,
    expires_in: u64,
    interval: u64,
) -> std::io::Result<NativeTokenResp> {
    let deadline = Instant::now() + Duration::from_secs(expires_in);
    let mut current_interval = Duration::from_secs(interval);

    loop {
        if Instant::now() >= deadline {
            return Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "device auth expired before authorization completed",
            ));
        }

        let body = {
            let mut serializer = url::form_urlencoded::Serializer::new(String::new());
            serializer
                .append_pair("client_id", client_id)
                .append_pair("grant_type", "urn:ietf:params:oauth:grant-type:device_code")
                .append_pair("device_code", device_code);
            if let Some(verifier) = pkce_verifier {
                serializer.append_pair("code_verifier", verifier);
            }
            serializer.finish()
        };
        let resp = client
            .post(native_device_token_endpoint(auth_base_url))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .map_err(std::io::Error::other)?;

        if resp.status().is_success() {
            let body = resp.text().await.map_err(std::io::Error::other)?;
            return serde_json::from_str(&body).map_err(std::io::Error::other);
        }

        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        let parsed_error = serde_json::from_str::<NativeTokenErrorResp>(&body).ok();
        let error = parsed_error
            .as_ref()
            .and_then(|payload| payload.error.as_deref())
            .unwrap_or_default();
        let description = parsed_error
            .as_ref()
            .and_then(|payload| payload.error_description.as_deref())
            .unwrap_or_default();

        match error {
            "authorization_pending" => {
                let remaining = deadline.saturating_duration_since(Instant::now());
                let sleep_for = current_interval.min(remaining);
                if sleep_for.is_zero() {
                    continue;
                }
                tokio::time::sleep(sleep_for).await;
            }
            "slow_down" => {
                current_interval = Duration::from_secs(
                    native_poll_delay_seconds(current_interval.as_secs(), "slow_down")
                        .unwrap_or_else(|| current_interval.as_secs()),
                );
                let remaining = deadline.saturating_duration_since(Instant::now());
                let sleep_for = current_interval.min(remaining);
                if sleep_for.is_zero() {
                    continue;
                }
                tokio::time::sleep(sleep_for).await;
            }
            "access_denied" => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "device authorization denied",
                ));
            }
            "expired_token" => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "device authorization expired",
                ));
            }
            _ => {
                return Err(std::io::Error::other(format!(
                    "native device auth failed with status {status}: {error} {description} {body}"
                )));
            }
        }
    }
}

fn device_code_prompt(verification_url: &str, code: Option<&str>) -> String {
    let version = env!("CARGO_PKG_VERSION");
    let code_section = code
        .map(|code| {
            format!(
                "\n2. Enter this one-time code {ANSI_GRAY}(expires in 15 minutes){ANSI_RESET}\n   {ANSI_BLUE}{code}{ANSI_RESET}\n"
            )
        })
        .unwrap_or_default();
    format!(
        "\nWelcome to Anzoth CLI [v{ANSI_GRAY}{version}{ANSI_RESET}]\n{ANSI_GRAY}Anzoth's command-line coding agent{ANSI_RESET}\n\
\nFollow these steps to sign in with your account using device code authorization:\n\
\n1. Open this link in your browser and sign in to your account\n   {ANSI_BLUE}{verification_url}{ANSI_RESET}\n\
{code_section}\
\n{ANSI_GRAY}Continue only if you started this login in Anzoth CLI. If a website or another person gave you this code, cancel.{ANSI_RESET}\n",
    )
}

fn print_device_code_prompt(verification_url: &str, code: Option<&str>) {
    let prompt = device_code_prompt(verification_url, code);
    println!("{prompt}");
}

pub async fn request_device_code(opts: &ServerOptions) -> std::io::Result<DeviceCode> {
    request_device_code_with_login_hint(opts, None).await
}

pub async fn request_device_code_with_login_hint(
    opts: &ServerOptions,
    login_hint: Option<&str>,
) -> std::io::Result<DeviceCode> {
    let base_url = opts.issuer.trim_end_matches('/');
    let client = create_raw_auth_client(base_url, opts.auth_route_config.as_ref())?;
    if base_url == "https://auth.anzoth.com/realms/anzoth" {
        let (uc, pkce_verifier) =
            request_native_device_code(&client, base_url, &opts.client_id, login_hint).await?;
        Ok(device_code_from_native_response(uc, Some(pkce_verifier)))
    } else {
        let api_base_url = format!("{base_url}/api/accounts");
        let uc = request_user_code(&client, &api_base_url, &opts.client_id).await?;

        Ok(DeviceCode {
            verification_url: format!("{base_url}/codex/device"),
            verification_url_complete: None,
            user_code: uc.user_code,
            expires_in: 15 * 60,
            pkce_verifier: None,
            device_auth_id: uc.device_auth_id,
            interval: uc.interval,
            flow: DeviceAuthFlow::Legacy,
        })
    }
}

pub async fn complete_device_code_login(
    opts: ServerOptions,
    device_code: DeviceCode,
) -> std::io::Result<()> {
    let base_url = opts.issuer.trim_end_matches('/');
    let client = create_raw_auth_client(base_url, opts.auth_route_config.as_ref())?;
    let tokens = match device_code.flow {
        DeviceAuthFlow::Native => {
            let native_tokens = poll_native_for_token(
                &client,
                base_url,
                &device_code.device_auth_id,
                &opts.client_id,
                device_code.pkce_verifier.as_deref(),
                device_code.expires_in,
                device_code.interval,
            )
            .await?;
            crate::server::ExchangedTokens {
                id_token: native_tokens.id_token,
                access_token: native_tokens.access_token,
                refresh_token: native_tokens.refresh_token,
            }
        }
        DeviceAuthFlow::Legacy => {
            let api_base_url = format!("{base_url}/api/accounts");

            let code_resp = poll_for_token(
                &client,
                &api_base_url,
                &device_code.device_auth_id,
                &device_code.user_code,
                device_code.interval,
            )
            .await?;

            let pkce = PkceCodes {
                code_verifier: code_resp.code_verifier,
                code_challenge: code_resp.code_challenge,
            };
            let redirect_uri = format!("{base_url}/deviceauth/callback");

            crate::server::exchange_code_for_tokens(
                base_url,
                &opts.client_id,
                &redirect_uri,
                &pkce,
                &code_resp.authorization_code,
                opts.auth_route_config.as_ref(),
            )
            .await
            .map_err(|err| std::io::Error::other(format!("device code exchange failed: {err}")))?
        }
    };

    if let Err(message) = crate::server::ensure_workspace_allowed(
        opts.forced_chatgpt_workspace_id.as_deref(),
        &tokens.id_token,
    ) {
        return Err(io::Error::new(io::ErrorKind::PermissionDenied, message));
    }

    crate::server::persist_tokens_async(
        &opts.codex_home,
        /*api_key*/ None,
        tokens.id_token,
        tokens.access_token,
        tokens.refresh_token,
        opts.cli_auth_credentials_store_mode,
        opts.auth_keyring_backend_kind,
    )
    .await
}

pub async fn run_device_code_login(opts: ServerOptions) -> std::io::Result<()> {
    let device_code = request_device_code_with_login_hint(&opts, None).await?;
    let code =
        (device_code.verification_url_complete.is_none()).then_some(device_code.user_code.as_str());
    print_device_code_prompt(&device_code.verification_url, code);
    complete_device_code_login(opts, device_code).await
}

pub async fn run_device_code_login_with_login_hint(
    opts: ServerOptions,
    login_hint: Option<&str>,
) -> std::io::Result<()> {
    let device_code = request_device_code_with_login_hint(&opts, login_hint).await?;
    let code =
        (device_code.verification_url_complete.is_none()).then_some(device_code.user_code.as_str());
    print_device_code_prompt(&device_code.verification_url, code);
    complete_device_code_login(opts, device_code).await
}

#[cfg(test)]
#[path = "device_code_auth_tests.rs"]
mod tests;
