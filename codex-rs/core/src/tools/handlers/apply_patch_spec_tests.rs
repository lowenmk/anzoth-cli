use super::*;
use std::collections::BTreeMap;

use codex_tools::JsonSchema;
use codex_tools::ResponsesApiTool;
use pretty_assertions::assert_eq;

#[test]
fn create_apply_patch_freeform_tool_matches_expected_spec() {
    assert_eq!(
        create_apply_patch_freeform_tool(/*include_environment_id*/ false),
        ToolSpec::Freeform(FreeformTool {
            name: "apply_patch".to_string(),
            description: APPLY_PATCH_FREEFORM_DESCRIPTION.to_string(),
            format: FreeformToolFormat {
                r#type: "grammar".to_string(),
                syntax: "lark".to_string(),
                definition: APPLY_PATCH_LARK_GRAMMAR.to_string(),
            },
        })
    );
}

#[test]
fn create_apply_patch_freeform_tool_describes_canonical_patch_grammar() {
    let ToolSpec::Freeform(tool) = create_apply_patch_freeform_tool(false) else {
        panic!("expected freeform tool");
    };
    assert!(tool.description.contains("*** Begin Patch"));
    assert!(tool.description.contains("*** End Patch"));
    assert!(tool.description.contains("*** Add File: path"));
    assert!(tool.description.contains("+hello"));
    assert!(tool.description.contains("never JSON"));
    assert!(tool.description.contains("Do not use Markdown fences"));
}

#[test]
fn create_apply_patch_freeform_tool_includes_environment_id_when_requested() {
    let ToolSpec::Freeform(tool) =
        create_apply_patch_freeform_tool(/*include_environment_id*/ true)
    else {
        panic!("expected freeform tool");
    };

    assert!(tool.format.definition.contains("environment_id?"));
    assert!(
        tool.format
            .definition
            .contains("\"*** Environment ID: \" filename LF")
    );
}

#[test]
fn create_apply_patch_function_tool_matches_expected_spec() {
    assert_eq!(
        create_apply_patch_function_tool(/*include_environment_id*/ false),
        ToolSpec::Function(ResponsesApiTool {
            name: "apply_patch".to_string(),
            description:
                "Use the `apply_patch` tool to edit files. Pass the complete patch text in the `patch` field."
                    .to_string(),
            strict: true,
            defer_loading: None,
            parameters: JsonSchema::object(
                BTreeMap::from([(
                    "patch".to_string(),
                    JsonSchema::string(Some(
                        "The complete apply_patch patch text.".to_string(),
                    )),
                )]),
                Some(vec!["patch".to_string()]),
                Some(false.into()),
            ),
            output_schema: None,
        })
    );
}
