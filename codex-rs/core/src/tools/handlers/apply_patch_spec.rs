use std::collections::BTreeMap;

use codex_tools::FreeformTool;
use codex_tools::FreeformToolFormat;
use codex_tools::JsonSchema;
use codex_tools::ResponsesApiTool;
use codex_tools::ToolSpec;

const APPLY_PATCH_LARK_GRAMMAR: &str = include_str!("apply_patch.lark");

/// Returns a custom tool that can be used to edit files. Well-suited for GPT-5 models
/// https://platform.openai.com/docs/guides/function-calling#custom-tools
pub fn create_apply_patch_freeform_tool(include_environment_id: bool) -> ToolSpec {
    let definition = if include_environment_id {
        APPLY_PATCH_LARK_GRAMMAR.replace(
            "start: begin_patch hunk+ end_patch",
            "start: begin_patch environment_id? hunk+ end_patch\nenvironment_id: \"*** Environment ID: \" filename LF",
        )
    } else {
        APPLY_PATCH_LARK_GRAMMAR.to_string()
    };
    ToolSpec::Freeform(FreeformTool {
        name: "apply_patch".to_string(),
        description: "Use the `apply_patch` tool to edit files. This is a FREEFORM tool, so do not wrap the patch in JSON.".to_string(),
        format: FreeformToolFormat {
            r#type: "grammar".to_string(),
            syntax: "lark".to_string(),
            definition,
        },
    })
}

pub fn create_apply_patch_function_tool(include_environment_id: bool) -> ToolSpec {
    let mut description =
        "Use the `apply_patch` tool to edit files. Pass the complete patch text in the `patch` field."
            .to_string();
    if include_environment_id {
        description.push_str(
            " When a turn has multiple environments, include an `*** Environment ID: ...` line in the patch text if needed.",
        );
    }

    let properties = BTreeMap::from([(
        "patch".to_string(),
        JsonSchema::string(Some("The complete apply_patch patch text.".to_string())),
    )]);

    ToolSpec::Function(ResponsesApiTool {
        name: "apply_patch".to_string(),
        description,
        strict: true,
        defer_loading: None,
        parameters: JsonSchema::object(
            properties,
            Some(vec!["patch".to_string()]),
            Some(false.into()),
        ),
        output_schema: None,
    })
}

#[cfg(test)]
#[path = "apply_patch_spec_tests.rs"]
mod tests;
