pub(crate) mod cache;
pub mod collaboration_mode_presets;
pub(crate) mod config;
pub mod manager;
pub mod model_info;
pub mod model_presets;
pub mod test_support;

pub use codex_protocol::auth::AuthMode;
pub use config::ModelsManagerConfig;

/// Load the bundled model catalog shipped with `codex-models-manager`.
pub fn bundled_models_response()
-> std::result::Result<codex_protocol::openai_models::ModelsResponse, serde_json::Error> {
    serde_json::from_str(include_str!("../models.json"))
}

/// Convert the client version string to a whole version string (e.g. "1.2.3-alpha.4" -> "1.2.3").
pub fn client_version_to_whole() -> String {
    format!(
        "{}.{}.{}",
        env!("CARGO_PKG_VERSION_MAJOR"),
        env!("CARGO_PKG_VERSION_MINOR"),
        env!("CARGO_PKG_VERSION_PATCH")
    )
}

#[cfg(test)]
mod tests {
    use super::bundled_models_response;

    #[test]
    fn anzoth_model_catalog_parses_and_defaults_to_coder() {
        let response: codex_protocol::openai_models::ModelsResponse =
            serde_json::from_str(include_str!("../../models-anzoth.json"))
                .expect("Anzoth model catalog should parse");
        let slugs: Vec<&str> = response
            .models
            .iter()
            .map(|model| model.slug.as_str())
            .collect();
        assert_eq!(slugs, ["Anzoth-Coder", "Anzoth-Core"]);
        assert_eq!(
            response.models.first().map(|model| model.slug.as_str()),
            Some("Anzoth-Coder")
        );
        assert!(response
            .models
            .iter()
            .all(|model| !model.use_responses_lite));
        assert!(response
            .models
            .iter()
            .all(|model| model.tool_mode.is_none()));
        assert!(response
            .models
            .iter()
            .all(|model| model.multi_agent_version.is_none()));
        assert!(response
            .models
            .iter()
            .all(|model| !model.supports_search_tool));
        assert!(response.models.iter().all(|model| {
            matches!(
                model.apply_patch_tool_type,
                Some(codex_protocol::openai_models::ApplyPatchToolType::Function)
            )
        }));

        let bundled = bundled_models_response().expect("bundled catalog should still parse");
        assert!(
            !bundled
                .models
                .iter()
                .any(|model| model.slug.starts_with("Anzoth-")),
            "the upstream bundled catalog must remain unchanged"
        );
    }
}
