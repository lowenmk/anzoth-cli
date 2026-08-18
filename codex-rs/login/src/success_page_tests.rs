use crate::server::DEFAULT_ISSUER;
use base64::Engine;
use pretty_assertions::assert_eq;
use serde_json::json;

use super::*;

#[test]
fn compose_success_url_uses_local_page_by_default() {
    let LoginSuccessRedirect::Local(url) = compose_success_url(
        /*port*/ 1455,
        DEFAULT_ISSUER,
        "e30.eyJodHRwczovL2FwaS5vcGVuYWkuY29tL2F1dGgiOnt9fQ.sig",
        "e30.eyJodHRwczovL2FwaS5vcGVuYWkuY29tL2F1dGgiOnt9fQ.sig",
        /*codex_streamlined_login*/ false,
        &LoginSuccessPage::default(),
    ) else {
        panic!("expected local success redirect");
    };
    let url = Url::parse(&url).expect("success URL should parse");

    assert_eq!(url.host_str(), Some("localhost"));
    assert_eq!(url.path(), "/success");
    assert!(url.query().is_none(), "success URL should be clean");
}

#[test]
fn compose_success_url_remains_clean_for_local_setup_cases() {
    let LoginSuccessRedirect::Local(url) = compose_success_url(
        /*port*/ 1455,
        DEFAULT_ISSUER,
        "e30.eyJodHRwczovL2FwaS5vcGVuYWkuY29tL2F1dGgiOnt9fQ.sig",
        "e30.eyJodHRwczovL2FwaS5vcGVuYWkuY29tL2F1dGgiOnt9fQ.sig",
        /*codex_streamlined_login*/ true,
        &LoginSuccessPage::default(),
    ) else {
        panic!("expected local success redirect");
    };
    let url = Url::parse(&url).expect("success URL should parse");

    assert_eq!(url.as_str(), "http://localhost:1455/success");
}

#[test]
fn compose_success_url_uses_hosted_page_when_requested() {
    assert_eq!(
        compose_success_url(
            /*port*/ 1455,
            DEFAULT_ISSUER,
            "e30.eyJodHRwczovL2FwaS5vcGVuYWkuY29tL2F1dGgiOnt9fQ.sig",
            "e30.eyJodHRwczovL2FwaS5vcGVuYWkuY29tL2F1dGgiOnt9fQ.sig",
            /*codex_streamlined_login*/ false,
            &LoginSuccessPage::Hosted {
                url: Url::parse(CODEX_OPEN_APP_URL).expect("open app URL should parse"),
                app_brand: LoginSuccessPageBrand::Chatgpt,
            },
        ),
        LoginSuccessRedirect::Hosted(
            "https://anzoth.com/?source=login&app_brand=chatgpt".to_string()
        )
    );
}

#[test]
fn compose_success_url_keeps_setup_on_local_page() {
    let encode = |bytes: &[u8]| base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes);
    let payload = encode(
        serde_json::to_string(&json!({
            "https://api.openai.com/auth": {
                "completed_platform_onboarding": false,
                "is_org_owner": true,
                "organization_id": "org_123",
                "project_id": "proj_123",
            }
        }))
        .expect("payload should serialize")
        .as_bytes(),
    );
    let access_payload = encode(
        serde_json::to_string(&json!({
            "https://api.openai.com/auth": {
                "chatgpt_plan_type": "team",
            }
        }))
        .expect("payload should serialize")
        .as_bytes(),
    );
    let id_token = format!("e30.{payload}.sig");
    let LoginSuccessRedirect::Local(url) = compose_success_url(
        /*port*/ 1455,
        DEFAULT_ISSUER,
        &id_token,
        &format!("e30.{access_payload}.sig"),
        /*codex_streamlined_login*/ true,
        &LoginSuccessPage::Hosted {
            url: Url::parse(CODEX_OPEN_APP_URL).expect("open app URL should parse"),
            app_brand: LoginSuccessPageBrand::Codex,
        },
    ) else {
        panic!("expected local success redirect");
    };
    let url = Url::parse(&url).expect("success URL should parse");

    assert_eq!(url.host_str(), Some("localhost"));
    assert_eq!(url.path(), "/success");
    assert!(
        url.query().is_none(),
        "setup cases should also use the clean URL"
    );
}

#[test]
fn success_page_html_is_anzoth_branded_and_token_free() {
    let html = include_str!("assets/success.html");

    for expected in [
        "<title>Signed in to Anzoth</title>",
        "Signed in to Anzoth",
        "You can close this window and return to the Anzoth CLI.",
        r#"src="__ANZOTH_LOGO_DATA_URI__""#,
        r#"href="__ANZOTH_FAVICON_DATA_URI__""#,
        r#"window.history.replaceState(null, "", "/success");"#,
    ] {
        assert!(
            html.contains(expected),
            "success page should contain {expected:?}"
        );
    }

    for forbidden in [
        "Codex",
        "OpenAI",
        "terminal",
        "id_token",
        "access_token",
        "refresh_token",
        "Open Anzoth",
    ] {
        assert!(
            !html.contains(forbidden),
            "success page should not contain {forbidden:?}"
        );
    }
}

#[test]
fn legacy_success_page_html_is_anzoth_branded_and_token_free() {
    let html = include_str!("assets/success_legacy.html");

    for expected in [
        "<title>Signed in to Anzoth</title>",
        "Signed in to Anzoth",
        "You can close this window and return to the Anzoth CLI.",
        r#"src="__ANZOTH_LOGO_DATA_URI__""#,
        r#"href="__ANZOTH_FAVICON_DATA_URI__""#,
        r#"window.history.replaceState(null, "", "/success");"#,
    ] {
        assert!(
            html.contains(expected),
            "legacy success page should contain {expected:?}"
        );
    }

    for forbidden in [
        "Codex",
        "OpenAI",
        "terminal",
        "id_token",
        "access_token",
        "refresh_token",
        "Open Anzoth",
    ] {
        assert!(
            !html.contains(forbidden),
            "legacy success page should not contain {forbidden:?}"
        );
    }
}
