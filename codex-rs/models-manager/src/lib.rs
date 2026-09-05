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

/// Load the bundled Anzoth runtime catalog shipped with the CLI.
pub fn bundled_anzoth_models_response()
-> std::result::Result<codex_protocol::openai_models::ModelsResponse, serde_json::Error> {
    serde_json::from_str(include_str!("../../models-anzoth.json"))
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
    use super::{bundled_anzoth_models_response, bundled_models_response};

    #[test]
    fn anzoth_model_catalog_parses_and_defaults_to_core() {
        let response: codex_protocol::openai_models::ModelsResponse =
            serde_json::from_str(include_str!("../../models-anzoth.json"))
                .expect("Anzoth model catalog should parse");
        let slugs: Vec<&str> = response
            .models
            .iter()
            .map(|model| model.slug.as_str())
            .collect();
        assert_eq!(slugs, ["Anzoth-Coder", "Anzoth-Core"]);
        assert!(
            response
                .models
                .iter()
                .any(|model| model.slug == "Anzoth-Core")
        );
        assert!(
            response
                .models
                .iter()
                .all(|model| !model.use_responses_lite)
        );
        assert!(
            response
                .models
                .iter()
                .all(|model| model.multi_agent_version.is_none())
        );
        assert!(
            response
                .models
                .iter()
                .all(|model| !model.supports_search_tool)
        );
        let bundled = bundled_anzoth_models_response().expect("bundled catalog should still parse");
        assert!(
            bundled
                .models
                .iter()
                .map(|model| model.slug.as_str())
                .eq(["Anzoth-Coder", "Anzoth-Core"]),
            "the bundled catalog must be the Anzoth runtime catalog"
        );
        let coder = response
            .models
            .iter()
            .find(|model| model.slug == "Anzoth-Coder")
            .expect("Anzoth-Coder should be bundled");
        assert_eq!(coder.context_window, Some(162500));
        assert_eq!(coder.max_context_window, Some(162500));
        assert_eq!(coder.effective_context_window_percent, 95);
        assert_eq!(
            coder.shell_type,
            codex_protocol::openai_models::ConfigShellToolType::ShellCommand
        );
        assert_eq!(
            coder.tool_mode,
            Some(codex_protocol::openai_models::ToolMode::Direct)
        );
        assert_eq!(coder.apply_patch_tool_type, None);
        assert_eq!(
            coder.input_modalities,
            vec![
                codex_protocol::openai_models::InputModality::Text,
                codex_protocol::openai_models::InputModality::Image,
            ]
        );

        let core = response
            .models
            .iter()
            .find(|model| model.slug == "Anzoth-Core")
            .expect("Anzoth-Core should be bundled");
        assert_eq!(core.context_window, Some(786432));
        assert_eq!(core.max_context_window, Some(786432));
        assert_eq!(core.effective_context_window_percent, 95);
        assert_eq!(
            core.shell_type,
            codex_protocol::openai_models::ConfigShellToolType::ShellCommand
        );
        assert_eq!(
            core.tool_mode,
            Some(codex_protocol::openai_models::ToolMode::CodeModeOnly)
        );
        assert_eq!(
            core.apply_patch_tool_type,
            Some(codex_protocol::openai_models::ApplyPatchToolType::Freeform)
        );
        assert_eq!(
            core.input_modalities,
            vec![
                codex_protocol::openai_models::InputModality::Text,
                codex_protocol::openai_models::InputModality::Image,
            ]
        );
    }

    #[test]
    fn bundled_models_catalog_uses_anzoth_identity_for_primary_model() {
        let response = bundled_models_response().expect("bundled models.json should parse");
        let model = response
            .models
            .iter()
            .find(|model| model.slug == "Anzoth-Core")
            .expect("bundled catalog should contain Anzoth-Core");
        assert_eq!(model.display_name, "Anzoth-Core");
        assert_eq!(
            model.description.as_deref(),
            Some("General-purpose model for Anzoth CLI.")
        );
        assert!(model.availability_nux.is_none());
    }
}
